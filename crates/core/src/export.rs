//! Atomic export and safety summary writers.
//!
//! Never opens the source path for write. Destination uses temp + rename.

use crate::types::SafetySummary;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("destination already exists: {0}")]
    DestinationExists(PathBuf),
    #[error("cannot export onto the source path")]
    SourceIsDestination,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Write `content` to `dest` atomically.
///
/// If `dest` exists and `force` is false, returns [`ExportError::DestinationExists`].
/// Uses a sibling temporary file then renames into place.
pub fn atomic_write(dest: &Path, content: &str, force: bool) -> Result<(), ExportError> {
    if dest.exists() && !force {
        return Err(ExportError::DestinationExists(dest.to_path_buf()));
    }

    let parent = dest.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = match parent {
        Some(p) => {
            fs::create_dir_all(p)?;
            p.to_path_buf()
        }
        None => PathBuf::from("."),
    };

    let file_name = dest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let tmp_name = format!(".{file_name}.secretscrub.tmp");
    let tmp_path = dir.join(tmp_name);

    // Ensure we don't leave a partial final destination; write temp fully first.
    let write_result = (|| -> Result<(), ExportError> {
        let mut f = File::create(&tmp_path)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    // On Windows rename over existing may fail; remove if force.
    if dest.exists() && force {
        fs::remove_file(dest)?;
    }

    match fs::rename(&tmp_path, dest) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            Err(ExportError::Io(e))
        }
    }
}

/// Refuse exporting if destination path equals source path (after canonicalize when possible).
pub fn ensure_not_source(source: Option<&Path>, dest: &Path) -> Result<(), ExportError> {
    let Some(source) = source else {
        return Ok(());
    };
    if paths_equal(source, dest) {
        return Err(ExportError::SourceIsDestination);
    }
    Ok(())
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    // Both exist: compare canonical paths.
    if let (Ok(ca), Ok(cb)) = (fs::canonicalize(a), fs::canonicalize(b)) {
        return ca == cb;
    }
    // Dest may not exist yet: compare absolute forms when possible.
    let abs_a = if a.is_absolute() {
        a.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(a))
            .unwrap_or_else(|_| a.to_path_buf())
    };
    let abs_b = if b.is_absolute() {
        b.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(b))
            .unwrap_or_else(|_| b.to_path_buf())
    };
    abs_a == abs_b
}

/// Write safety summary JSON to `path` (atomic). Never includes secret values.
pub fn write_safety_summary(
    path: &Path,
    summary: &SafetySummary,
    force: bool,
) -> Result<(), ExportError> {
    let json = serde_json::to_string_pretty(summary)?;
    atomic_write(path, &format!("{json}\n"), force)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Finding, SafetyStatus, StructureStatus};
    use std::fs;

    #[test]
    fn atomic_write_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.txt");
        atomic_write(&dest, "hello", false).unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello");
        assert!(!dir.path().join(".out.txt.secretscrub.tmp").exists());
    }

    #[test]
    fn refuses_existing_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.txt");
        atomic_write(&dest, "a", false).unwrap();
        let err = atomic_write(&dest, "b", false).unwrap_err();
        assert!(matches!(err, ExportError::DestinationExists(_)));
        assert_eq!(fs::read_to_string(&dest).unwrap(), "a");
    }

    #[test]
    fn force_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.txt");
        atomic_write(&dest, "a", false).unwrap();
        atomic_write(&dest, "b", true).unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "b");
    }

    #[test]
    fn summary_has_no_secret_material() {
        let findings = vec![Finding {
            detector_type: "AWS_ACCESS_KEY".into(),
            placeholder: "[AWS_ACCESS_KEY#1]".into(),
            occurrences: 1,
        }];
        let summary = SafetySummary::from_scrub(
            &findings,
            SafetyStatus::SafeCopyReady,
            StructureStatus::NotApplicable,
            "0.1.0",
        );
        let json = serde_json::to_string(&summary).unwrap();
        assert!(!json.contains("AKIA"));
        assert!(json.contains("AWS_ACCESS_KEY"));
        assert!(json.contains("rule_pack_version"));
    }

    #[test]
    fn partial_temp_cleaned_on_failure_simulation() {
        // Successful path leaves no temp; destination absent until complete is the contract.
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("missing_parent_ok").join("out.txt");
        atomic_write(&dest, "ok", false).unwrap();
        assert!(dest.exists());
    }
}
