# 02 — Semantic placeholders and value correlation

Status: ready-for-agent  
Type: AFK  
Done: 2026-07-11 (build-loop)  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Within one **workspace** scrub, allocate **semantic placeholders** (e.g. `[AWS_ACCESS_KEY#1]`) and ensure the same original sensitive value maps to the same placeholder everywhere it appears. Different values of the same type get distinct indices. Placeholders must **not** be stable across separate scrub sessions (no cross-export correlation).

End-to-end: multi-line log fixture with the same secret twice and a second distinct secret → consistent placeholders in the safe copy + findings that report occurrence counts.

## Acceptance criteria

- [x] Placeholder format is typed and human-readable (`[TYPE#N]`), not anonymous `***`
- [x] Same value → same placeholder across the entire artifact (and later multi-file workspace when that exists)
- [x] Distinct values → distinct indices; order is deterministic for a given input
- [x] Two independent scrub calls on the same input do not guarantee identical placeholder indices (document and test non-stability across sessions)
- [x] Findings include detector type, placeholder, and occurrence count
- [x] Golden fixture tests cover repeated values, single occurrence, and mixed types

## User stories covered

5, 6, 12 (partial)

## Blocked by

- `01-scaffold-core-cli-first-redact`

## Comments

-
