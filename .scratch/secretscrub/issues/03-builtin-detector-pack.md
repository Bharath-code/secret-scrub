# 03 — Built-in high-precision detector pack

Status: ready-for-agent  
Type: AFK  
Done: 2026-07-11 (build-loop)  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Ship a versioned **rule pack** of built-in detectors consumed by the core via a narrow rule-evaluation interface. Cover private-beta defaults with precision-first patterns: common provider secrets (AWS, GitHub, Stripe, OpenAI), JWT, generic key/value secrets, emails, and IP addresses. No live credential validation. No network.

End-to-end: one multi-secret log fixture scrubbed via CLI → all expected classes redacted with correct types; known non-secrets in a FP fixture left alone.

## Acceptance criteria

- [x] Rule-pack module loads built-in detectors with version identifier exposed to findings/export summary
- [x] Detectors implemented with linear-time-friendly matching (regex + literal/Aho-Corasick as appropriate); no catastrophic-backtracking patterns in defaults
- [x] Coverage at minimum: AWS-shaped, GitHub token-shaped, Stripe secret-shaped, OpenAI-shaped, JWT, email, IPv4 (and IPv6 if low-risk), generic `KEY=value` / header-style secrets
- [x] Overlapping candidates resolve deterministically (document precedence: longer match / higher-specificity type wins)
- [x] Golden fixtures for true positives per type; at least one FP-exclusion fixture for common non-secrets
- [x] TRUST.md non-claims respected: no “complete coverage” language in CLI help
- [x] Detector changelog stub exists for future pack changes

## User stories covered

4, 7, 25 (partial — versioned pack only, no signed updates yet)

## Blocked by

- `01-scaffold-core-cli-first-redact`
- `02-placeholders-value-correlation` (preferred so placeholders are typed correctly from day one of multi-detector)

## Comments

-
