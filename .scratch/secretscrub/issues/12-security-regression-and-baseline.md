# 12 — Security regression suite and detector baseline

Status: needs-triage  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)  
Related: `.scratch/secretscrub/TRUST.md`, `.scratch/secretscrub/ODDS-IMPROVEMENT.md`

## What to build

Lock private-beta trust with automated **security regression** tests and an internal **detector baseline** on golden fixtures (precision/recall on fixtures only — not marketing claims). Cover path traversal, symlink handling, destination conflicts, denied Tauri capabilities (when desktop exists), CSP assumptions, and diagnostic-content exclusion (no secrets in logs).

End-to-end: `cargo test` (and desktop e2e where applicable) fails if a known unsafe behavior regresses; `docs` or fixture README states baseline how-to for maintainers.

## Acceptance criteria

- [ ] Regression tests: path traversal, symlink escape, destination overwrite refusal, export atomicity under failure
- [ ] Logging/diagnostics never contain secret values or raw artifact snippets in failure paths under test
- [ ] Desktop: capability deny cases and CSP “no remote script” smoke checks (if app present)
- [ ] Golden fixture suite runnable in one command; baseline metrics documented for maintainers only
- [ ] FN report → fixture workflow documented (link TRUST.md)
- [ ] No claim of universal detection in any user-facing string added by this work

## User stories covered

17 (partial — clear/workspace retention tested if implemented), 20, 35, 37 (partial — process readiness), 38 (partial — diagnostic exclusion)

## Blocked by

- `05-atomic-export-safety-summary`
- `06-folder-workspace-bundle`
- `08-streaming-cancel-limits`
- `09-desktop-scan-export-mvp` (for desktop-specific checks; core/CLI security tests can land earlier)

## Comments

-
