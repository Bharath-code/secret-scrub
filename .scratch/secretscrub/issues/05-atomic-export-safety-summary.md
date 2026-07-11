# 05 — Atomic export and safety summary

Status: ready-for-agent  
Type: AFK  
Done: 2026-07-11 (build-loop)  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Deliver the **export service** path from the CLI: write a new safe copy to a user-chosen destination using atomic completion semantics; never overwrite the original input; on conflict require explicit resolution; optionally write a plain-language / machine-readable **safety summary** (counts by type, structure validation, rule-pack version — never secret values).

End-to-end: `secretscrub scrub ./app.log -o ./app.safe.log --summary ./app.summary.json` leaves original untouched and only appears at destination when complete.

## Acceptance criteria

- [x] Export writes only transformed content to the destination; source path is never opened for write
- [x] Atomic write pattern (temp + rename or equivalent) so crash/cancel does not leave a misleading partial “safe” file at the final path
- [x] Existing destination without force/overwrite flag → clear error; no silent clobber
- [x] Safety summary includes replacement counts by type, structure status, product/rule-pack version; excludes secret values and raw snippets
- [x] Black-box CLI tests for success, conflict, and summary shape
- [x] Interruption/cancel path documented and tested at least for “destination absent or incomplete”

## User stories covered

18, 19, 22 (partial), 36 (partial)

## Blocked by

- `01-scaffold-core-cli-first-redact`
- `02-placeholders-value-correlation`

## Comments

-
