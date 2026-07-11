//! Structure-preserving and workspace integration tests.

use secretscrub_core::{
    export_workspace_tree, scrub_with_path, scrub_workspace, CancelFlag, RulePack, SafetyStatus,
    ScrubConfig, StructureStatus, WorkspaceLimits,
};
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> (PathBuf, String) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../fixtures");
    path.push(name);
    let s = fs::read_to_string(&path).unwrap();
    (path, s)
}

#[test]
fn json_roundtrip_parseable() {
    let (path, input) = fixture("sample.json");
    let out = scrub_with_path(&input, Some(&path), &ScrubConfig::default()).unwrap();
    assert_eq!(out.structure_status, StructureStatus::Valid);
    assert_eq!(out.safety_status, SafetyStatus::SafeCopyReady);
    let v: serde_json::Value = serde_json::from_str(&out.text).unwrap();
    assert_eq!(v["count"], 3);
    assert_eq!(v["service"], "payments");
    assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn yaml_roundtrip_parseable() {
    let (path, input) = fixture("sample.yaml");
    let out = scrub_with_path(&input, Some(&path), &ScrubConfig::default()).unwrap();
    assert_eq!(out.structure_status, StructureStatus::Valid);
    let v: serde_yaml::Value = serde_yaml::from_str(&out.text).unwrap();
    assert_eq!(v["meta"]["region"], "us-east-1");
    assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn env_preserves_structure() {
    let (path, input) = fixture("sample.env");
    let out = scrub_with_path(&input, Some(&path), &ScrubConfig::default()).unwrap();
    assert_eq!(out.structure_status, StructureStatus::Valid);
    assert!(out.text.contains("AWS_ACCESS_KEY_ID="));
    assert!(out.text.contains("DEBUG=true"));
    assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn bad_json_review_required() {
    let (path, input) = fixture("bad.json");
    let out = scrub_with_path(&input, Some(&path), &ScrubConfig::default()).unwrap();
    assert_eq!(out.structure_status, StructureStatus::Invalid);
    assert_eq!(out.safety_status, SafetyStatus::ReviewRequired);
    assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn toml_unsupported() {
    let out = scrub_with_path(
        "key = \"AKIAIOSFODNN7EXAMPLE\"\n",
        Some(std::path::Path::new("x.toml")),
        &ScrubConfig::default(),
    )
    .unwrap();
    assert_eq!(out.structure_status, StructureStatus::Unsupported);
    assert_eq!(out.safety_status, SafetyStatus::ReviewRequired);
}

#[test]
fn workspace_export_tree() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.log"), "x=AKIAIOSFODNN7EXAMPLE\n").unwrap();
    fs::write(dir.path().join("b.json"), r#"{"k":"AKIAIOSFODNN7EXAMPLE"}"#).unwrap();
    fs::write(dir.path().join("blob.bin"), [0u8, 1, 2, 0, 9]).unwrap();

    let cancel = CancelFlag::new();
    let result = scrub_workspace(
        dir.path(),
        0,
        RulePack::BuiltinV1,
        &WorkspaceLimits::for_tests(),
        &cancel,
        None,
    )
    .unwrap();

    assert_eq!(result.safety_status, SafetyStatus::ReviewRequired);
    assert!(result
        .files
        .iter()
        .any(|f| f.outcome.path.contains("blob")
            && matches!(
                f.outcome.inclusion,
                secretscrub_core::FileInclusion::Unsupported
            )));

    let out = tempfile::tempdir().unwrap();
    // tempdir already exists; force replaces it with the safe tree.
    export_workspace_tree(&result, out.path(), true, &cancel).unwrap();
    assert!(out.path().join("a.log").exists());
    assert!(out.path().join("b.json").exists());
    assert!(!out.path().join("blob.bin").exists());

    let a = fs::read_to_string(out.path().join("a.log")).unwrap();
    let b = fs::read_to_string(out.path().join("b.json")).unwrap();
    let ph = &result.findings[0].placeholder;
    assert!(a.contains(ph.as_str()));
    assert!(b.contains(ph.as_str()));
}
