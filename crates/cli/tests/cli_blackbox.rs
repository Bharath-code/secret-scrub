//! Black-box CLI tests for single-file safe-share spine.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

const AWS: &str = "AKIAIOSFODNN7EXAMPLE";

#[test]
fn stdin_redacts_to_stdout() {
    let mut cmd = cargo_bin_cmd!("secretscrub");
    cmd.args(["scrub", "--session-seed", "0"])
        .write_stdin(format!("before {AWS} after\n"))
        .assert()
        .success()
        .stdout(predicate::str::contains(AWS).not())
        .stdout(predicate::str::contains("before"))
        .stdout(predicate::str::contains("after"))
        .stdout(predicate::str::contains("[AWS_ACCESS_KEY#"));
}

#[test]
fn file_to_output_leaves_source_untouched() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("app.log");
    let dest = dir.path().join("app.safe.log");
    fs::write(&src, format!("key={AWS}\n")).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            src.to_str().unwrap(),
            "-o",
            dest.to_str().unwrap(),
            "--session-seed",
            "0",
        ])
        .assert()
        .success();

    let original = fs::read_to_string(&src).unwrap();
    assert!(original.contains(AWS), "source must be immutable");
    let safe = fs::read_to_string(&dest).unwrap();
    assert!(!safe.contains(AWS));
    assert!(safe.contains("[AWS_ACCESS_KEY#"));
}

#[test]
fn refuses_existing_destination_without_force() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("in.log");
    let dest = dir.path().join("out.log");
    fs::write(&src, format!("{AWS}\n")).unwrap();
    fs::write(&dest, "existing\n").unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            src.to_str().unwrap(),
            "-o",
            dest.to_str().unwrap(),
            "--session-seed",
            "0",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    assert_eq!(fs::read_to_string(&dest).unwrap(), "existing\n");
}

#[test]
fn summary_json_shape_no_secrets() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("in.log");
    let dest = dir.path().join("out.log");
    let summary = dir.path().join("summary.json");
    fs::write(&src, format!("{AWS}\n")).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            src.to_str().unwrap(),
            "-o",
            dest.to_str().unwrap(),
            "--summary",
            summary.to_str().unwrap(),
            "--session-seed",
            "0",
        ])
        .assert()
        .success();

    let body = fs::read_to_string(&summary).unwrap();
    assert!(!body.contains(AWS));
    assert!(body.contains("rule_pack_version"));
    assert!(body.contains("replacement_counts"));
    assert!(body.contains("AWS_ACCESS_KEY"));
    assert!(body.contains("disclaimer"));
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(v.get("product_version").is_some());
    assert!(v.get("findings").is_some());
}
