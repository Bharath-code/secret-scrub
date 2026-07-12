# SecretScrub — Trust & Detection Honesty Policy

Status: ready-for-agent

Trust is the product. One overclaim or silent miss can end the company. This document is binding for product copy, UX, detectors, and support.

---

## Hard non-claims (never say these)

- “Guaranteed safe to share”  
- “Finds every secret / all PII”  
- “Compliant with [regulation] because you used SecretScrub”  
- “Validates that credentials are inactive”  
- “Military-grade / bank-grade / unbreakable”  

## Allowed claims (only if true in product)

- “Processed locally on your device for scan, review, and export in the default workflow”  
- “Detects **common** provider tokens, JWTs, emails, IPs, and configured custom rules”  
- “Replaces repeated values with **consistent placeholders within one workspace**”  
- “Preserves structure for **supported** formats; marks unsupported files for review”  

## Known detection limits (state these plainly)

- JSON and YAML **keys are not scanned** — only values are. A secret used as a
  key (or a key name that is itself sensitive) passes through unchanged.
- When a key name looks secret-ish (`password`, `passwd`, `api_key`,
  `secret_key`, `access_token`, `auth_token`, `private_key`,
  `client_secret` — any underscore/hyphen/case spelling), the entire
  **string** value under that key is redacted, even if no detector
  pattern matches the bare value. Non-string values (numbers, booleans)
  under a secret-ish key are left unchanged, and nested objects/arrays
  under a secret-ish key are walked normally rather than blanket-redacted.
- Files over the per-file size limit (10 MiB) are excluded and reported, not
  scanned. Processing is whole-file, not streaming.

- “Never overwrites your original input”  
- “Optional network only for license/update paths you can disable after activation (when those features ship)”  

---

## Detection philosophy (odds vs false negatives)

| Principle | Practice |
| --- | --- |
| Precision over cleverness | Prefer high-signal provider patterns before broad fuzzy matches |
| User review is part of safety | Desktop path always surfaces findings before export |
| No silent “all clear” | Unsupported, partial, or low-confidence → Review required |
| Conservative rule changes | Widening a detector requires fixtures + changelog |
| Scope is versioned | Export summary includes rule-pack / product version |
| Correlation scope | Placeholder maps are never persisted; correlation is guaranteed only within one workspace scrub. Indices are first-seen sequential, so re-scrubbing identical input reproduces the same placeholders — do not present placeholder numbering as an anonymity layer |

### Severity of misses

| Event | Response |
| --- | --- |
| Beta FN report | New fixture; fix or document limitation; thank reporter; never shame |
| Public FN with harm | Incident note; patch if feasible; transparent changelog; do not gaslight |
| User shared raw after ignoring review | Empathy + UX review (was warning clear?) — not product liability theater |

---

## UX trust requirements (must ship with desktop)

1. Persistent **“Processed locally — nothing uploaded”** on input and review.  
2. Findings list with type + occurrence count + jump-to-location.  
3. Side-by-side or segmented original vs safe copy.  
4. Status badges: icon + text (not color alone).  
5. Export summary: counts by type, structure validation, excluded files + why.  
6. Clear path to mark “keep for this export” with warning (one export scope).  
7. Cancel and clear never leave a half-written “safe” destination file.

CLI trust requirements:

- Exit codes distinguish: clean / review-required / unsupported / failure  
- Machine-readable findings without printing secret values in summaries when avoidable  
- `--help` states local processing and detection limits  

---

## False negative intake (beta)

Users must be able to report a miss **without** sending the secret.

Template:

```
Product version:
Rule pack version:
File type (log/json/yaml/env/other):
Detector they expected:
Description of pattern shape (e.g. "Stripe live secret prefix") — NOT the real value:
Did review UI show anything nearby?:
Can you add a synthetic fixture that reproduces?: yes/no
```

Support rule: **never require original artifacts**. If support cannot proceed without content, decline and improve docs/fixtures instead.

---

## Rule-pack change control

1. Add/change detector only with: intent, sample true positives, known FP risks, fixtures.  
2. Prefer new optional rules over silently broadening defaults.  
3. Sign rule packs when update channel exists.  
4. Changelog in plain language: added / changed FP behavior / action required.  

---

## Privacy defaults

| Data | Policy |
| --- | --- |
| Artifact content | Local workspace only; cleared on clear/close |
| Telemetry | Off by default |
| Diagnostics | Opt-in; version + error category only; no paths, names, snippets, fingerprints of secrets |
| Accounts | Not required for core scrub in v1 |
| Marketing site analytics | Separate from desktop app; never load into app WebView |

---

## Public trust artifacts (build these)

| Artifact | Purpose |
| --- | --- |
| README privacy section | First impression for engineers |
| In-app privacy statement | Buyers who open the app |
| Detector baseline (fixture-only metrics) | Honest quality signal for us, not vanity marketing |
| SBOM + signed releases | Serious buyers and agencies |
| “How we handle FN reports” page | Turns fear into process |

---

## Copy snippets (approved tone)

**Good:**  
“SecretScrub redacts common secrets and identifiers locally, shows every change, and exports a copy you can review. It cannot guarantee that every sensitive value is found.”

**Bad:**  
“SecretScrub makes your logs 100% safe to paste anywhere.”
