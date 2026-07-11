# 01 — Scaffold core + CLI first redaction path

Status: ready-for-agent  
Type: AFK  
Done: 2026-07-11 (build-loop)  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Create the Rust workspace with a shared **core domain library** and a **CLI adapter**. Deliver one complete path: read plain text from stdin (or a single file path), run a single high-precision detector (e.g. AWS access-key-shaped token), emit redacted text on stdout, and emit a minimal findings summary. No network. No desktop.

This is the first demoable product spine: `echo '…' | secretscrub` (or `cargo run -p secretscrub-cli`) produces a safe copy.

## Acceptance criteria

- [x] Cargo workspace exists with at least `crates/core` (library) and `crates/cli` (binary); `cargo test` and `cargo build` succeed from repo root
- [x] Core exposes a stable function/API: content + rule config in → transformed text, typed findings, export-safety status out
- [x] CLI accepts stdin and/or a single file path; writes redacted text to stdout (or `--output`) without modifying the source file
- [x] At least one golden fixture: synthetic AWS-shaped key is redacted; surrounding non-sensitive text is unchanged
- [x] Tests assert observable output and findings, not internal matcher structure
- [x] README documents how to run the CLI and that processing is local-only

## User stories covered

1, 2, 4 (partial — single detector), 15 (partial — CLI/local), 16, 23 (partial)

## Blocked by

None — can start immediately

## Comments

-
