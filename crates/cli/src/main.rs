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
    atomic_write, ensure_not_source, export_workspace_tree, scrub_with_path, scrub_workspace,
    write_safety_summary, CancelFlag, ExitCodeKind, ExportError, FileInclusion, FileReport,
    ProgressEvent, RulePack, SafetySummary, ScrubConfig, ScrubError, StructureStatus,
    WorkspaceError, WorkspaceLimits, WorkspaceResult, PRODUCT_VERSION, RULE_PACK_VERSION,
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

        /// Fixed session seed for placeholder indices (testing). Default: random.
        #[arg(long = "session-seed", hide = true)]
        session_seed: Option<u64>,

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
    seed: u64,
    limits: WorkspaceLimits,
    progress: bool,
}

fn run() -> Result<ExitCodeKind, String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scrub {
            path,
            output,
            summary,
            force,
            format,
            session_seed,
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
                seed: session_seed.unwrap_or_else(random_seed),
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

fn scrub_stdin(run: &ScrubRun) -> Result<ExitCodeKind, String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("read stdin: {e}"))?;
    if buf.len() as u64 > run.limits.max_file_size {
        return Err(format!(
            "stdin exceeds max_file_size {}",
            run.limits.max_file_size
        ));
    }
    for line in buf.lines() {
        if line.len() > run.limits.max_line_length {
            return Err(format!(
                "line exceeds max_line_length {}",
                run.limits.max_line_length
            ));
        }
    }
    finish_single(&buf, None, run)
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
    let content =
        fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    for line in content.lines() {
        if line.len() > run.limits.max_line_length {
            return Err(format!(
                "line exceeds max_line_length {}",
                run.limits.max_line_length
            ));
        }
    }
    finish_single(&content, Some(path), run)
}

fn finish_single(
    content: &str,
    source: Option<&Path>,
    run: &ScrubRun,
) -> Result<ExitCodeKind, String> {
    let config = ScrubConfig {
        session_seed: run.seed,
        ..ScrubConfig::default()
    };
    let result = scrub_with_path(content, source, &config).map_err(|e| match e {
        ScrubError::EmptyInput => "input is empty".to_string(),
    })?;

    let fully_unsupported = result.structure_status == StructureStatus::Unsupported;

    let summary = SafetySummary::from_scrub(
        &result.findings,
        result.safety_status,
        result.structure_status,
        &result.rule_pack_version,
    );

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
        run.seed,
        RulePack::BuiltinV1,
        &run.limits,
        &cancel,
        Some(&mut progress_cb),
    )
    .map_err(ws_err)?;

    if result.cancelled {
        return Err("scrub cancelled".into());
    }

    let summary = workspace_summary(&result);

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

fn export_err(e: ExportError) -> String {
    e.to_string()
}

fn ws_err(e: WorkspaceError) -> String {
    e.to_string()
}

fn random_seed() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut h = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
        .hash(&mut h);
    std::process::id().hash(&mut h);
    h.finish()
}
