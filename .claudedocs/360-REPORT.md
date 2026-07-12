# SecretScrub — 360° Report

Date: 2026-07-12
Scope: code review, architecture, business review, ICP, first-10-customers plan
Verified: `cargo test` 43/43 pass, clippy clean, bugs below reproduced with live CLI runs

## TL;DR

Engineering is unusually disciplined for pre-launch (43 passing tests, honest failure
states, atomic exports), but there is one **critical, reproduced bug that breaks the
core product promise**: placeholder correlation is nondeterministically wrong when a
file contains two distinct secrets of the same type. On the business side, the plans
are excellent on paper but **zero of the 15 planned validation interviews have
happened** — the project is violating its own "build only after demand proof" gate.
Fix the bug, then stop coding and go talk to 15 people.

---

## 1. Code Review

### CRITICAL — placeholder correlation is broken (reproduced)

`crates/core/src/placeholder.rs:47` — `reindex_type()` re-assigns display indices for
*all* previously seen values every time a new value of that type appears. But earlier
placeholders were already baked into output text (env files scrub per line, JSON per
string, workspaces per file).

Reproduction (two distinct AWS keys + one repeat, 6 runs, random seeds):

```
first=[AWS_ACCESS_KEY#1] second=[AWS_ACCESS_KEY#1] first_again=[AWS_ACCESS_KEY#2]
```

In 3 of 6 runs two **different** secrets got the same placeholder and the repeated
value got a different one. This inverts the signature feature (same value → same
placeholder; different values distinguishable). Tests miss it because
`correlates_across_files` uses one distinct value and unit tests use seed 0.

**Fix:** assign each value its next sequential index at first sight; never re-index.
Delete `reindex_type`/`offset_for` (the seed permutation adds no privacy — correlation
is per-workspace by design — and is the direct cause). Add a regression fixture: two
distinct values + a repeat, across two files.

### Medium

1. **JSON key reordering** — `structure.rs:102` uses default `serde_json` (BTreeMap);
   keys come out alphabetized (reproduced: `zeta/alpha/mid` → `alpha/mid/zeta`).
   Valid JSON, but review diffs become noise and field order can carry meaning.
   Fix: enable `preserve_order` feature on serde_json.
2. **`serde_yaml` is unmaintained** (archived March 2024). Abandoned parser on the
   trust path of a security product. Consider `serde_yml` or `saphyr` before beta.
3. **OpenAI detector false positive** — `detect.rs:49` `sk-[A-Za-z0-9_-]{20,}`
   matched `sk-formatting-helper-utils-v2` (reproduced). Tighten (require digits /
   higher entropy) per the stated precision-first policy.
4. **Force + cancel can destroy user data** — `workspace.rs:493` `--force` deletes an
   existing destination dir, then cancel mid-export (`workspace.rs:503`) deletes the
   partial tree: old contents and new export both gone. Consider temp-sibling +
   whole-tree rename (per-file `atomic_write` already does this).

### Minor

- PRD promises streaming; `workspace.rs:220` reads whole files. Fine at the 10 MiB
  limit — update the PRD claim rather than building streaming.
- Empty files silently excluded from exported bundles (`workspace.rs:317`); copy them
  through verbatim so bundles aren't missing files.
- JSON/YAML keys never scanned (values only); document in TRUST.md.

### Genuinely good

Fail-closed unsupported handling (TOML/binary emit empty text, never original),
symlinks skipped with recorded reasons, atomic temp+rename exports, source-overwrite
refusal, no secret values in summaries (tested), stable 4-value exit-code contract,
black-box CLI tests.

---

## 2. Architecture Review

**Verdict: sound, appropriately boring, right-sized.** Core/CLI split with CLI as a
thin adapter is exactly right for the planned Tauri desktop — the shell becomes a
third adapter over the same `scrub_workspace` API. Module boundaries
(detect → placeholder → structure → workspace → export) map cleanly to the PRD.

Watch items:

- **`ScrubConfig.rule_pack` is decorative** — only supplies a version string;
  detectors are a hardcoded static list in `detect.rs`. Correct YAGNI now, but custom
  rules / signed rule-packs (a paid feature) will force detectors behind the RulePack
  interface. Decide the shape before the desktop app couples to the current one.
- **Findings lack positions.** `Finding` is `{type, placeholder, occurrences}` — no
  line/offset. The desktop review UX ("jump to occurrence") needs spans; adding them
  later reworks `redact_plain` counting. Decide the shape before Tauri work.
- `O(n·m)` regex pass is fine at current limits; Aho-Corasick is premature until
  profiling says otherwise.

---

## 3. Business Review

Planning artifacts (PRD, business plan, battle card, TRUST.md) are top-decile:
honest anti-claims, a real category ("local privacy preflight"), explicit walk-away
criteria, self-imposed gates.

**The plan is not being followed.** VALIDATION.md targets 15 interviews in 14 days
before deep investment ("Fail any gate → do not expand beyond CLI spike"). The
interview log is an empty template — zero conversations — yet the repo already has a
full engine, workspace bundles, a design system, and a marketing landing page. That
is the exact "build before demand" risk the risk table marks Critical.

Underweighted strategic risks:

1. **Frequency/retention** — "share a sensitive log" may be monthly, not weekly.
   Great problem, bad habit loop. #1 thing interviews must measure.
2. **"Good enough" ceiling** — LLM chat UIs increasingly warn on pasted secrets; a
   sed script covers the single-file case. Defensible ground = multi-file correlated
   bundles + review UX + structural validity — exactly the feature the critical bug
   breaks. The demo is the moat; the demo is currently broken ~50% of the time.
3. **$39/yr × 5 customers is proof, not revenue** — fine, but keep desktop spend
   gated on that proof.
4. **Pricing asymmetry** — free CLI does nearly everything Pro promises except the
   GUI review pane. Interviews should probe what a CLI-only user would pay for.

---

## 4. ICP Assessment

| ICP | Assessment |
|---|---|
| 1. AI-assisted indie founder (1–10 eng) | Right first target; sharpen trigger to **"pastes production output into an LLM ≥3×/week"** — observable and self-identifying. |
| 2. Agencies (WordPress/Shopify/DevOps) | Underrated — stronger willingness-to-pay (fear is *client* data: contractual/reputational) and naturally higher vendor-escalation frequency. Consider co-first. |
| 3. B2B SaaS support teams | Correctly deferred; procurement kills solo-founder cycle time. |

Missing ICP: **AI-agent power users** (Claude Code / Cursor users running agents
against production repos and logs). Highest sharing frequency of any segment; live in
the CLI; a `secretscrub` pipe or MCP hook fits natively. Free-tier evangelists even
if not first payers.

---

## 5. First 10 Customers

Prerequisite: fix the correlation bug (found in minute one of any demo), then run
the interviews. Sequence, not parallel spray:

- **1–3: warm network (wk 1–2).** Founders/agency owners shipping production SaaS.
  VALIDATION.md interview, then live CLI on *their* real artifact. Ask for $39 the
  moment they export something they actually send. Founding-user price lock is fine;
  free copies prove nothing.
- **4–6: communities (wk 2–5).** Indie Hackers, r/SaaS, r/selfhosted, Show HN
  ("local CLI that makes logs safe to paste into ChatGPT"), Cursor/Claude-Code
  Discords. Lead with the honest angle — the trust posture is the differentiator.
  Free CLI is the funnel; desktop review UX is the paid pitch.
- **7–8: agency outreach (wk 4–8).** 20–30 small WordPress/Shopify/hosting agencies.
  Pitch: "When your engineers send client logs to Kinsta/Stripe/a plugin vendor,
  what stops a client API key going with them?" Offer $249 agency pilot to two.
- **9–10: content inbound (wk 6–12).** "How to safely share logs with AI," "What to
  redact before vendor support," plus `brew install secretscrub` and a GitHub
  presence. High buyer intent, near-zero competition.

**Kill/persevere gate (enforce your own docs):** if after ~12 weeks there aren't 5
payers, or beta users don't return with a second real artifact within 30 days, the
frequency thesis failed — re-segment to agencies-only or agent-workflow integration
before writing the Tauri app.

---

## 6. Final 360 Verdict

| Dimension | Grade | One-liner |
|---|---|---|
| Code quality | B+ | Rigorous and honest; one critical reproduced bug in the flagship feature |
| Architecture | A− | Clean core/adapter split; findings need positions before desktop work |
| Business plan | A (paper) / D (execution) | Excellent docs; 0 of 15 mandatory interviews done |
| ICP clarity | B+ | Right segments, right order; sharpen trigger, add agent-users |
| Overall odds | Fixable inversion | Product risk small and known; demand risk open — only interviews retire it |

**Do next, in order:** (1) fix `placeholder.rs` re-indexing + regression fixture,
(2) `serde_json/preserve_order`, (3) freeze features and run the 15 interviews,
(4) only then decide the Tauri desktop spend.
