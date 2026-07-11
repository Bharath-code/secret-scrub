# SecretScrub — Competitive Battle Card

Status: ready-for-agent

Use this in sales, landing copy, and founder conversations. Goal: win the **safe-share** job, not the **enterprise detection** job.

---

## Category we own

**Local privacy preflight for technical artifacts.**

Job-to-be-done:

> When I need to share production context with AI, a vendor, or a teammate, I want a reviewed safe copy in seconds so I don’t leak secrets or waste time on manual redaction.

---

## 30-second contrast

| If buyer says… | We say… |
| --- | --- |
| “We already have GitGuardian / TruffleHog.” | “Great for finding leaks in git/CI. SecretScrub is for the moment you’re about to **paste or send one log** — local review + usable export.” |
| “I just run sed / a regex.” | “Works once. Falls over on multi-file bundles, consistent placeholders, valid JSON, and knowing what you missed.” |
| “Isn’t this DLP?” | “No org policy engine. One developer, offline, one artifact, export proof.” |
| “Will it catch everything?” | “No tool does. We catch common patterns, show every change, and never claim universal safety.” |

---

## Comparison matrix

| Capability | SecretScrub | TruffleHog / GG-class scanners | DIY regex/script | Generic DLP |
| --- | --- | --- | --- | --- |
| Primary output | Safe **export** + review | Findings / alerts | Edited text | Block/alert events |
| Local-first offline core | Yes (product requirement) | Varies; often service-oriented | Yes | Rarely for this UX |
| Side-by-side review UX | Core workflow | Not the product | No | No |
| Semantic consistent placeholders | Yes within workspace | N/A (detect, not transform) | Manual / brittle | No |
| Preserve JSON/YAML validity | Explicit goal | N/A | Easy to break | N/A |
| CI/repo continuous monitoring | Out of scope | Strength | Possible with glue | Adjacent |
| Credential live validation | Out of scope v1 | Often a feature | No | No |
| Buyer | Individual / small team | Security / platform | Individual hacker | Enterprise security |
| Time-to-value for “share this log” | Seconds | Wrong workflow | Minutes–hours | Procurement cycle |

---

## When we **lose** on purpose

Walk away or redirect if they need:

- Org-wide secret sprawl dashboards  
- PR/CI blocking as the main job  
- SSO, audit, centralized policy console first  
- Automatic key rotation  

Those are real needs — different products. Competing there destroys focus and odds.

---

## When we **must win**

Buyer situation:

1. Pastes into ChatGPT/Claude/Copilot during incidents  
2. Sends client logs to a host/plugin/payment vendor  
3. Prepares multi-file support bundles  
4. Has no security team and does not want a hosted log sink  

Proof they need:

- Faster than their current path  
- Local-only trust is legible  
- Safe copy still useful for debugging  

---

## Messaging do / don’t

### Do

- “Share production context safely in seconds.”  
- “Drop → review → export. Nothing uploaded.”  
- “Consistent placeholders so the story still makes sense.”  
- “Review required when we can’t mark something safe.”

### Don’t

- “Best-in-class secret detection.”  
- “Military-grade / zero risk / never leaks.”  
- “Replaces GitGuardian.”  
- Detector count leaderboards  

---

## Objection handling

| Objection | Response | Proof to build |
| --- | --- | --- |
| “Free tools exist” | Free CLI for try; Pro is review + bundles + rules | Time trial vs script |
| “I don’t trust desktop apps” | Open core engine path later; signed builds; local-only indicator; no account | Privacy statement + offline demo |
| “False negatives scare me” | Explicit limits + review UX + export summary with pack version | TRUST.md + FN fixtures |
| “We rarely need this” | Aimed at AI-debug frequency; profiles make return easy | Interview frequency filter |
| “Too cheap / too expensive” | $39 validation price; agency plan if multi-seat appears | Price questions in VALIDATION.md |

---

## Content angles that steal demand (not scanner SEO)

1. How to safely share logs with ChatGPT  
2. Incident bundle checklist before vendor support  
3. Redacting `.env` and JSON without breaking structure  
4. Agency playbook: client logs to third parties  
5. Why “find secrets in git” ≠ “make this log shareable”

Each piece ends with: local tool, review, export — not “deploy our scanner.”
