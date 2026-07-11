# 04 — Structure-preserving JSON / YAML / env transform

Status: needs-triage  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Add a **structured-artifact transformer** path so supported formats remain useful after redaction: valid JSON stays parseable; YAML and environment-style files retain usable structure; secrets inside values are replaced with placeholders. Unsupported or broken structures surface **Review required** / non-safe status — never silent “safe.”

End-to-end: CLI scrub of `fixture.json` / `fixture.yaml` / `fixture.env` → safe files re-parse where claimed valid; findings still list secrets.

## Acceptance criteria

- [ ] Supported: JSON, YAML, environment files (and plain text/logs remain supported)
- [ ] After redaction, valid JSON fixtures still parse; YAML fixtures still parse when input was valid
- [ ] Keys/structure needed for debugging remain; only sensitive values are replaced
- [ ] Malformed structured input → explicit non-safe / review-required outcome (not a fake success)
- [ ] Contract tests for parse-roundtrip on supported fixtures; failure cases for broken input
- [ ] TOML either in-scope with same contract or explicitly labeled unsupported for private beta (document choice in issue comments if deferred)

## User stories covered

8, 9, 10 (partial — structure and useful context), 20 (partial)

## Blocked by

- `01-scaffold-core-cli-first-redact`
- `03-builtin-detector-pack` (or minimum detectors enough to hit values inside structures)

## Comments

-
