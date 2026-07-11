# Detector rule-pack changelog

Rule pack versioning is independent of the application semver when needed.
Current bundled pack: **0.1.0** (`builtin`).

## 0.1.0 — 2026-07-11

### Added

- `AWS_ACCESS_KEY` — `AKIA` / `ASIA` + 16 uppercase alphanumerics  
- `GITHUB_TOKEN` — `ghp_` and `github_pat_` shapes  
- `STRIPE_SECRET` — `sk_live_` / `sk_test_`  
- `OPENAI_API_KEY` — `sk-` / `sk-proj-` (Stripe shapes excluded)  
- `JWT` — three-part `eyJ…` compact serialization  
- `GENERIC_SECRET` — common key names with `=` / `:` values  
- `EMAIL` — basic address shape  
- `IP_ADDRESS` — IPv4  

### False-positive notes

- IPv4 may match some dotted quads in non-network contexts; review before export.  
- `GENERIC_SECRET` only fires on named keys with values ≥ 8 chars.  

### Process

- Prefer precision over recall.  
- Every detector change needs fixtures (TP and known FP risks).  
- No silent widening of defaults without this changelog entry.  
