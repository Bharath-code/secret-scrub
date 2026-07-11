# 11 — Recipient profiles (AI / vendor / incident)

Status: needs-triage  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Add named **recipient profiles** that adjust redaction scope without forcing a rule language: at least **AI prompt**, **vendor support**, and **incident bundle**. Profile choice is visible in review header and reflected in findings/export summary. Nontechnical users can complete the flow without custom rules.

End-to-end: same fixture under AI vs incident profile produces documented, testable differences (e.g. broader PII in AI profile) without breaking structure preservation.

## Acceptance criteria

- [ ] Three built-in profiles with human-readable protection scope descriptions
- [ ] Profile selected before or at scan; shown on review and in safety summary
- [ ] Core accepts profile as part of output policy / rule config — UI and CLI both can set it
- [ ] Fixture tests lock expected behavior differences per profile (avoid vague “feels stricter”)
- [ ] Default profile is safe for the most common AI-share case or explicitly documented
- [ ] Custom rules remain out of scope for this issue (Pro later) unless trivial stub

## User stories covered

14, 19 (partial — summary includes profile)

## Blocked by

- `03-builtin-detector-pack`
- `07-cli-exit-codes-findings-json`
- `09-desktop-scan-export-mvp` (desktop selector); CLI flag can ship even if desktop lags

## Comments

-
