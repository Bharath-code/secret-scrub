//! Black-box CLI tests: single-file, structure, workspace, exit codes.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

const AWS: &str = "AKIAIOSFODNN7EXAMPLE";

#[test]
fn stdin_redacts_to_stdout() {
    cargo_bin_cmd!("secretscrub")
        .args(["scrub", "--session-seed", "0"])
        .write_stdin(format!("before {AWS} after\n"))
        .assert()
        .code(0)
        .stdout(predicate::str::contains(AWS).not())
        .stdout(predicate::str::contains("before"))
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
        .code(0);

    assert!(fs::read_to_string(&src).unwrap().contains(AWS));
    assert!(!fs::read_to_string(&dest).unwrap().contains(AWS));
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
        .code(1)
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
        .code(0);

    let body = fs::read_to_string(&summary).unwrap();
    assert!(!body.contains(AWS));
    assert!(body.contains("rule_pack_version"));
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(v.get("findings").is_some());
}

#[test]
fn format_json_stdout_contract() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("in.log");
    fs::write(&src, format!("{AWS}\n")).unwrap();

    let assert = cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            src.to_str().unwrap(),
            "--format",
            "json",
            "--session-seed",
            "0",
        ])
        .assert()
        .code(0);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(!stdout.contains(AWS));
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(v.get("safety_status").is_some());
    assert!(v.get("findings").is_some());
    assert!(v.get("product_version").is_some());
}

#[test]
fn review_required_exit_code_for_bad_json() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("bad.json");
    fs::write(&src, format!("{{ not json {AWS}\n")).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            src.to_str().unwrap(),
            "--format",
            "json",
            "--session-seed",
            "0",
        ])
        .assert()
        .code(2);
}

#[test]
fn unsupported_toml_exit_code() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("x.toml");
    fs::write(&src, "k = \"v\"\n").unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            src.to_str().unwrap(),
            "--format",
            "json",
            "--session-seed",
            "0",
        ])
        .assert()
        .code(3);
}

#[test]
fn folder_workspace_correlated_export() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.log"), format!("a={AWS}\n")).unwrap();
    fs::write(dir.path().join("b.log"), format!("b={AWS}\n")).unwrap();
    let out = dir.path().join("safe");

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            dir.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--session-seed",
            "0",
            "--format",
            "json",
        ])
        .assert()
        .code(0);

    let a = fs::read_to_string(out.join("a.log")).unwrap();
    let b = fs::read_to_string(out.join("b.log")).unwrap();
    assert!(!a.contains(AWS));
    assert!(!b.contains(AWS));
    // Same placeholder token in both files (workspace correlation).
    let start = a.find("[AWS_ACCESS_KEY#").expect("placeholder in a");
    let end = a[start..].find(']').expect("close bracket") + start + 1;
    let ph = &a[start..end];
    assert!(b.contains(ph), "expected {ph} in {b}");
}

#[test]
fn empty_input_failure_exit() {
    cargo_bin_cmd!("secretscrub")
        .args(["scrub", "--session-seed", "0"])
        .write_stdin("")
        .assert()
        .code(1);
}

#[test]
fn oversize_file_fails() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("big.log");
    fs::write(&src, "x".repeat(200)).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            src.to_str().unwrap(),
            "--max-file-size",
            "50",
            "--session-seed",
            "0",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("max_file_size"));
}
