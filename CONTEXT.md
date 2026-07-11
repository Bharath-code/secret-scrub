# CONTEXT.md — SecretScrub domain language

Single-context vocabulary for humans and agents. Prefer these terms in code, issues, and UI copy.

## Product

| Term | Meaning |
| --- | --- |
| **SecretScrub** | Local-first desktop app + CLI that produces safe-to-share copies of technical artifacts |
| **Safe copy / safe artifact** | Transformed output with sensitive values replaced; never the original file path overwritten |
| **Workspace** | One active scrub session: inputs, findings, placeholders, preview state |
| **Recipient profile** | Named redaction intent: AI prompt, vendor support, incident bundle |
| **Safety summary** | Non-sensitive export note: counts, types, rule-pack version, validation status |

## Core workflow

| Term | Meaning |
| --- | --- |
| **Scan** | Local detection + transformation pass (no upload) |
| **Finding** | One detected sensitive candidate with type, confidence/review state, occurrences |
| **Review** | User inspection of findings and original vs safe panes before export |
| **Export** | Atomic write of safe bundle (+ optional summary) to a user-chosen destination |
| **Review required** | Status when export must not claim fully safe (unsupported, partial, or unresolved issues) |
| **Clear workspace** | Remove temporary originals, value maps, and preview state |

## Detection and rules

| Term | Meaning |
| --- | --- |
| **Detector** | Pattern or matcher for a sensitive class (e.g. Stripe secret, email) |
| **Rule pack** | Versioned set of detectors + placeholder types; may be signed |
| **Custom rule** | User-defined local detector for org-specific identifiers |
| **Semantic placeholder** | Typed stable-within-workspace token, e.g. `[STRIPE_SECRET#1]` |
| **Value correlation** | Same original value → same placeholder inside one workspace only |
| **False positive (FP)** | Non-sensitive value treated as sensitive |
| **False negative (FN)** | Sensitive value missed by detectors |

## Architecture (modules)

| Term | Meaning |
| --- | --- |
| **Core domain library** | Shared Rust redaction engine used by CLI and desktop |
| **Rule-pack service** | Loads/validates detectors; exposes compiled policy to core |
| **Structured-artifact transformer** | Format-aware transform that preserves validity or reports partial/unsupported |
| **Artifact workspace module** | Import, temp handling, cancel, clear |
| **Export service** | Destination write, atomic completion, summary |
| **Desktop adapter** | Tauri commands + thin UI client |
| **CLI adapter** | Args, stdin/stdout, exit codes, machine-readable findings |

## Status language (export safety)

Use exactly these user-facing ideas:

- **Safe copy ready** — supported inputs reviewed; export allowed with stated detector scope  
- **Review required** — user must inspect; do not imply full safety  
- **Unsupported** — format/path cannot be marked safe  
- **Structure valid / invalid** — post-transform parse check for supported structured formats  

## Explicit non-goals (do not invent features for)

Cloud scanning, hosted artifact storage, CI/repo monitoring as v1, live credential validation, OCR/image redaction, SSO/team console as v1.

## Related docs

- `.scratch/secretscrub/PRD.md`  
- `.scratch/secretscrub/BUSINESS-PLAN.md`  
- `.scratch/secretscrub/ODDS-IMPROVEMENT.md`  
- `.scratch/secretscrub/TRUST.md`  
- `.scratch/secretscrub/COMPETITIVE.md`  
- `.scratch/secretscrub/VALIDATION.md`  
