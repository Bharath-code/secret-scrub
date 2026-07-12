//! Scrub API: content + config → transformed text + findings + status.

use crate::format::{format_from_path, ContentFormat};
use crate::placeholder::PlaceholderAllocator;
use crate::rulepack::RulePack;
use crate::structure::{findings_from_counts, scrub_structured};
use crate::types::{Finding, SafetyStatus, StructureStatus};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScrubError {
    #[error("input is empty")]
    EmptyInput,
}

/// Options for a single workspace scrub.
#[derive(Debug, Clone)]
pub struct ScrubConfig {
    pub rule_pack: RulePack,
    /// No-op, kept for API/CLI compatibility. Placeholder indices are
    /// per-type sequential in first-seen order; correlation is per-workspace
    /// by design.
    pub session_seed: u64,
    /// Optional format override; when None, inferred from path or plain text.
    pub format: Option<ContentFormat>,
}

impl Default for ScrubConfig {
    fn default() -> Self {
        Self {
            rule_pack: RulePack::BuiltinV1,
            session_seed: 0,
            format: None,
        }
    }
}

/// Result of scrubbing a single artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrubResult {
    pub text: String,
    pub findings: Vec<Finding>,
    pub safety_status: SafetyStatus,
    pub structure_status: StructureStatus,
    pub rule_pack_version: String,
    pub note: Option<String>,
    pub format: ContentFormat,
}

/// Redact sensitive values in `content` (plain text / structure auto from path).
pub fn scrub(content: &str, config: &ScrubConfig) -> Result<ScrubResult, ScrubError> {
    scrub_with_path(content, None, config)
}

/// Redact content with optional path for format inference.
pub fn scrub_with_path(
    content: &str,
    path: Option<&Path>,
    config: &ScrubConfig,
) -> Result<ScrubResult, ScrubError> {
    if content.is_empty() {
        return Err(ScrubError::EmptyInput);
    }

    let format = config.format.unwrap_or_else(|| {
        path.map(format_from_path)
            .unwrap_or(ContentFormat::PlainText)
    });

    let mut allocator = PlaceholderAllocator::new();
    let structured = scrub_structured(content, format, &mut allocator);
    let findings = findings_from_counts(structured.counts, &allocator);

    Ok(ScrubResult {
        text: structured.text,
        findings,
        safety_status: structured.safety_status,
        structure_status: structured.structure_status,
        rule_pack_version: config.rule_pack.version().to_string(),
        note: structured.note,
        format,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const AWS: &str = "AKIAIOSFODNN7EXAMPLE";

    #[test]
    fn redacts_aws_and_keeps_context() {
        let input = format!("error user=alice key={AWS} ok");
        let result = scrub(&input, &ScrubConfig::default()).unwrap();
        assert!(!result.text.contains(AWS));
        assert!(result.text.contains("user=alice"));
        assert!(result.text.contains("ok"));
        assert!(result
            .findings
            .iter()
            .any(|f| f.detector_type == "AWS_ACCESS_KEY"));
        assert_eq!(result.safety_status, SafetyStatus::SafeCopyReady);
    }

    #[test]
    fn repeated_value_correlated() {
        let input = format!("a={AWS} mid b={AWS}");
        let result = scrub(
            &input,
            &ScrubConfig {
                session_seed: 42,
                ..Default::default()
            },
        )
        .unwrap();
        let f = result
            .findings
            .iter()
            .find(|f| f.detector_type == "AWS_ACCESS_KEY")
            .unwrap();
        assert_eq!(f.occurrences, 2);
        let ph = &f.placeholder;
        assert_eq!(result.text.matches(ph.as_str()).count(), 2);
    }

    #[test]
    fn empty_input_errors() {
        assert!(matches!(
            scrub("", &ScrubConfig::default()),
            Err(ScrubError::EmptyInput)
        ));
    }

    #[test]
    fn json_path_structure_valid() {
        let input = r#"{"token":"AKIAIOSFODNN7EXAMPLE"}"#;
        let result = scrub_with_path(
            input,
            Some(Path::new("cfg.json")),
            &ScrubConfig::default(),
        )
        .unwrap();
        assert_eq!(result.structure_status, StructureStatus::Valid);
        assert!(serde_json::from_str::<serde_json::Value>(&result.text).is_ok());
    }
}
