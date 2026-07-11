//! SecretScrub CLI — local-only scrub of a single file or stdin.
//!
//! Processing never uploads content. Detection covers common patterns only;
//! it cannot guarantee every sensitive value is found.

use clap::{Parser, Subcommand};
use secretscrub_core::{
    atomic_write, ensure_not_source, scrub, write_safety_summary, ExportError, SafetySummary,
    ScrubConfig, ScrubError, PRODUCT_VERSION, RULE_PACK_VERSION,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "secretscrub",
    version = PRODUCT_VERSION,
    about = "Local-first safe-share scrubber for logs and configs",
    long_about = "SecretScrub redacts common secrets and identifiers on your device.\n\
                  It does not upload artifacts and cannot guarantee every sensitive value is found.\n\
                  Review the safe copy before sharing."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scrub a file or stdin and emit a safe copy (local only)
    Scrub {
        /// Input file path. If omitted, read stdin.
        path: Option<PathBuf>,

        /// Write safe copy to this path (atomic). Default: stdout.
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Write machine-readable safety summary JSON to this path (never includes secret values).
        #[arg(long = "summary")]
        summary: Option<PathBuf>,

        /// Overwrite existing output/summary paths.
        #[arg(long = "force")]
        force: bool,

        /// Fixed session seed for placeholder indices (testing). Default: random.
        #[arg(long = "session-seed", hide = true)]
        session_seed: Option<u64>,

        /// Print findings summary to stderr as JSON (no secret values).
        #[arg(long = "findings-json")]
        findings_json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scrub {
            path,
            output,
            summary,
            force,
            session_seed,
            findings_json,
        } => scrub_command(path, output, summary, force, session_seed, findings_json),
    }
}

fn scrub_command(
    path: Option<PathBuf>,
    output: Option<PathBuf>,
    summary_path: Option<PathBuf>,
    force: bool,
    session_seed: Option<u64>,
    findings_json: bool,
) -> Result<ExitCode, String> {
    let (content, source) = read_input(path.as_deref())?;
    let seed = session_seed.unwrap_or_else(random_seed);
    let config = ScrubConfig {
        session_seed: seed,
        ..ScrubConfig::default()
    };

    let result = scrub(&content, &config).map_err(|e| match e {
        ScrubError::EmptyInput => "input is empty".to_string(),
    })?;

    if let Some(ref dest) = output {
        ensure_not_source(source.as_deref(), dest).map_err(export_err)?;
        atomic_write(dest, &result.text, force).map_err(export_err)?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(result.text.as_bytes())
            .map_err(|e| e.to_string())?;
        if !result.text.ends_with('\n') {
            let _ = stdout.write_all(b"\n");
        }
    }

    if let Some(ref sp) = summary_path {
        let summary = SafetySummary::from_scrub(
            &result.findings,
            result.safety_status,
            result.structure_status,
            &result.rule_pack_version,
        );
        write_safety_summary(sp, &summary, force).map_err(export_err)?;
    }

    if findings_json {
        let summary = SafetySummary::from_scrub(
            &result.findings,
            result.safety_status,
            result.structure_status,
            &result.rule_pack_version,
        );
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
        );
    } else if output.is_some() {
        eprintln!(
            "secretscrub: local scrub complete (rule_pack={RULE_PACK_VERSION}, findings={})",
            result.findings.len()
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn read_input(path: Option<&std::path::Path>) -> Result<(String, Option<PathBuf>), String> {
    match path {
        Some(p) => {
            let s = fs::read_to_string(p).map_err(|e| format!("read {}: {e}", p.display()))?;
            Ok((s, Some(p.to_path_buf())))
        }
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("read stdin: {e}"))?;
            Ok((buf, None))
        }
    }
}

fn export_err(e: ExportError) -> String {
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
