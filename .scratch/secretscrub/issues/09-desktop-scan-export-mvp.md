# 09 — Desktop MVP: scan and export via Tauri

Status: needs-triage  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Stand up **Tauri v2 + React/TS** desktop shell as a thin client over the same Rust core. First vertical desktop path: user picks a file (or pastes text), sees local-only trust message, runs **Scan locally**, views findings summary, **Export safe copy** to a chosen location. Default-deny capabilities: only the commands and filesystem scopes required for this flow.

Not full visual polish or side-by-side panes (issue 10). Must call real core, not a mocked engine, for the happy path.

## Acceptance criteria

- [ ] Desktop app builds on macOS 14+ target; works offline for scan/export after launch
- [ ] Commands exposed are minimal and typed; no broad shell or arbitrary FS access
- [ ] Restrictive CSP; no remote scripts/fonts/analytics loaded in the app WebView
- [ ] Flow: input → scan progress → findings count/types → export → reveal/output path
- [ ] Original never modified; export uses core export service semantics
- [ ] E2E test against command boundary (not mocked redaction) for happy path + permission/export failure messaging
- [ ] Persistent “Processed locally — nothing uploaded” visible on the primary screen

## User stories covered

1, 2, 15, 18, 29 (partial), 33

## Blocked by

- `05-atomic-export-safety-summary`
- `03-builtin-detector-pack`
- `07-cli-exit-codes-findings-json` (preferred so status enums are shared)

## Comments

-
