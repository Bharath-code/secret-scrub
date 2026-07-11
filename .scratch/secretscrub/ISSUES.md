# SecretScrub — Implementation issues index

Parent PRD: [`PRD.md`](./PRD.md)  
Odds / gates: [`ODDS-IMPROVEMENT.md`](./ODDS-IMPROVEMENT.md)  
Domain language: [`../../CONTEXT.md`](../../CONTEXT.md)

Prefer AFK slices; grab lowest unblocked number.

| Progress | Slice |
| --- | --- |
| done | 01 scaffold, 02 placeholders, 03 detectors, 05 export (2026-07-11) |
| done | 04 structured formats, 06 folder, 07 CLI contract, 08 limits/cancel (2026-07-11) |
| next | 09 desktop MVP → 10 review UI → 11 profiles → 12 security baseline |

## Dependency graph (tracer bullets)

```text
01 scaffold + first redact
├── 02 placeholders + correlation
│   ├── 03 built-in detector pack
│   │   ├── 04 structured JSON/YAML/env
│   │   ├── 09 desktop scan/export MVP ──► 10 review + trust + a11y
│   │   └── 11 recipient profiles
│   └── 05 atomic export + safety summary
│       ├── 06 folder workspace bundle
│       │   ├── 07 CLI exit codes + findings JSON
│       │   └── 08 streaming + cancel + limits
│       └── 12 security regression + baseline
└── (01 also unblocks early CLI demos alone)
```

## Issue list

| # | File | Type | Title | Blocked by |
| --- | --- | --- | --- | --- |
| 01 | [issues/01-scaffold-core-cli-first-redact.md](./issues/01-scaffold-core-cli-first-redact.md) | AFK | Scaffold core + CLI first redaction path | — |
| 02 | [issues/02-placeholders-value-correlation.md](./issues/02-placeholders-value-correlation.md) | AFK | Semantic placeholders and value correlation | 01 |
| 03 | [issues/03-builtin-detector-pack.md](./issues/03-builtin-detector-pack.md) | AFK | Built-in high-precision detector pack | 01, 02 |
| 04 | [issues/04-structured-json-yaml-env.md](./issues/04-structured-json-yaml-env.md) | AFK | Structure-preserving JSON / YAML / env | 01, 03 |
| 05 | [issues/05-atomic-export-safety-summary.md](./issues/05-atomic-export-safety-summary.md) | AFK | Atomic export and safety summary | 01, 02 |
| 06 | [issues/06-folder-workspace-bundle.md](./issues/06-folder-workspace-bundle.md) | AFK | Folder workspace and multi-file bundle | 02, 04, 05 |
| 07 | [issues/07-cli-exit-codes-findings-json.md](./issues/07-cli-exit-codes-findings-json.md) | AFK | CLI exit codes and machine-readable findings | 01, 05, 06* |
| 08 | [issues/08-streaming-cancel-limits.md](./issues/08-streaming-cancel-limits.md) | AFK | Streaming, cancellation, hostile-input limits | 01, 05, 06 |
| 09 | [issues/09-desktop-scan-export-mvp.md](./issues/09-desktop-scan-export-mvp.md) | AFK | Desktop MVP scan + export (Tauri) | 03, 05, 07* |
| 10 | [issues/10-desktop-review-trust-a11y.md](./issues/10-desktop-review-trust-a11y.md) | AFK | Review panes, trust chrome, accessibility | 02, 09 |
| 11 | [issues/11-recipient-profiles.md](./issues/11-recipient-profiles.md) | AFK | Recipient profiles (AI / vendor / incident) | 03, 07, 09* |
| 12 | [issues/12-security-regression-and-baseline.md](./issues/12-security-regression-and-baseline.md) | AFK | Security regression + detector baseline | 05, 06, 08, 09* |

\* Preferred dependency; slice notes allow a thinner first land where marked.

## Suggested build order (private beta spine)

1. **01 → 02 → 03 → 05** — demoable CLI safe-share for a single file  
2. **04 → 06 → 07 → 08** — bundles, contracts, scale/cancel  
3. **09 → 10 → 11** — desktop review workflow  
4. **12** — harden before external beta  

## Explicitly deferred (not issues yet)

Per PRD out-of-scope / later phases:

- Offline Pro license, signed rule-pack updates, custom local rules UI  
- Windows/Linux packages, team policy console, SSO  
- Live credential validation, CI/repo integrations, OCR  
- Customer interview execution (process lives in `VALIDATION.md`)  

## How to triage

Set `Status:` on each issue file to one of: `needs-triage` | `needs-info` | `ready-for-agent` | `ready-for-human` | `wontfix`.
