//! SecretScrub CLI — local-only scrub of files, folders, or stdin.
//!
//! Exit codes (stable automation contract):
//!   0  clean (`safe_copy_ready`)
//!   1  failure (IO, empty input, cancel during export, etc.)
//!   2  review required
//!   3  unsupported (nothing safe produced)
//!
//! Processing never uploads content. Detection covers common patterns only.

use clap::{Parser, Subcommand, ValueEnum};
use secretscrub_core::{
    atomic_write, ensure_not_source, export_workspace_tree, load_summary, looks_binary,
    scrub_with_path, scrub_workspace, seal_single_file, seal_workspace, verify_safe_copy,
    write_safety_summary, CancelFlag, ContentFormat, ExitCodeKind, ExportError, FileInclusion,
    FileReport, ProgressEvent, RulePack, SafetyStatus, SafetySummary, ScrubConfig, ScrubError,
    ScrubResult, StructureStatus, VerifyError, WorkspaceError, WorkspaceLimits, WorkspaceResult,
    PRODUCT_VERSION, RULE_PACK_VERSION,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "secretscrub",
    version = PRODUCT_VERSION,
    about = "Local-first safe-share scrubber for logs and configs",
    long_about = "SecretScrub redacts common secrets and identifiers on your device.\n\
                  It does not upload artifacts and cannot guarantee every sensitive value is found.\n\
                  Review the safe copy before sharing.\n\n\
                  Exit codes: 0 clean, 1 failure, 2 review_required, 3 unsupported."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    /// Safe copy to stdout or --output (default)
    Text,
    /// Machine-readable findings/workspace report on stdout (no secret values)
    Json,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scrub a file, directory, or stdin (local only)
    Scrub {
        /// Input file or directory. If omitted, read stdin.
        path: Option<PathBuf>,

        /// Write safe copy (file) or safe tree (directory) here. Default for files: stdout.
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Write machine-readable safety summary JSON to this path (never includes secret values).
        #[arg(long = "summary")]
        summary: Option<PathBuf>,

        /// Overwrite existing output/summary paths.
        #[arg(long = "force")]
        force: bool,

        /// Report format: text (safe copy) or json (findings report on stdout).
        #[arg(long = "format", value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Max directory depth for folder scrubs.
        #[arg(long = "max-depth")]
        max_depth: Option<usize>,

        /// Max bytes per file.
        #[arg(long = "max-file-size")]
        max_file_size: Option<u64>,

        /// Max files in a folder scrub.
        #[arg(long = "max-files")]
        max_files: Option<usize>,

        /// Max characters per line (plain/env).
        #[arg(long = "max-line-length")]
        max_line_length: Option<usize>,

        /// Print progress events on stderr.
        #[arg(long = "progress")]
        progress: bool,
    },

    /// Verify a safe copy against a safety summary receipt (hash integrity only)
    Verify {
        /// Path to the exported safe file or workspace directory.
        path: PathBuf,

        /// Path to the safety summary JSON written at scrub time.
        #[arg(long = "summary")]
        summary: PathBuf,

        /// Report format: text (default) or json.
        #[arg(long = "format", value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(kind) => ExitCode::from(kind.as_u8()),
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(ExitCodeKind::Failure.as_u8())
        }
    }
}

/// Bundled runtime options for a scrub invocation (keeps helper arity low).
struct ScrubRun {
    path: Option<PathBuf>,
    output: Option<PathBuf>,
    summary_path: Option<PathBuf>,
    force: bool,
    format: OutputFormat,
    limits: WorkspaceLimits,
    progress: bool,
}

fn run() -> Result<ExitCodeKind, String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Verify {
            path,
            summary,
            format,
        } => verify_command(&path, &summary, format),
        Commands::Scrub {
            path,
            output,
            summary,
            force,
            format,
            max_depth,
            max_file_size,
            max_files,
            max_line_length,
            progress,
        } => {
            let mut limits = WorkspaceLimits::default();
            if let Some(d) = max_depth {
                limits.max_depth = d;
            }
            if let Some(s) = max_file_size {
                limits.max_file_size = s;
            }
            if let Some(n) = max_files {
                limits.max_file_count = n;
            }
            if let Some(l) = max_line_length {
                limits.max_line_length = l;
            }
            scrub_command(ScrubRun {
                path,
                output,
                summary_path: summary,
                force,
                format,
                limits,
                progress,
            })
        }
    }
}

fn scrub_command(run: ScrubRun) -> Result<ExitCodeKind, String> {
    match run.path {
        Some(ref p) if p.is_dir() => scrub_dir(p, &run),
        Some(ref p) => scrub_file(p, &run),
        None => scrub_stdin(&run),
    }
}

/// Bytes classified as text (with the same binary sniff used for workspace
/// scrubs) or rejected as unsupported before any scrub attempt.
enum BytesCheck {
    Text(String),
    Unsupported(&'static str),
}

fn check_bytes(bytes: Vec<u8>) -> BytesCheck {
    if looks_binary(&bytes) {
        return BytesCheck::Unsupported("binary or non-text content is unsupported");
    }
    match String::from_utf8(bytes) {
        Ok(s) => BytesCheck::Text(s),
        Err(_) => BytesCheck::Unsupported("invalid UTF-8"),
    }
}

/// Synthetic result for input rejected by the binary/UTF-8 check: no safe
/// copy is ever produced for it.
fn unsupported_result(reason: &str) -> ScrubResult {
    ScrubResult {
        text: String::new(),
        findings: Vec::new(),
        safety_status: SafetyStatus::ReviewRequired,
        structure_status: StructureStatus::Unsupported,
        rule_pack_version: RULE_PACK_VERSION.to_string(),
        note: Some(reason.to_string()),
        format: ContentFormat::BinaryUnsupported,
    }
}

fn scrub_stdin(run: &ScrubRun) -> Result<ExitCodeKind, String> {
    let mut bytes = Vec::new();
    io::stdin()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("read stdin: {e}"))?;
    if bytes.len() as u64 > run.limits.max_file_size {
        return Err(format!(
            "stdin exceeds max_file_size {}",
            run.limits.max_file_size
        ));
    }
    let content = match check_bytes(bytes) {
        BytesCheck::Text(s) => s,
        BytesCheck::Unsupported(reason) => {
            return finish_single(unsupported_result(reason), None, run);
        }
    };
    for line in content.lines() {
        if line.len() > run.limits.max_line_length {
            return Err(format!(
                "line exceeds max_line_length {}",
                run.limits.max_line_length
            ));
        }
    }
    let result = scrub_with_path(&content, None, &ScrubConfig::default()).map_err(|e| match e {
        ScrubError::EmptyInput => "input is empty".to_string(),
    })?;
    finish_single(result, None, run)
}

fn scrub_file(path: &Path, run: &ScrubRun) -> Result<ExitCodeKind, String> {
    let meta = fs::metadata(path).map_err(|e| format!("stat {}: {e}", path.display()))?;
    if meta.len() > run.limits.max_file_size {
        return Err(format!(
            "file size {} exceeds max_file_size {}",
            meta.len(),
            run.limits.max_file_size
        ));
    }
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let content = match check_bytes(bytes) {
        BytesCheck::Text(s) => s,
        BytesCheck::Unsupported(reason) => {
            return finish_single(unsupported_result(reason), Some(path), run);
        }
    };
    for line in content.lines() {
        if line.len() > run.limits.max_line_length {
            return Err(format!(
                "line exceeds max_line_length {}",
                run.limits.max_line_length
            ));
        }
    }
    let result = scrub_with_path(&content, Some(path), &ScrubConfig::default()).map_err(
        |e| match e {
            ScrubError::EmptyInput => "input is empty".to_string(),
        },
    )?;
    finish_single(result, Some(path), run)
}

fn finish_single(
    result: ScrubResult,
    source: Option<&Path>,
    run: &ScrubRun,
) -> Result<ExitCodeKind, String> {
    let fully_unsupported = result.structure_status == StructureStatus::Unsupported;

    let mut summary = SafetySummary::from_scrub(
        &result.findings,
        result.safety_status,
        result.structure_status,
        &result.rule_pack_version,
    );
    seal_single_file(&mut summary, &result.text);

    match run.format {
        OutputFormat::Json => {
            if let Some(ref dest) = run.output {
                if !result.text.is_empty() {
                    ensure_not_source(source, dest).map_err(export_err)?;
                    atomic_write(dest, &result.text, run.force).map_err(export_err)?;
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
            );
        }
        OutputFormat::Text => {
            if let Some(ref dest) = run.output {
                if !result.text.is_empty() {
                    ensure_not_source(source, dest).map_err(export_err)?;
                    atomic_write(dest, &result.text, run.force).map_err(export_err)?;
                }
            } else if !result.text.is_empty() {
                let mut stdout = io::stdout().lock();
                stdout
                    .write_all(result.text.as_bytes())
                    .map_err(|e| e.to_string())?;
                if !result.text.ends_with('\n') {
                    let _ = stdout.write_all(b"\n");
                }
            }
            if let Some(ref sp) = run.summary_path {
                write_safety_summary(sp, &summary, run.force).map_err(export_err)?;
            }
            if run.output.is_some() {
                eprintln!(
                    "secretscrub: local scrub complete (rule_pack={RULE_PACK_VERSION}, findings={}, status={:?})",
                    result.findings.len(),
                    result.safety_status
                );
            }
        }
    }

    if let Some(ref sp) = run.summary_path {
        if matches!(run.format, OutputFormat::Json) {
            write_safety_summary(sp, &summary, run.force).map_err(export_err)?;
        }
    }

    Ok(ExitCodeKind::from_statuses(
        result.safety_status,
        fully_unsupported,
        false,
    ))
}

fn scrub_dir(root: &Path, run: &ScrubRun) -> Result<ExitCodeKind, String> {
    let cancel = CancelFlag::new();
    let show_progress = run.progress;
    let mut progress_cb = |ev: ProgressEvent| {
        if show_progress {
            match ev {
                ProgressEvent::WorkspaceStarted { root } => {
                    eprintln!("progress: workspace_start {root}")
                }
                ProgressEvent::FileStarted { path } => eprintln!("progress: file_start {path}"),
                ProgressEvent::FileFinished { path, included } => {
                    eprintln!("progress: file_done {path} included={included}")
                }
                ProgressEvent::WorkspaceFinished { file_count } => {
                    eprintln!("progress: workspace_done files={file_count}")
                }
                ProgressEvent::Cancelled => eprintln!("progress: cancelled"),
                ProgressEvent::LimitHit { kind, detail } => {
                    eprintln!("progress: limit {kind} {detail}")
                }
            }
        }
    };

    let result = scrub_workspace(
        root,
        RulePack::BuiltinV1,
        &run.limits,
        &cancel,
        Some(&mut progress_cb),
    )
    .map_err(ws_err)?;

    if result.cancelled {
        return Err("scrub cancelled".into());
    }

    let mut summary = workspace_summary(&result);
    seal_workspace(&mut summary, &result);

    if let Some(ref dest) = run.output {
        ensure_not_source(Some(root), dest).map_err(export_err)?;
        export_workspace_tree(&result, dest, run.force, &cancel).map_err(|e| match e {
            WorkspaceError::Cancelled => {
                "export cancelled; destination cleaned when possible".into()
            }
            other => ws_err(other),
        })?;
    } else if matches!(run.format, OutputFormat::Text) {
        return Err("directory scrub requires --output <dir>".into());
    }

    match run.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
            );
        }
        OutputFormat::Text => {
            if let Some(ref sp) = run.summary_path {
                write_safety_summary(sp, &summary, run.force).map_err(export_err)?;
            }
            eprintln!(
                "secretscrub: workspace scrub complete (files={}, findings={}, status={:?})",
                result.files.len(),
                result.findings.len(),
                result.safety_status
            );
        }
    }

    if let Some(ref sp) = run.summary_path {
        if matches!(run.format, OutputFormat::Json) {
            write_safety_summary(sp, &summary, run.force).map_err(export_err)?;
        }
    }

    Ok(ExitCodeKind::from_statuses(
        result.safety_status,
        result.is_fully_unsupported(),
        false,
    ))
}

fn workspace_summary(result: &WorkspaceResult) -> SafetySummary {
    let files: Vec<FileReport> = result
        .files
        .iter()
        .map(|f| FileReport {
            path: f.outcome.path.clone(),
            status: match f.outcome.inclusion {
                FileInclusion::Included => "included".into(),
                FileInclusion::Excluded => "excluded".into(),
                FileInclusion::Unsupported => "unsupported".into(),
            },
            reason: f.outcome.reason.clone(),
            structure_status: f.outcome.structure_status,
            safety_status: f.outcome.safety_status,
            findings_count: f.outcome.findings_count,
            sha256: None,
        })
        .collect();
    SafetySummary::build(
        &result.findings,
        result.safety_status,
        result.structure_status,
        &result.rule_pack_version,
        Some(files),
    )
}

fn verify_command(
    path: &Path,
    summary_path: &Path,
    format: OutputFormat,
) -> Result<ExitCodeKind, String> {
    let summary = load_summary(summary_path).map_err(verify_err)?;
    match verify_safe_copy(path, &summary) {
        Ok(report) => {
            match format {
                OutputFormat::Json => {
                    let v = serde_json::json!({
                        "ok": true,
                        "hash_scheme": report.hash_scheme,
                        "content_sha256": report.content_sha256,
                        "product_version": report.product_version,
                        "rule_pack_version": report.rule_pack_version,
                    });
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?
                    );
                }
                OutputFormat::Text => {
                    eprintln!(
                        "verify ok: content matches summary (scheme={}, rule_pack={}, product={})",
                        report.hash_scheme, report.rule_pack_version, report.product_version
                    );
                }
            }
            Ok(ExitCodeKind::Clean)
        }
        Err(e) => {
            match format {
                OutputFormat::Json => {
                    let v = serde_json::json!({
                        "ok": false,
                        "error": e.to_string(),
                        "hash_scheme": summary.hash_scheme,
                        "product_version": summary.product_version,
                        "rule_pack_version": summary.rule_pack_version,
                    });
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&v).map_err(|err| err.to_string())?
                    );
                }
                OutputFormat::Text => {
                    eprintln!("verify failed: {e}");
                }
            }
            // Mismatch / verify failure is exit 1 per issue contract.
            match e {
                VerifyError::Io(_) | VerifyError::Json(_) => Err(e.to_string()),
                _ => Ok(ExitCodeKind::Failure),
            }
        }
    }
}

fn verify_err(e: VerifyError) -> String {
    e.to_string()
}

fn export_err(e: ExportError) -> String {
    e.to_string()
}

fn ws_err(e: WorkspaceError) -> String {
    e.to_string()
}
