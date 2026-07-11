//! Structure-preserving transforms for JSON, YAML, and env files.

use crate::detect::find_candidates;
use crate::format::ContentFormat;
use crate::placeholder::PlaceholderAllocator;
use crate::types::{Finding, SafetyStatus, StructureStatus};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Outcome of transforming one artifact with structure awareness.
#[derive(Debug, Clone)]
pub struct StructuredScrub {
    pub text: String,
    pub counts: HashMap<(String, String), usize>,
    pub structure_status: StructureStatus,
    pub safety_status: SafetyStatus,
    /// Human reason when review is required or structure invalid.
    pub note: Option<String>,
}

/// Redact plain text using the shared allocator; return text + occurrence counts.
pub fn redact_plain(
    content: &str,
    allocator: &mut PlaceholderAllocator,
) -> (String, HashMap<(String, String), usize>) {
    let candidates = find_candidates(content);
    let mut counts: HashMap<(String, String), usize> = HashMap::new();

    for c in &candidates {
        let _ = allocator.placeholder_for(c.detector_type, &c.value);
        *counts
            .entry((c.detector_type.to_string(), c.value.clone()))
            .or_insert(0) += 1;
    }

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    for c in &candidates {
        let ph = allocator
            .assigned_placeholder(c.detector_type, &c.value)
            .expect("placeholder assigned");
        replacements.push((c.start, c.end, ph));
    }

    (apply_replacements(content, &replacements), counts)
}

fn apply_replacements(content: &str, replacements: &[(usize, usize, String)]) -> String {
    let mut ordered = replacements.to_vec();
    ordered.sort_by_key(|b| std::cmp::Reverse(b.0));
    let mut out = content.to_string();
    for (start, end, ph) in ordered {
        out.replace_range(start..end, &ph);
    }
    out
}

/// Structure-aware scrub for a single content blob.
pub fn scrub_structured(
    content: &str,
    format: ContentFormat,
    allocator: &mut PlaceholderAllocator,
) -> StructuredScrub {
    match format {
        ContentFormat::PlainText => {
            let (text, counts) = redact_plain(content, allocator);
            StructuredScrub {
                text,
                counts,
                structure_status: StructureStatus::NotApplicable,
                safety_status: SafetyStatus::SafeCopyReady,
                note: None,
            }
        }
        ContentFormat::Json => scrub_json(content, allocator),
        ContentFormat::Yaml => scrub_yaml(content, allocator),
        ContentFormat::Env => scrub_env(content, allocator),
        ContentFormat::TomlUnsupported => StructuredScrub {
            // Never emit original content for unsupported formats.
            text: String::new(),
            counts: HashMap::new(),
            structure_status: StructureStatus::Unsupported,
            safety_status: SafetyStatus::ReviewRequired,
            note: Some(
                "TOML is unsupported in private beta; not marked safe for share".into(),
            ),
        },
        ContentFormat::BinaryUnsupported => StructuredScrub {
            text: String::new(),
            counts: HashMap::new(),
            structure_status: StructureStatus::Unsupported,
            safety_status: SafetyStatus::ReviewRequired,
            note: Some("binary or non-text content is unsupported".into()),
        },
    }
}

fn scrub_json(content: &str, allocator: &mut PlaceholderAllocator) -> StructuredScrub {
    match serde_json::from_str::<JsonValue>(content) {
        Ok(mut value) => {
            let mut counts = HashMap::new();
            redact_json_value(&mut value, allocator, &mut counts);
            match serde_json::to_string_pretty(&value) {
                Ok(text) => StructuredScrub {
                    text: format!("{text}\n"),
                    counts,
                    structure_status: StructureStatus::Valid,
                    safety_status: SafetyStatus::SafeCopyReady,
                    note: None,
                },
                Err(e) => {
                    let (text, counts) = redact_plain(content, allocator);
                    StructuredScrub {
                        text,
                        counts,
                        structure_status: StructureStatus::Invalid,
                        safety_status: SafetyStatus::ReviewRequired,
                        note: Some(format!("JSON re-serialize failed: {e}")),
                    }
                }
            }
        }
        Err(e) => {
            let (text, counts) = redact_plain(content, allocator);
            StructuredScrub {
                text,
                counts,
                structure_status: StructureStatus::Invalid,
                safety_status: SafetyStatus::ReviewRequired,
                note: Some(format!("malformed JSON: {e}")),
            }
        }
    }
}

fn redact_json_value(
    value: &mut JsonValue,
    allocator: &mut PlaceholderAllocator,
    counts: &mut HashMap<(String, String), usize>,
) {
    match value {
        JsonValue::String(s) => {
            let (new_s, c) = redact_plain(s, allocator);
            merge_counts(counts, c);
            *s = new_s;
        }
        JsonValue::Array(items) => {
            for item in items {
                redact_json_value(item, allocator, counts);
            }
        }
        JsonValue::Object(map) => {
            for (_k, v) in map.iter_mut() {
                redact_json_value(v, allocator, counts);
            }
        }
        _ => {}
    }
}

fn scrub_yaml(content: &str, allocator: &mut PlaceholderAllocator) -> StructuredScrub {
    match serde_yaml::from_str::<serde_yaml::Value>(content) {
        Ok(mut value) => {
            let mut counts = HashMap::new();
            redact_yaml_value(&mut value, allocator, &mut counts);
            match serde_yaml::to_string(&value) {
                Ok(text) => StructuredScrub {
                    text,
                    counts,
                    structure_status: StructureStatus::Valid,
                    safety_status: SafetyStatus::SafeCopyReady,
                    note: None,
                },
                Err(e) => {
                    let (text, counts) = redact_plain(content, allocator);
                    StructuredScrub {
                        text,
                        counts,
                        structure_status: StructureStatus::Invalid,
                        safety_status: SafetyStatus::ReviewRequired,
                        note: Some(format!("YAML re-serialize failed: {e}")),
                    }
                }
            }
        }
        Err(e) => {
            let (text, counts) = redact_plain(content, allocator);
            StructuredScrub {
                text,
                counts,
                structure_status: StructureStatus::Invalid,
                safety_status: SafetyStatus::ReviewRequired,
                note: Some(format!("malformed YAML: {e}")),
            }
        }
    }
}

fn redact_yaml_value(
    value: &mut serde_yaml::Value,
    allocator: &mut PlaceholderAllocator,
    counts: &mut HashMap<(String, String), usize>,
) {
    match value {
        serde_yaml::Value::String(s) => {
            let (new_s, c) = redact_plain(s, allocator);
            merge_counts(counts, c);
            *s = new_s;
        }
        serde_yaml::Value::Sequence(items) => {
            for item in items {
                redact_yaml_value(item, allocator, counts);
            }
        }
        serde_yaml::Value::Mapping(map) => {
            // Redact values only — keep keys for debugging structure.
            let keys: Vec<serde_yaml::Value> = map.keys().cloned().collect();
            for k in keys {
                if let Some(v) = map.get_mut(k) {
                    redact_yaml_value(v, allocator, counts);
                }
            }
        }
        _ => {}
    }
}

/// Env-style files: redact values after `=` / `:`, preserve keys and comments.
fn scrub_env(content: &str, allocator: &mut PlaceholderAllocator) -> StructuredScrub {
    let mut counts = HashMap::new();
    let mut out_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            out_lines.push(line.to_string());
            continue;
        }
        if let Some(eq) = line.find('=') {
            let (key, rest) = line.split_at(eq);
            let value = &rest[1..]; // skip '='
            let (new_val, c) = redact_plain(value, allocator);
            merge_counts(&mut counts, c);
            out_lines.push(format!("{key}={new_val}"));
        } else {
            let (new_line, c) = redact_plain(line, allocator);
            merge_counts(&mut counts, c);
            out_lines.push(new_line);
        }
    }

    // Preserve trailing newline if original had one.
    let mut text = out_lines.join("\n");
    if content.ends_with('\n') {
        text.push('\n');
    }

    StructuredScrub {
        text,
        counts,
        structure_status: StructureStatus::Valid,
        safety_status: SafetyStatus::SafeCopyReady,
        note: None,
    }
}

fn merge_counts(
    into: &mut HashMap<(String, String), usize>,
    from: HashMap<(String, String), usize>,
) {
    for (k, v) in from {
        *into.entry(k).or_insert(0) += v;
    }
}

/// Build sorted findings from count map + allocator.
pub fn findings_from_counts(
    counts: HashMap<(String, String), usize>,
    allocator: &PlaceholderAllocator,
) -> Vec<Finding> {
    let mut findings: Vec<Finding> = counts
        .into_iter()
        .map(|((detector_type, value), occurrences)| {
            let placeholder = allocator
                .assigned_placeholder(&detector_type, &value)
                .unwrap_or_else(|| format!("[{detector_type}#?]"));
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
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::placeholder::PlaceholderAllocator;

    #[test]
    fn json_stays_parseable() {
        let mut a = PlaceholderAllocator::new(0);
        let input = r#"{"key":"AKIAIOSFODNN7EXAMPLE","n":1}"#;
        let out = scrub_structured(input, ContentFormat::Json, &mut a);
        assert_eq!(out.structure_status, StructureStatus::Valid);
        let v: JsonValue = serde_json::from_str(&out.text).unwrap();
        assert!(v["key"].as_str().unwrap().contains("AWS_ACCESS_KEY"));
        assert_eq!(v["n"], 1);
    }

    #[test]
    fn malformed_json_review_required() {
        let mut a = PlaceholderAllocator::new(0);
        let out = scrub_structured("{not json", ContentFormat::Json, &mut a);
        assert_eq!(out.structure_status, StructureStatus::Invalid);
        assert_eq!(out.safety_status, SafetyStatus::ReviewRequired);
    }

    #[test]
    fn env_preserves_keys() {
        let mut a = PlaceholderAllocator::new(0);
        let input = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\n# comment\n";
        let out = scrub_structured(input, ContentFormat::Env, &mut a);
        assert!(out.text.starts_with("AWS_ACCESS_KEY_ID="));
        assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(out.text.contains("# comment"));
    }
}
