use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Product version embedded in safety summaries.
pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Export / scrub safety status shown to users and automation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyStatus {
    /// Supported content reviewed by detectors; still not a universal guarantee.
    SafeCopyReady,
    /// Something needs human review (unsupported format, partial, etc.).
    ReviewRequired,
}

/// Structural validation after transform (plain text is not applicable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureStatus {
    NotApplicable,
    Valid,
    Invalid,
    Unsupported,
}

/// CLI / automation exit codes (stable contract).
///
/// | Code | Meaning |
/// |------|---------|
/// | 0 | Clean completion (`safe_copy_ready`) |
/// | 1 | Execution failure (IO, empty input, cancel during export, etc.) |
/// | 2 | Completed with `review_required` |
/// | 3 | Unsupported input (nothing safe produced) |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCodeKind {
    Clean = 0,
    Failure = 1,
    ReviewRequired = 2,
    Unsupported = 3,
}

impl ExitCodeKind {
    pub fn from_statuses(
        safety: SafetyStatus,
        fully_unsupported: bool,
        hard_failure: bool,
    ) -> Self {
        if hard_failure {
            return ExitCodeKind::Failure;
        }
        if fully_unsupported {
            return ExitCodeKind::Unsupported;
        }
        match safety {
            SafetyStatus::SafeCopyReady => ExitCodeKind::Clean,
            SafetyStatus::ReviewRequired => ExitCodeKind::ReviewRequired,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// One aggregated finding for a sensitive value class within a workspace scrub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Detector / placeholder type name, e.g. `AWS_ACCESS_KEY`.
    pub detector_type: String,
    /// Semantic placeholder assigned in this workspace, e.g. `[AWS_ACCESS_KEY#1]`.
    pub placeholder: String,
    /// Number of occurrences of this exact original value.
    pub occurrences: usize,
}

/// Finding entry in a safety summary (never includes secret values).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryFinding {
    pub detector_type: String,
    pub placeholder: String,
    pub occurrences: usize,
}

/// Per-file report for multi-file workspaces (no secret values).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileReport {
    pub path: String,
    pub status: String,
    pub reason: Option<String>,
    pub structure_status: StructureStatus,
    pub safety_status: SafetyStatus,
    pub findings_count: usize,
    /// SHA-256 (hex) of the exported safe file bytes when the file was written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// Machine-readable safety summary written next to an export.
///
/// When sealed for attestation, includes content hash(es), `hash_scheme`, and
/// `created_at` so a recipient can run `secretscrub verify`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetySummary {
    pub product_version: String,
    pub rule_pack_version: String,
    pub safety_status: SafetyStatus,
    pub structure_status: StructureStatus,
    /// Counts of replacements grouped by detector type (not by secret value).
    pub replacement_counts: BTreeMap<String, usize>,
    pub findings: Vec<SummaryFinding>,
    /// Present for folder/workspace scrubs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileReport>>,
    /// Human-readable limits disclaimer (non-claim language).
    pub disclaimer: String,
    /// Self-describing hash construction id (e.g. `sha256-single-v1`).
    #[serde(default)]
    pub hash_scheme: String,
    /// Aggregate content SHA-256 (hex) of the exported safe copy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
    /// UTC time the summary was sealed (RFC 3339).
    #[serde(default)]
    pub created_at: String,
}

impl SafetySummary {
    pub fn from_scrub(
        findings: &[Finding],
        safety_status: SafetyStatus,
        structure_status: StructureStatus,
        rule_pack_version: &str,
    ) -> Self {
        Self::build(findings, safety_status, structure_status, rule_pack_version, None)
    }

    pub fn build(
        findings: &[Finding],
        safety_status: SafetyStatus,
        structure_status: StructureStatus,
        rule_pack_version: &str,
        files: Option<Vec<FileReport>>,
    ) -> Self {
        let mut replacement_counts: BTreeMap<String, usize> = BTreeMap::new();
        for f in findings {
            *replacement_counts
                .entry(f.detector_type.clone())
                .or_insert(0) += f.occurrences;
        }
        let summary_findings = findings
            .iter()
            .map(|f| SummaryFinding {
                detector_type: f.detector_type.clone(),
                placeholder: f.placeholder.clone(),
                occurrences: f.occurrences,
            })
            .collect();
        Self {
            product_version: PRODUCT_VERSION.to_string(),
            rule_pack_version: rule_pack_version.to_string(),
            safety_status,
            structure_status,
            replacement_counts,
            findings: summary_findings,
            files,
            disclaimer: "SecretScrub redacts common patterns locally. It cannot guarantee every sensitive value is found. Review the safe copy before sharing.".to_string(),
            hash_scheme: String::new(),
            content_sha256: None,
            created_at: String::new(),
        }
    }
}
