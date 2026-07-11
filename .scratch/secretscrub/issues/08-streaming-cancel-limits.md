# 08 — Streaming scan, cancellation, and hostile-input limits

Status: needs-triage  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Make the engine **streaming-capable** for large text/logs so full files need not sit entirely in memory when avoidable; support **cancellation** that leaves originals untouched and avoids partial final exports; enforce depth/size/time/memory limits for adversarial or oversized inputs with clear review-required or failure outcomes.

End-to-end: large fixture + cancel mid-scan → no destination safe file; oversized line/file → bounded failure with actionable message.

## Acceptance criteria

- [ ] Large-input path processes in chunks/streams with documented memory bounds approach
- [ ] Cancel API/CLI signal stops work; original input unchanged; final export path not left half-written
- [ ] Limits: max file size, max line length, max recursion, timeout or work budget — all tested
- [ ] Property or stress test: cancellation at arbitrary points does not corrupt export semantics
- [ ] UI-facing progress hooks exist at the core/workspace boundary (even if only CLI prints progress for now)
- [ ] Desktop remains unblocked later: progress events are structured, not println-only forever

## User stories covered

21, 22, 36

## Blocked by

- `01-scaffold-core-cli-first-redact`
- `05-atomic-export-safety-summary`
- `06-folder-workspace-bundle` (for multi-file cancel)

## Comments

-
