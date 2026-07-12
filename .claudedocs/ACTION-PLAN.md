# SecretScrub — Action Plan

Date: 2026-07-12
Source: `.claudedocs/360-REPORT.md`
Rule: phases are sequential gates. Do not start a phase until the prior gate passes.

## Phase 0 — Fix trust-breaking bugs (this week, ~1 day of work)

- [x] **P0** Fix placeholder correlation (`crates/core/src/placeholder.rs`)
      Replace `reindex_type`/`offset_for` with a per-type sequential counter:
      first-seen value gets the next index, indices never change afterwards.
      Remove `session_seed` from the allocation path (keep the CLI flag as a no-op
      or drop it — correlation is per-workspace by design, the permutation added
      no privacy and caused the bug).
- [x] **P0** Regression fixture: one workspace, two files, two distinct
      AWS keys + one repeated key; assert same value → same placeholder in both
      files, distinct values → distinct placeholders, across many seeds/runs.
- [x] **P1** `serde_json = { features = ["preserve_order"] }` so scrubbed JSON
      keeps key order; assert order in `json_roundtrip_parseable`.
- [x] **P1** Tighten `OPENAI_API_KEY` regex (require entropy/digits; fixture:
      `sk-formatting-helper-utils-v2` must NOT match).
- [x] **P2** Export safety: build workspace export in a temp sibling dir and rename
      into place, so `--force` + cancel can't delete a pre-existing destination.
- [x] **P2** Swap `serde_yaml` (archived) for `serde_yml` or `saphyr`.
- [x] **P2** Copy empty files through to exported bundles instead of excluding.
- [x] **P2** Docs honesty pass: PRD "streaming" claim → whole-file at ≤10 MiB;
      TRUST.md: JSON/YAML keys are not scanned.

**Gate:** `cargo test` green including new regression fixtures; the two-key demo is
stable across 20 runs.

## Phase 1 — Validation interviews (weeks 1–2, feature freeze)

No new product code except Phase 0 items. Run VALIDATION.md as written:

- [ ] 15 interviews: ~8 indie founders who paste prod output into LLMs weekly,
      ~5 agency owners/leads, ~2 support engineers.
- [ ] Each interview: last real artifact they hesitated to share, recipient, how
      they redacted, time spent, what they feared; then live CLI demo on THEIR
      artifact; then price ladder (probe CLI-only willingness to pay, not just Pro).
- [ ] Log every interview in VALIDATION.md (the log is currently empty).
- [ ] Score the gate table after 15.

**Gate (from VALIDATION.md):** ≥8/15 describe recurring manual redaction or
avoidance; ≥5 agree to private beta. Fail → re-segment (agencies-only or
AI-agent-workflow ICP) or kill before any desktop spend.

## Phase 2 — Private beta + first customers (weeks 3–8)

- [ ] Recruit 10 beta users from Phase 1 + communities (Indie Hackers, Show HN,
      r/SaaS, Cursor/Claude-Code Discords). Honest-tool framing.
- [ ] Distribution: GitHub repo public, `brew install` tap, one-line install.
- [ ] Customers 1–3: warm network, live demo on their artifact, ask for $39
      founding-user price at the moment of first real export.
- [ ] Customers 4–6: community inbound from the free CLI.
- [ ] Customers 7–8: direct outreach to 20–30 WordPress/Shopify/hosting agencies;
      offer 2 pilots of the $249 agency plan.
- [ ] Instrument the trust metrics from BUSINESS-PLAN.md (time-to-safe-copy,
      30-day second-use rate, support tickets needing originals = 0).

**Gate (from BUSINESS-PLAN.md):** ≥60% complete an export in <5 min; ≥3 users
return with a second real artifact within 30 days.

## Phase 3 — Paid validation + content (weeks 6–12)

- [ ] Content: "How to safely share logs with AI", "Safe incident-bundle
      checklist", "What to redact before vendor support".
- [ ] Customers 9–10 from content/search inbound.
- [ ] Only if paying users demand it: offline licensing, signed rule packs,
      custom rules (this is when RulePack stops being decorative — move detectors
      behind it, and add positions/spans to `Finding` for the review UX).

**Gate:** 5 paid individuals or 2 paid agencies at list price, no discount pressure.

## Phase 4 — Desktop (only after Phase 3 gate)

- [ ] Tauri shell as a third adapter over `scrub_workspace` (no engine rework).
- [ ] Prerequisites decided in Phase 3: Finding spans, RulePack interface.
- [ ] macOS signing/notarization dry run before public download.

## Kill / persevere

At ~12 weeks: no 5 payers OR no repeat usage → the frequency thesis failed.
Re-segment to agencies-only or agent-workflow integration (MCP hook / pre-paste
pipe for Claude Code & Cursor users) before writing any desktop code.
