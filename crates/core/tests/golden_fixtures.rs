//! Golden fixture tests — observable redaction behavior only.
//! Synthetic secrets only; never real credentials.

use secretscrub_core::{scrub, ScrubConfig, SafetyStatus};
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../fixtures");
    path.push(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn cfg(seed: u64) -> ScrubConfig {
    ScrubConfig {
        session_seed: seed,
        ..ScrubConfig::default()
    }
}

#[test]
fn aws_log_redacts_key_keeps_context() {
    let input = fixture("aws_log.txt");
    let out = scrub(&input, &cfg(0)).unwrap();
    assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(out.text.contains("request_id=abc-123"));
    assert!(out.text.contains("[AWS_ACCESS_KEY#"));
    assert_eq!(out.safety_status, SafetyStatus::SafeCopyReady);
}

#[test]
fn repeated_aws_correlated() {
    let input = fixture("repeated_aws.txt");
    let out = scrub(&input, &cfg(7)).unwrap();
    let f = out
        .findings
        .iter()
        .find(|f| f.detector_type == "AWS_ACCESS_KEY" && f.occurrences == 2)
        .expect("repeated key finding");
    assert_eq!(out.text.matches(f.placeholder.as_str()).count(), 2);
    // Second distinct key appears once
    assert!(out
        .findings
        .iter()
        .any(|f| f.detector_type == "AWS_ACCESS_KEY" && f.occurrences == 1));
}

#[test]
fn multi_secret_fixture() {
    let input = fixture("multi_secret.txt");
    let out = scrub(&input, &cfg(1)).unwrap();
    for t in [
        "AWS_ACCESS_KEY",
        "GITHUB_TOKEN",
        "STRIPE_SECRET",
        "OPENAI_API_KEY",
        "JWT",
        "EMAIL",
        "IP_ADDRESS",
        "GENERIC_SECRET",
    ] {
        assert!(
            out.findings.iter().any(|f| f.detector_type == t),
            "missing detector type {t} in {:?}",
            out.findings
        );
    }
    assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(!out.text.contains("ghp_"));
    assert!(!out.text.contains("sk_test_"));
    assert!(!out.text.contains("sk-proj-"));
    assert!(!out.text.contains("user@example.com"));
    assert!(!out.text.contains("203.0.113.10"));
}

#[test]
fn false_positive_fixture_preserves_benign() {
    let input = fixture("false_positive.txt");
    let out = scrub(&input, &cfg(0)).unwrap();
    // Benign version strings and non-secret ids should remain
    assert!(out.text.contains("version=1.2.3-rc1"));
    assert!(out.text.contains("build_id=not-a-secret-value"));
    // Slug-like sk- names are not OpenAI keys
    assert!(out.text.contains("sk-formatting-helper-utils-v2"));
    // No AWS/GitHub shaped tokens in this fixture
    assert!(!out
        .findings
        .iter()
        .any(|f| f.detector_type == "AWS_ACCESS_KEY"));
}

#[test]
fn output_is_seed_independent() {
    let input = fixture("repeated_aws.txt");
    let a = scrub(&input, &cfg(1)).unwrap();
    let b = scrub(&input, &cfg(2)).unwrap();
    // session_seed is a no-op: indices are per-type sequential in
    // first-seen order, identical across seeds and runs.
    assert_eq!(a.text, b.text);
    assert_eq!(a.findings, b.findings);
}

#[test]
fn rule_pack_version_present() {
    let out = scrub("AKIAIOSFODNN7EXAMPLE\n", &cfg(0)).unwrap();
    assert_eq!(out.rule_pack_version, secretscrub_core::rule_pack_version());
}
