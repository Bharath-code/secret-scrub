//! Black-box CLI tests: single-file, structure, workspace, exit codes.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

const AWS: &str = "AKIAIOSFODNN7EXAMPLE";

#[test]
fn stdin_redacts_to_stdout() {
    cargo_bin_cmd!("secretscrub")
        .args(["scrub"])
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
        ])
        .assert()
        .code(0);

    let body = fs::read_to_string(&summary).unwrap();
    assert!(!body.contains(AWS));
    assert!(body.contains("rule_pack_version"));
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(v.get("findings").is_some());
    assert!(v.get("content_sha256").is_some());
    assert_eq!(
        v.get("hash_scheme").and_then(|x| x.as_str()),
        Some("sha256-single-v1")
    );
    assert!(v.get("created_at").is_some());
}

#[test]
fn verify_round_trip_single_file() {
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
        ])
        .assert()
        .code(0);

    cargo_bin_cmd!("secretscrub")
        .args([
            "verify",
            dest.to_str().unwrap(),
            "--summary",
            summary.to_str().unwrap(),
        ])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("verify ok"));
}

#[test]
fn verify_fails_on_tampered_safe_copy() {
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
        ])
        .assert()
        .code(0);

    // Tamper one byte of the safe copy.
    let mut body = fs::read_to_string(&dest).unwrap();
    body.push('x');
    fs::write(&dest, body).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "verify",
            dest.to_str().unwrap(),
            "--summary",
            summary.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("verify failed"));
}

#[test]
fn verify_round_trip_workspace() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.log"), format!("a={AWS}\n")).unwrap();
    fs::write(dir.path().join("b.log"), format!("b=clean\n")).unwrap();
    let out = dir.path().join("safe");
    let summary = dir.path().join("summary.json");

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            dir.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--summary",
            summary.to_str().unwrap(),
        ])
        .assert()
        .code(0);

    let body = fs::read_to_string(&summary).unwrap();
    assert!(!body.contains(AWS));
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        v.get("hash_scheme").and_then(|x| x.as_str()),
        Some("sha256-workspace-v1")
    );
    assert!(v.get("content_sha256").is_some());

    cargo_bin_cmd!("secretscrub")
        .args([
            "verify",
            out.to_str().unwrap(),
            "--summary",
            summary.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"ok\": true"));
}

#[test]
fn verify_fails_on_tampered_workspace_file() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.log"), format!("a={AWS}\n")).unwrap();
    let out = dir.path().join("safe");
    let summary = dir.path().join("summary.json");

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            dir.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--summary",
            summary.to_str().unwrap(),
        ])
        .assert()
        .code(0);

    let tampered = out.join("a.log");
    let mut body = fs::read_to_string(&tampered).unwrap();
    body.push_str("tamper");
    fs::write(&tampered, body).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "verify",
            out.to_str().unwrap(),
            "--summary",
            summary.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("a.log").or(predicate::str::contains("mismatch")));
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
fn sensitive_filename_forces_review_required_exit_code() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("alice@example.com.log"), "nothing sensitive here\n").unwrap();
    let out = dir.path().join("safe");

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            dir.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .assert()
        .code(2);
}

#[test]
fn unremarkable_filename_stays_clean_exit_code() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("notes.log"), "nothing sensitive here\n").unwrap();
    let out = dir.path().join("safe");

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            dir.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .assert()
        .code(0);
}

#[test]
fn binary_file_unsupported_exit_code() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("blob.log");
    // Valid UTF-8 (no NUL bytes) but mostly control characters, matching
    // the workspace path's looks_binary sniff.
    fs::write(&src, [1u8, 2, 3, 4, 5, 6, 7, 11, 12, 14, 15]).unwrap();

    let assert = cargo_bin_cmd!("secretscrub")
        .args(["scrub", src.to_str().unwrap()])
        .assert()
        .code(3);
    assert!(assert.get_output().stdout.is_empty());
}

#[test]
fn binary_stdin_unsupported_exit_code() {
    cargo_bin_cmd!("secretscrub")
        .args(["scrub"])
        .write_stdin(vec![0u8, 1, 2, 3])
        .assert()
        .code(3);
}

#[test]
fn empty_input_failure_exit() {
    cargo_bin_cmd!("secretscrub")
        .args(["scrub"])
        .write_stdin("")
        .assert()
        .code(1);
}

#[test]
fn check_clean_input_exits_zero_silently() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("clean.log");
    fs::write(&src, "hello world\nno secrets here\n").unwrap();

    let assert = cargo_bin_cmd!("secretscrub")
        .args(["scrub", "--check", src.to_str().unwrap()])
        .assert()
        .code(0);
    assert!(assert.get_output().stdout.is_empty());
    assert!(assert.get_output().stderr.is_empty());
    // Source untouched; no extra files created in the dir.
    assert_eq!(fs::read_to_string(&src).unwrap(), "hello world\nno secrets here\n");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn check_secret_bearing_exits_two_with_typed_report() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("leaky.log");
    fs::write(&src, format!("key={AWS}\nagain={AWS}\n")).unwrap();

    let assert = cargo_bin_cmd!("secretscrub")
        .args(["scrub", "--check", src.to_str().unwrap()])
        .assert()
        .code(2);

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("AWS_ACCESS_KEY"));
    assert!(stderr.contains("distinct"));
    assert!(!stderr.contains(AWS), "check report must never leak secret values");
    assert!(assert.get_output().stdout.is_empty());
    // No safe copy or other files created.
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    assert!(fs::read_to_string(&src).unwrap().contains(AWS));
}

#[test]
fn check_never_creates_output_file() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("in.log");
    fs::write(&src, format!("{AWS}\n")).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args(["scrub", "--check", src.to_str().unwrap()])
        .assert()
        .code(2);

    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn check_conflicts_with_output() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("in.log");
    let dest = dir.path().join("out.log");
    fs::write(&src, "x\n").unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            "--check",
            src.to_str().unwrap(),
            "-o",
            dest.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn check_multi_path_aggregates_exit_code() {
    let dir = tempdir().unwrap();
    let clean = dir.path().join("clean.log");
    let dirty = dir.path().join("dirty.log");
    fs::write(&clean, "ok\n").unwrap();
    fs::write(&dirty, format!("{AWS}\n")).unwrap();

    cargo_bin_cmd!("secretscrub")
        .args([
            "scrub",
            "--check",
            clean.to_str().unwrap(),
            dirty.to_str().unwrap(),
        ])
        .assert()
        .code(2);
}

#[test]
fn check_stdin_secret() {
    cargo_bin_cmd!("secretscrub")
        .args(["scrub", "--check"])
        .write_stdin(format!("{AWS}\n"))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("AWS_ACCESS_KEY"))
        .stderr(predicate::str::contains(AWS).not());
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
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("max_file_size"));
}
