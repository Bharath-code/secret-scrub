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
    let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        ["service", "aws_key", "nested", "count"],
        "scrubbed JSON must preserve original key order"
    );
}

#[test]
fn yaml_roundtrip_parseable() {
    let (path, input) = fixture("sample.yaml");
    let out = scrub_with_path(&input, Some(&path), &ScrubConfig::default()).unwrap();
    assert_eq!(out.structure_status, StructureStatus::Valid);
    let v: serde_norway::Value = serde_norway::from_str(&out.text).unwrap();
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
fn cross_file_correlation_stable_across_runs() {
    const KEY1: &str = "AKIAIOSFODNN7EXAMPLE";
    const KEY2: &str = "AKIAZZZZZZZZZZZZZZZZ";

    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.log"), format!("k1={KEY1}\nk2={KEY2}\n")).unwrap();
    fs::write(dir.path().join("b.log"), format!("again={KEY1}\n")).unwrap();

    let cancel = CancelFlag::new();
    let run = || {
        scrub_workspace(
            dir.path(),
            RulePack::BuiltinV1,
            &WorkspaceLimits::for_tests(),
            &cancel,
            None,
        )
        .unwrap()
    };

    let text_of = |result: &secretscrub_core::WorkspaceResult, name: &str| -> String {
        result
            .files
            .iter()
            .find(|f| f.outcome.path.contains(name))
            .and_then(|f| f.text.clone())
            .unwrap_or_else(|| panic!("scrubbed text for {name}"))
    };

    let first = run();
    let (a0, b0) = (text_of(&first, "a.log"), text_of(&first, "b.log"));

    let ph = |text: &str, prefix: &str| -> String {
        let start = text.find('[').unwrap_or_else(|| panic!("no placeholder after {prefix}"));
        let end = text[start..].find(']').unwrap() + start;
        text[start..=end].to_string()
    };
    let ph1 = ph(a0.split("k2=").next().unwrap(), "k1");
    let ph2 = ph(a0.split("k2=").nth(1).unwrap(), "k2");
    let ph1_b = ph(&b0, "again");

    assert_eq!(ph1, ph1_b, "same value must map to same placeholder in both files");
    assert_ne!(ph1, ph2, "distinct values must get distinct placeholders");

    // Placeholder indices are first-seen sequential, so identical input
    // reproduces identical output across independent runs.
    for run_n in 1..=5 {
        let r = run();
        assert_eq!(text_of(&r, "a.log"), a0, "run {run_n} changed a.log output");
        assert_eq!(text_of(&r, "b.log"), b0, "run {run_n} changed b.log output");
    }
}

#[test]
fn cancelled_force_export_preserves_existing_destination() {
    let src = tempfile::tempdir().unwrap();
    fs::write(src.path().join("a.log"), "x=AKIAIOSFODNN7EXAMPLE\n").unwrap();

    let cancel = CancelFlag::new();
    let result = scrub_workspace(
        src.path(),
        RulePack::BuiltinV1,
        &WorkspaceLimits::for_tests(),
        &cancel,
        None,
    )
    .unwrap();

    let out = tempfile::tempdir().unwrap();
    fs::write(out.path().join("precious.txt"), "keep me").unwrap();

    cancel.cancel();
    let err = export_workspace_tree(&result, out.path(), true, &cancel);
    assert!(err.is_err(), "cancelled export must fail");
    assert_eq!(
        fs::read_to_string(out.path().join("precious.txt")).unwrap(),
        "keep me",
        "cancelled --force export must not touch the existing destination"
    );
    // No staging leftovers next to the destination.
    let parent = out.path().parent().unwrap();
    assert!(!fs::read_dir(parent).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".secretscrub-export-")
    }));
}

#[test]
fn workspace_export_tree() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.log"), "x=AKIAIOSFODNN7EXAMPLE\n").unwrap();
    fs::write(dir.path().join("b.json"), r#"{"k":"AKIAIOSFODNN7EXAMPLE"}"#).unwrap();
    fs::write(dir.path().join("blob.bin"), [0u8, 1, 2, 0, 9]).unwrap();
    fs::write(dir.path().join("empty.log"), "").unwrap();

    let cancel = CancelFlag::new();
    let result = scrub_workspace(
        dir.path(),
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
    // Empty files are copied through, not dropped.
    assert_eq!(fs::read_to_string(out.path().join("empty.log")).unwrap(), "");

    let a = fs::read_to_string(out.path().join("a.log")).unwrap();
    let b = fs::read_to_string(out.path().join("b.json")).unwrap();
    let ph = &result.findings[0].placeholder;
    assert!(a.contains(ph.as_str()));
    assert!(b.contains(ph.as_str()));
}

#[test]
fn force_export_leaves_no_staging_or_aside_leftovers() {
    let src = tempfile::tempdir().unwrap();
    fs::write(src.path().join("a.log"), "x=AKIAIOSFODNN7EXAMPLE\n").unwrap();
    let cancel = CancelFlag::new();
    let result = scrub_workspace(
        src.path(),
        RulePack::BuiltinV1,
        &WorkspaceLimits::for_tests(),
        &cancel,
        None,
    )
    .unwrap();

    let out = tempfile::tempdir().unwrap();
    fs::write(out.path().join("old.txt"), "previous export").unwrap();

    export_workspace_tree(&result, out.path(), true, &cancel).unwrap();

    assert!(out.path().join("a.log").exists());
    assert!(!out.path().join("old.txt").exists());
    let parent = out.path().parent().unwrap();
    assert!(!fs::read_dir(parent).unwrap().any(|e| {
        let name = e.unwrap().file_name().to_string_lossy().into_owned();
        name.starts_with(".secretscrub-export-")
    }));
}

#[cfg(target_os = "macos")]
#[test]
fn force_export_swap_failure_restores_destination_and_cleans_staging() {
    // ponytail: uses `chflags uchg` (macOS/BSD-only) to make the rename of
    // an existing, otherwise-writable destination fail deterministically,
    // so this test doesn't run on Linux CI — the logic itself is
    // platform-independent, only the failure trigger isn't.
    use std::process::Command;

    let src = tempfile::tempdir().unwrap();
    fs::write(src.path().join("a.log"), "x=AKIAIOSFODNN7EXAMPLE\n").unwrap();
    let cancel = CancelFlag::new();
    let result = scrub_workspace(
        src.path(),
        RulePack::BuiltinV1,
        &WorkspaceLimits::for_tests(),
        &cancel,
        None,
    )
    .unwrap();

    let parent = tempfile::tempdir().unwrap();
    let dest = parent.path().join("existing");
    fs::create_dir_all(&dest).unwrap();
    fs::write(dest.join("precious.txt"), "keep me").unwrap();

    // Immutable flag on the destination itself blocks renaming it aside,
    // even though the parent directory stays writable (so staging setup
    // still succeeds and the failure hits the intended rename step).
    assert!(Command::new("chflags")
        .args(["uchg", dest.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let err = export_workspace_tree(&result, &dest, true, &cancel);

    Command::new("chflags")
        .args(["nouchg", dest.to_str().unwrap()])
        .status()
        .unwrap();

    assert!(err.is_err(), "export over an immutable destination must fail");
    assert_eq!(
        fs::read_to_string(dest.join("precious.txt")).unwrap(),
        "keep me",
        "a failed swap must never lose the pre-existing destination"
    );
    assert!(!fs::read_dir(parent.path()).unwrap().any(|e| {
        let name = e.unwrap().file_name().to_string_lossy().into_owned();
        name.starts_with(".secretscrub-export-")
    }));
}
