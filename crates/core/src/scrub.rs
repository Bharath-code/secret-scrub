//! Scrub API: content + config → transformed text + findings + status.

use crate::detect::find_candidates;
use crate::placeholder::PlaceholderAllocator;
use crate::rulepack::RulePack;
use crate::types::{Finding, SafetyStatus, StructureStatus};
use std::collections::HashMap;
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
    /// Seeds placeholder index permutation. Use a fixed value in tests;
    /// production CLI should use a random seed per invocation.
    pub session_seed: u64,
}

impl Default for ScrubConfig {
    fn default() -> Self {
        Self {
            rule_pack: RulePack::BuiltinV1,
            session_seed: 0,
        }
    }
}

/// Result of scrubbing a single artifact (plain text path for private-beta spine).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrubResult {
    pub text: String,
    pub findings: Vec<Finding>,
    pub safety_status: SafetyStatus,
    pub structure_status: StructureStatus,
    pub rule_pack_version: String,
}

/// Redact sensitive values in `content` using the built-in rule pack.
///
/// Plain text: structure is `NotApplicable`. Safety is `SafeCopyReady` when the
/// scan completes (findings may be empty). Review-required is reserved for later
/// unsupported/partial paths.
pub fn scrub(content: &str, config: &ScrubConfig) -> Result<ScrubResult, ScrubError> {
    if content.is_empty() {
        return Err(ScrubError::EmptyInput);
    }

    let candidates = find_candidates(content);

    // Pass 1: discover unique values in first-seen order and count occurrences.
    let mut allocator = PlaceholderAllocator::new(config.session_seed);
    let mut counts: HashMap<(String, String), usize> = HashMap::new();
    for c in &candidates {
        // Allocate (or look up) without emitting replacements yet.
        let _ = allocator.placeholder_for(c.detector_type, &c.value);
        *counts
            .entry((c.detector_type.to_string(), c.value.clone()))
            .or_insert(0) += 1;
    }

    // Pass 2: build replacements using final indices (stable after full discovery).
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    for c in &candidates {
        let ph = allocator
            .assigned_placeholder(c.detector_type, &c.value)
            .expect("placeholder assigned in pass 1");
        replacements.push((c.start, c.end, ph));
    }

    let text = apply_replacements(content, &replacements);

    let mut findings: Vec<Finding> = counts
        .into_iter()
        .map(|((detector_type, value), occurrences)| {
            let placeholder = allocator
                .assigned_placeholder(&detector_type, &value)
                .expect("placeholder assigned during scrub");
            Finding {
                detector_type,
                placeholder,
                occurrences,
            }
        })
        .collect();

    findings.sort_by(|a, b| {
        a.detector_type
            .cmp(&b.detector_type)
            .then_with(|| a.placeholder.cmp(&b.placeholder))
    });

    Ok(ScrubResult {
        text,
        findings,
        safety_status: SafetyStatus::SafeCopyReady,
        structure_status: StructureStatus::NotApplicable,
        rule_pack_version: config.rule_pack.version().to_string(),
    })
}

/// Apply non-overlapping replacements from end to start so offsets stay valid.
fn apply_replacements(content: &str, replacements: &[(usize, usize, String)]) -> String {
    let mut ordered = replacements.to_vec();
    ordered.sort_by_key(|b| std::cmp::Reverse(b.0));
    let mut out = content.to_string();
    for (start, end, ph) in ordered {
        out.replace_range(start..end, &ph);
    }
    out
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
}
