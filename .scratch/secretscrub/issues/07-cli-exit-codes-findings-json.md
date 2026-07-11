# 07 — CLI exit codes and machine-readable findings

Status: ready-for-agent
Done: 2026-07-11 (build-loop)  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Freeze the **CLI public contract** for automation: stable exit statuses and a JSON findings summary mode that distinguishes clean completion, review-required completion, unsupported input, and execution failure. Suitable for scripted support workflows without a desktop.

End-to-end: black-box tests assert exit codes and JSON schema for success, review-required, unsupported, and hard failure cases.

## Acceptance criteria

- [x] Documented exit code table (e.g. 0 clean, 2 review-required, 3 unsupported, 1 failure — exact mapping recorded in CLI help and README)
- [x] `--format json` (or equivalent) emits findings summary without secret values
- [x] JSON includes: findings list (type, placeholder, counts), per-file status if multi-file, safety status, versions
- [x] Stdin mode and path mode both covered
- [x] Black-box CLI tests are the source of truth for the contract
- [x] Free-CLI size/limit messaging can hook here if commercial limits apply later (document extension point only)

## User stories covered

23, 24, 28 (partial — free CLI usefulness)

## Blocked by

- `01-scaffold-core-cli-first-redact`
- `05-atomic-export-safety-summary`
- `06-folder-workspace-bundle` (for multi-file fields; can ship single-file contract first then extend)

## Comments

-
