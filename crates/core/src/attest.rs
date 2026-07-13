//! Content attestation hashes for safety summaries and `verify`.
//!
//! Hash schemes (self-describing via `hash_scheme` on the summary):
//! - `sha256-single-v1`: SHA-256 of the exported safe file bytes (UTF-8 text as written).
//! - `sha256-workspace-v1`: per included file SHA-256 of its bytes; `content_sha256` is
//!   SHA-256 over lines `"{path}\0{file_sha256}\n"` sorted by path (path uses `/`).

use crate::types::SafetySummary;
use crate::workspace::{FileArtifact, WorkspaceResult};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

/// Single-file export attestation scheme.
pub const HASH_SCHEME_SINGLE_V1: &str = "sha256-single-v1";
/// Workspace tree attestation scheme.
pub const HASH_SCHEME_WORKSPACE_V1: &str = "sha256-workspace-v1";

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("summary JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("summary missing content_sha256")]
    MissingContentHash,
    #[error("unsupported hash_scheme: {0}")]
    UnsupportedScheme(String),
    #[error("content hash mismatch")]
    ContentMismatch,
    #[error("file hash mismatch: {0}")]
    FileMismatch(String),
    #[error("expected a file path for single-file summary")]
    ExpectedFile,
    #[error("expected a directory path for workspace summary")]
    ExpectedDirectory,
    #[error("safe copy missing file listed in summary: {0}")]
    MissingFile(String),
    #[error("summary metadata mismatch: {0}")]
    MetadataMismatch(String),
}

/// Hex-encoded SHA-256 of raw bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

/// UTC timestamp as RFC 3339 (second precision) without external time crates.
pub fn utc_timestamp_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_unix_secs_rfc3339(secs)
}

fn format_unix_secs_rfc3339(secs: u64) -> String {
    // Civil date from Unix seconds (proleptic Gregorian, UTC).
    let days = (secs / 86_400) as i64;
    let tod = secs % 86_400;
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;

    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Howard Hinnant civil_from_days (UTC days since 1970-01-01).
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

/// Attach single-file attestation fields to a summary (mutates in place).
pub fn seal_single_file(summary: &mut SafetySummary, safe_text: &str) {
    summary.hash_scheme = HASH_SCHEME_SINGLE_V1.to_string();
    summary.content_sha256 = Some(sha256_hex(safe_text.as_bytes()));
    if summary.created_at.is_empty() {
        summary.created_at = utc_timestamp_rfc3339();
    }
    // Clear any workspace file digests.
    if let Some(files) = summary.files.as_mut() {
        for f in files.iter_mut() {
            f.sha256 = None;
        }
    }
}

/// Build workspace file digests and aggregate content hash from scrub artifacts.
pub fn seal_workspace(summary: &mut SafetySummary, result: &WorkspaceResult) {
    summary.hash_scheme = HASH_SCHEME_WORKSPACE_V1.to_string();
    if summary.created_at.is_empty() {
        summary.created_at = utc_timestamp_rfc3339();
    }

    let digests = workspace_file_digests(&result.files);
    summary.content_sha256 = Some(aggregate_workspace_hash(&digests));

    if let Some(files) = summary.files.as_mut() {
        for f in files.iter_mut() {
            f.sha256 = digests
                .iter()
                .find(|(p, _)| p == &f.path)
                .map(|(_, h)| h.clone());
        }
    }
}

/// `(relative_path_with_slashes, sha256_hex)` for every artifact that has exportable text.
pub fn workspace_file_digests(files: &[FileArtifact]) -> Vec<(String, String)> {
    let mut digests: Vec<(String, String)> = files
        .iter()
        .filter_map(|f| {
            let text = f.text.as_ref()?;
            // Only hash files that would be written by export (text present).
            let path = normalize_rel_path(&f.relative_path);
            Some((path, sha256_hex(text.as_bytes())))
        })
        .collect();
    digests.sort_by(|a, b| a.0.cmp(&b.0));
    digests
}

fn normalize_rel_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Aggregate workspace content hash from sorted `(path, file_sha256)` pairs.
pub fn aggregate_workspace_hash(digests: &[(String, String)]) -> String {
    let mut hasher = Sha256::new();
    for (path, file_hash) in digests {
        hasher.update(path.as_bytes());
        hasher.update([0u8]);
        hasher.update(file_hash.as_bytes());
        hasher.update([b'\n']);
    }
    hex::encode(hasher.finalize())
}

/// Result of a successful verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    pub ok: bool,
    pub hash_scheme: String,
    pub content_sha256: String,
    pub product_version: String,
    pub rule_pack_version: String,
    /// Files that mismatched (workspace only).
    pub mismatched_files: Vec<String>,
}

/// Verify a safe copy against a safety summary receipt.
pub fn verify_safe_copy(safe_path: &Path, summary: &SafetySummary) -> Result<VerifyReport, VerifyError> {
    let expected = summary
        .content_sha256
        .as_ref()
        .ok_or(VerifyError::MissingContentHash)?;

    match summary.hash_scheme.as_str() {
        HASH_SCHEME_SINGLE_V1 => verify_single(safe_path, summary, expected),
        HASH_SCHEME_WORKSPACE_V1 => verify_workspace(safe_path, summary, expected),
        other => Err(VerifyError::UnsupportedScheme(other.to_string())),
    }
}

fn verify_single(
    safe_path: &Path,
    summary: &SafetySummary,
    expected: &str,
) -> Result<VerifyReport, VerifyError> {
    if !safe_path.is_file() {
        return Err(VerifyError::ExpectedFile);
    }
    let bytes = fs::read(safe_path)?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(VerifyError::ContentMismatch);
    }
    Ok(VerifyReport {
        ok: true,
        hash_scheme: summary.hash_scheme.clone(),
        content_sha256: actual,
        product_version: summary.product_version.clone(),
        rule_pack_version: summary.rule_pack_version.clone(),
        mismatched_files: vec![],
    })
}

fn verify_workspace(
    safe_root: &Path,
    summary: &SafetySummary,
    expected: &str,
) -> Result<VerifyReport, VerifyError> {
    if !safe_root.is_dir() {
        return Err(VerifyError::ExpectedDirectory);
    }

    let files = summary.files.as_ref().ok_or_else(|| {
        VerifyError::MetadataMismatch("workspace summary missing files[]".into())
    })?;

    let mut digests: Vec<(String, String)> = Vec::new();
    let mut mismatched: Vec<String> = Vec::new();

    for f in files {
        // Only files that were sealed with a digest are required on disk.
        let Some(expected_file_hash) = f.sha256.as_ref() else {
            continue;
        };
        let path = safe_root.join(f.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !path.is_file() {
            return Err(VerifyError::MissingFile(f.path.clone()));
        }
        let bytes = fs::read(&path)?;
        let actual = sha256_hex(&bytes);
        if &actual != expected_file_hash {
            mismatched.push(f.path.clone());
        }
        digests.push((f.path.clone(), actual));
    }

    digests.sort_by(|a, b| a.0.cmp(&b.0));
    let actual_aggregate = aggregate_workspace_hash(&digests);

    if !mismatched.is_empty() {
        return Err(VerifyError::FileMismatch(mismatched.join(", ")));
    }
    if actual_aggregate != expected {
        return Err(VerifyError::ContentMismatch);
    }

    Ok(VerifyReport {
        ok: true,
        hash_scheme: summary.hash_scheme.clone(),
        content_sha256: actual_aggregate,
        product_version: summary.product_version.clone(),
        rule_pack_version: summary.rule_pack_version.clone(),
        mismatched_files: vec![],
    })
}

/// Load a safety summary from a JSON file.
pub fn load_summary(path: &Path) -> Result<SafetySummary, VerifyError> {
    let body = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&body)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SafetyStatus, StructureStatus, SummaryFinding};
    use std::collections::BTreeMap;

    #[test]
    fn single_file_hash_is_stable() {
        assert_eq!(
            sha256_hex(b"hello\n"),
            "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
    }

    #[test]
    fn workspace_aggregate_order_independent_of_input_order() {
        let mut a: Vec<(String, String)> = vec![
            ("b.log".into(), "aa".into()),
            ("a.log".into(), "bb".into()),
        ];
        let mut b: Vec<(String, String)> = vec![
            ("a.log".into(), "bb".into()),
            ("b.log".into(), "aa".into()),
        ];
        a.sort_by(|x, y| x.0.cmp(&y.0));
        b.sort_by(|x, y| x.0.cmp(&y.0));
        assert_eq!(aggregate_workspace_hash(&a), aggregate_workspace_hash(&b));
    }

    #[test]
    fn civil_epoch() {
        assert_eq!(format_unix_secs_rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_unix_secs_rfc3339(86_400), "1970-01-02T00:00:00Z");
    }

    #[test]
    fn seal_single_sets_scheme_and_hash() {
        let mut s = SafetySummary {
            product_version: "0.1.0".into(),
            rule_pack_version: "0.1.0".into(),
            safety_status: SafetyStatus::SafeCopyReady,
            structure_status: StructureStatus::NotApplicable,
            replacement_counts: BTreeMap::new(),
            findings: vec![SummaryFinding {
                detector_type: "AWS_ACCESS_KEY".into(),
                placeholder: "[AWS_ACCESS_KEY#1]".into(),
                occurrences: 1,
            }],
            files: None,
            disclaimer: "x".into(),
            hash_scheme: String::new(),
            content_sha256: None,
            created_at: String::new(),
        };
        seal_single_file(&mut s, "safe\n");
        assert_eq!(s.hash_scheme, HASH_SCHEME_SINGLE_V1);
        assert_eq!(s.content_sha256.as_deref(), Some(sha256_hex(b"safe\n").as_str()));
        assert!(s.created_at.ends_with('Z'));
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("AKIA"));
    }
}
