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

/// Machine-readable safety summary written next to an export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetySummary {
    pub product_version: String,
    pub rule_pack_version: String,
    pub safety_status: SafetyStatus,
    pub structure_status: StructureStatus,
    /// Counts of replacements grouped by detector type (not by secret value).
    pub replacement_counts: BTreeMap<String, usize>,
    pub findings: Vec<SummaryFinding>,
    /// Human-readable limits disclaimer (non-claim language).
    pub disclaimer: String,
}

impl SafetySummary {
    pub fn from_scrub(
        findings: &[Finding],
        safety_status: SafetyStatus,
        structure_status: StructureStatus,
        rule_pack_version: &str,
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
            disclaimer: "SecretScrub redacts common patterns locally. It cannot guarantee every sensitive value is found. Review the safe copy before sharing.".to_string(),
        }
    }
}
