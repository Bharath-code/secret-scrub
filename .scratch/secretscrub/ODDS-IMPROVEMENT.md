# SecretScrub — Odds Improvement Plan

Status: ready-for-agent

Purpose: convert every material **weakness** and **threat** into owned actions, proof, and kill criteria. This is how we stop hoping the PRD is enough.

Last updated: 2026-07-10

---

## North-star odds metric

We improve odds when **all three** are true:

1. **Demand proof** — real people show the pain and will try a beta.
2. **Trust proof** — users understand local-only processing and will export after review.
3. **Defensibility proof** — users pick SecretScrub over free scripts and enterprise scanners for the safe-share job.

Gate to heavy build (Tauri polish, licensing, multi-platform):

| Signal | Pass threshold |
| --- | --- |
| Problem interviews | 15 done; ≥8/15 recurring pain; ≥5 beta yes |
| Repeat use | ≥3 beta users return with a second real artifact |
| Willingness to pay | ≥3 verbal “would pay ~$39/yr” or ≥1 paid |
| Trust | ≥80% of observed first runs complete export without asking “is this uploaded?” |

If gates fail after 4 weeks of honest outreach → pause product build, re-interview, or kill.

---

## Weakness → counterplay

### W1 — No code, fixtures, or detector baselines

**Why odds drop:** Cannot demo; cannot measure FP/FN; every claim is theoretical.

| Action | Done when | Owner focus |
| --- | --- | --- |
| Scaffold Rust workspace (`core` + `cli`) | `cargo test` runs in CI-less local | Eng |
| Ship **golden fixtures** (AWS, GitHub, Stripe, JWT, email, IP, JSON/YAML preserve) | Fixture suite green; known bad cases documented | Eng |
| Publish **detector baseline** (precision/recall on fixture set only; no marketing claims) | `docs/detector-baseline.md` or fixture README | Eng |
| CLI vertical slice: path/stdin → redact → findings JSON + exit codes | Demo in &lt;60s | Eng |

**Odds rule:** Do not market detectors until fixtures + baseline exist. Demo the workflow, not coverage counts.

### W2 — No customer validation artifacts

**Why odds drop:** Building on founder conviction only; high kill risk after sunk cost.

| Action | Done when | Owner focus |
| --- | --- | --- |
| Run interview script (see `VALIDATION.md`) | 15 conversations logged | Founder |
| Capture last real artifact story (recipient, time spent, fear) | Each row has a concrete story | Founder |
| Score ICP fit (indie / agency / support) | ICP histogram filled | Founder |
| Beta waitlist with email + use case | ≥10 names | Founder |

**Odds rule:** Code time is capped until 8/15 pain threshold is met. Prefer interviews over polish.

### W3 — No public README / brand surface

**Why odds drop:** Repo looks empty; hard to recruit beta; weak trust for a privacy product.

| Action | Done when | Owner focus |
| --- | --- | --- |
| Root README with pitch, local-only promise, non-claims | README exists and is honest | Founder |
| One-page privacy promise (what never leaves device) | Linked from README | Founder |
| Sample “before → after” using **synthetic** fixtures only | Screenshot or CLI transcript in README | Eng |

**Odds rule:** Never put real customer secrets in docs. Synthetic only.

### W4 — Docs still `needs-triage` / not agent-ready

**Why odds drop:** Agent build loops thrash; work is not sliced.

| Action | Done when | Owner focus |
| --- | --- | --- |
| Accept PRD + business plan statuses after interview gate | Status → `ready-for-agent` | Founder |
| Slice tracer-bullet issues under `.scratch/secretscrub/issues/` | At least 5 numbered issues | Founder/Eng |
| Add `CONTEXT.md` + first ADR (module boundaries) | Domain terms stable | Eng |

### W5 — Naming drift (`rust-product` folder vs `secret-scrub` remote)

**Why odds drop:** Confusion in docs, issues, and mental model.

| Action | Done when | Owner focus |
| --- | --- | --- |
| Standardize product name **SecretScrub** / repo **secret-scrub** | README + AGENTS + PRD consistent | Founder |
| Optional: rename local folder when convenient | Local path matches mental model | Founder |

Low priority vs validation; fix when it costs &lt;10 minutes.

---

## Threat → counterplay

### T1 — Enterprise scanners own “secrets” mindshare

**Counter-position (memorize):**

> They find leaks in repos and pipelines.  
> We prepare **one artifact** so you can **share it safely** — locally, with review, as a usable copy.

| Action | Done when |
| --- | --- |
| Keep battle card current (`COMPETITIVE.md`) | One table buyers understand in 30s |
| Never compete on “most detectors” | Marketing checklist bans detector-count claims |
| Content angle: “safe share to AI / vendor” not “secret scanning” | First 3 content drafts use this frame |
| Landing hero: drop → review → export | Not “detect 1000 secret types” |

**Kill competitor comparison if:** buyers only want CI repo scanning. That is not our ICP.

### T2 — Free regex scripts and one-off habits

**Why they win on price:** free, already installed.

**Why we win when:**

1. **Consistent placeholders** across a multi-file incident folder  
2. **Structure preserved** (JSON still parses)  
3. **Review UI** that makes export confidence faster than ad-hoc sed  
4. **Honest unsupported** status (scripts fail silently)  
5. **Repeatable profiles** (AI prompt / vendor / incident)

| Action | Done when |
| --- | --- |
| Benchmark time: manual/script vs SecretScrub on same fixture bundle | Time-to-safe-copy metric logged |
| Free CLI covers the “try before Pro” path with size limits | Limit documented; upgrade path clear |
| Demo script for founders: 3-file folder with correlated token | 90-second wow path |
| In product copy: “faster than find-and-replace” not “replaces security team” | Copy review pass |

**Odds rule:** If beta users finish faster with their old script, we do not have a product yet — fix workflow, not pricing.

### T3 — One bad false negative destroys trust

**This is existential.** Treat as security + brand risk.

| Layer | Policy |
| --- | --- |
| Product language | Never “guaranteed safe.” Always “reviewed safe copy for common patterns.” |
| Default UI state | Export allowed after review; status can still say limitations |
| Unsupported / low confidence | Blocks misleading “all clear” badges |
| Scope of v1 detectors | Prefer high-precision provider patterns + emails/IPs over clever guesses |
| Regression | Every FN report becomes a fixture; no silent rule widening |
| User responsibility | Export summary lists detector pack version + what was checked |

| Action | Done when |
| --- | --- |
| Write trust policy (`TRUST.md`) | Linked from README and in-app later |
| FN intake template for beta | Users can report without pasting secrets |
| Conservative rule-pack process | Changelog + signature; no silent behavior change |
| Side-by-side review mandatory in desktop path | Cannot one-click export without seeing findings count |

**Odds rule:** Ship fewer detectors that users can understand rather than broad regex that hides miss risk.

### T4 — Platform risk (macOS signing, notarization, WebView)

| Action | When | Done when |
| --- | --- | --- |
| Apple Developer enrollment | Before public beta | Account active |
| Notarization dry-run on empty Tauri shell | Early in desktop work | Signed .dmg installs offline |
| Tauri capability matrix documented | Desktop PR1 | Default-deny list in ADR |
| CLI-first distribution | Now | Users can try without signed GUI |
| Fallback: CLI + simple TUI if WebView blocks | Contingency only | Decision logged |

**Odds rule:** Private beta can be **CLI + unsigned local build** for trusted testers. Public download requires signing.

### T5 — Low-frequency utility (retention)

| Action | Done when |
| --- | --- |
| Target high-frequency AI-debug users first | Interview tags show AI paste weekly+ |
| Profiles + custom rules make return visits worth it | ≥1 custom rule or profile reuse in beta |
| Optional export footer “Prepared with SecretScrub” | Opt-in only |
| Do not build monthly gamification | Explicit non-goal |

### T6 — $39/yr underprices or fails to signal trust

| Action | Done when |
| --- | --- |
| Price test in interviews (“$0 / $39 / $79 / $149”) | Van Westendorp-style notes on ≥10 people |
| Keep $39 as default until 5 paid; revisit after | Decision note after first paid cohort |
| Agency $249 only after multi-seat ask appears | No team console built early |

---

## 30-day odds sprint (ordered)

### Days 1–7 — Demand (highest ROI)

1. Schedule 15 interviews using `VALIDATION.md`.  
2. Log every call the same day.  
3. Freeze feature ideas that do not map to a quoted pain.  
4. Recruit 10 beta candidates (names &gt; waitlist vanity).

### Days 5–14 — Trustable thin product

5. Scaffold `core` + `cli`.  
6. Golden fixtures + baseline.  
7. CLI: scrub path/stdin → redacted out + findings JSON.  
8. README with synthetic before/after.

### Days 10–21 — Differentiation proof

9. Time trials vs manual redaction on incident-folder fixture.  
10. Battle-card language in all outreach.  
11. 10 private beta installs (CLI OK).  
12. Observe: export completion, questions about upload, second-use.

### Days 21–30 — Go / no-go

13. Score gates (top of this doc).  
14. If go: slice desktop review UI issues; start Apple signing track.  
15. If no-go: write postmortem; do not expand scope.

---

## Anti-patterns (odds destroyers)

- Marketing “military-grade” or “never leaks.”  
- Building Windows/Linux before macOS workflow is loved.  
- Adding CI/repo integrations to chase scanner competitors.  
- Silent detector changes without changelog.  
- Collecting telemetry by default.  
- Spending a month on visual polish before CLI proof.  
- Interviewing only friends who will be polite.

---

## Scoreboard (update weekly)

| Week | Interviews | Beta active | Second-use | Paid/verbal pay | CLI demo ready | Notes |
| --- | --- | --- | --- | ---: | --- | --- |
| 1 | 0 | 0 | 0 | 0 | no | baseline |
| 2 |  |  |  |  |  |  |
| 3 |  |  |  |  |  |  |
| 4 |  |  |  |  |  |  |

---

## Related files

- `VALIDATION.md` — interview script + log  
- `COMPETITIVE.md` — battle card vs scanners / scripts  
- `TRUST.md` — detection honesty + FN policy  
- `PRD.md` — product contract  
- `BUSINESS-PLAN.md` — market and GTM  
