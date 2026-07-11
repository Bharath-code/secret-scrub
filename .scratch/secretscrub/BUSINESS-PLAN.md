# SecretScrub — Business Plan and Market Rationale

Status: needs-triage  
Odds plan: see `ODDS-IMPROVEMENT.md`, `TRUST.md`, `VALIDATION.md`, `COMPETITIVE.md`

## Executive Summary

SecretScrub is a local-first desktop app and CLI that turns logs, configuration files, code snippets, and incident bundles into safe-to-share copies. It detects secrets and sensitive identifiers, replaces them consistently with meaningful placeholders, preserves the debugging structure, and exports a reviewed safe artifact.

The product is not a generic secret scanner. Its job is to make safe sharing faster than manual redaction.

## Why This Product Should Exist

Modern debugging depends on sharing context: a founder asks an AI assistant about a production error, an agency sends a client log to a vendor, or a support engineer prepares an incident bundle. Those artifacts routinely contain credentials, tokens, email addresses, IP addresses, and customer information.

The current choices are both poor:

- Manually find and replace sensitive values, which is slow, inconsistent, and easy to get wrong.
- Strip so much information that an AI tool, vendor, or teammate cannot diagnose the problem.

SecretScrub creates a third option: a usable, reviewed, local safe copy in seconds. It turns data protection from a security-team afterthought into a normal developer workflow.

Sensitive-information disclosure is identified as a major LLM-application risk by OWASP. Secret-management vendors also document broad scanning support across logs, configurations, filesystems, and source artifacts, demonstrating that sensitive values appear in exactly the artifacts users need to share during debugging. The opportunity is not to outbuild those platforms on detection coverage; it is to own the last-mile safe-share workflow.

## Market Problem

### Functional problem

Users need to share technical evidence quickly, but cannot safely determine whether it contains sensitive information or redact it without damaging the evidence.

### Economic problem

Manual redaction delays incident resolution and consumes expensive engineering or support time. A disclosure can trigger credential rotation, customer communication, support escalation, and reputational damage.

### Emotional problem

Users are uncertain whether an artifact is safe to paste into an AI prompt or send externally. That uncertainty either causes risky behavior or blocks productivity.

### Market gap

Enterprise secret scanners discover and manage exposures. SecretScrub prepares a single artifact for safe sharing before exposure occurs.

| Category | Primary output | Primary buyer | SecretScrub distinction |
| --- | --- | --- | --- |
| Secret scanner | Finding, alert, remediation ticket | Security or platform team | Exports a usable safe copy instead of only reporting a finding |
| DLP platform | Policy enforcement and compliance event | Enterprise security | Works offline and locally for one developer workflow |
| Screenshot/log tool | Capture, storage, or display | Individual developer | Redacts and preserves debugging relationships across an artifact |
| Manual redaction | A risky edited copy | Any technical user | Consistent placeholders, review, structural validation, and export proof |

## Positioning

**Category:** local privacy preflight for technical artifacts.

**One-line pitch:** “Share production context safely in seconds.”

**Outcome statement:** “Drop a log or config, review what changed, and export a safe copy without uploading the original.”

**What we do not claim:** SecretScrub cannot guarantee detection of every sensitive value and does not validate whether a credential is active. It reduces common human error in a defined sharing workflow.

## Ideal Customer Profiles

### ICP 1 — AI-assisted indie SaaS founder or small product team

**Profile:** one to ten engineers; production SaaS; uses ChatGPT, Copilot, Claude, or vendor support to debug; no dedicated security team.

**Trigger:** a production incident, a customer support escalation, or a recurring fear of pasting production logs into an AI tool.

**Pain:** needs an answer now, knows raw logs are unsafe, and does not want an enterprise security platform.

**Buyer and user:** founder, technical cofounder, senior engineer.

**Why this is first:** short buying cycle, direct access, high AI-tool usage, and a simple individual license.

### ICP 2 — Web-development, DevOps, or software agency

**Profile:** two to twenty technical people handling many client environments and vendor escalations.

**Trigger:** sharing logs/configuration with a hosting provider, plugin vendor, payment provider, or client.

**Pain:** client data and keys cross team boundaries; manual redaction is inconsistent; an agency needs a repeatable process.

**Buyer and user:** agency owner, technical lead, support engineer.

**Why this is second:** higher willingness to pay and clear team policy need, but longer validation and support cycle.

### ICP 3 — B2B SaaS support or solutions engineering team

**Profile:** support engineers receive production diagnostics from customers and need safe escalation internally or to a vendor.

**Trigger:** an incident package must be shared outside the original support boundary.

**Pain:** sensitive customer context slows escalation and risks mishandling data.

**Buyer and user:** support operations lead, head of engineering, security champion.

**Why this is later:** stronger compliance requirements, procurement friction, and expectations for team management.

### Customers to avoid initially

- Large regulated enterprises requiring SSO, audit trails, centralized policy management, data-residency guarantees, and formal compliance certifications.
- Teams seeking an all-purpose secret scanner, a DLP product, or automatic credential rotation.
- Users who primarily need screenshot capture, cloud backup, or generic file organization.

## Buyer-Focused User Stories

1. As an indie founder, I want to sanitize a production error before asking an AI assistant for help, so that I can debug quickly without pasting a key or customer data.
2. As an agency engineer, I want a repeatable safe-share workflow for every client environment, so that I do not rely on each employee remembering manual redaction steps.
3. As an agency owner, I want a local tool rather than a hosted log service, so that client artifacts do not create another vendor-risk surface.
4. As a support engineer, I want to send a vendor enough diagnostic context to reproduce an issue, so that I do not spend days exchanging incomplete logs.
5. As a technical lead, I want placeholders to remain consistent within a bundle, so that engineers can follow a request or account through an incident without seeing the original values.
6. As a buyer, I want to see exactly what was redacted before I export, so that I can trust the tool instead of adding another manual review step.
7. As a privacy-conscious developer, I want the core workflow to work offline, so that I can use it during an outage or under strict customer-data rules.
8. As a team lead, I want custom local rules for company identifiers, so that generic detectors do not miss our domain-specific data.
9. As a support lead, I want a non-sensitive export summary, so that I can document how a safe bundle was prepared without retaining the original artifact.
10. As a Pro subscriber, I want signed detector updates, so that new token formats are covered without giving a vendor access to my artifacts.
11. As an evaluator, I want a free CLI and a sample artifact mode, so that I can validate usefulness before purchasing a desktop license.
12. As a user, I want an honest review-required state for unsupported files, so that the product never gives me false confidence.

## Product Strategy

### Beachhead product

Launch a macOS desktop application plus a free CLI focused only on text, logs, `.env` files, JSON, YAML, TOML, and source-like files. The desktop experience wins through drop → scan → review → export. The CLI creates developer trust and distribution.

### 10× experience

The product succeeds when it makes the safe path materially faster than the unsafe one:

1. Drop a production log, folder, or pasted text.
2. Scan locally and stream results without freezing the machine.
3. Show a side-by-side original/safe-copy review with consistent semantic placeholders.
4. Export a valid, useful, non-destructive safe bundle.

The “holy-sh*t” moment is a large incident folder becoming a shareable bundle in seconds while the user can still trace the same redacted token, account, or request across every file.

### Product boundaries

The first release deliberately does not scan repositories, monitor cloud systems, store customer artifacts, validate live credentials, or operate as a DLP platform. Focus makes the product easier to trust, build, and maintain.

## Business Model

| Plan | Customer | Price | Included value |
| --- | --- | --- | --- |
| Free CLI | Individual developer | Free | Standard detectors, local path/standard-input scanning, limited file/bundle size, structured findings output |
| Pro Desktop | Founder or individual professional | $39/year | Unlimited local scans, full desktop review/export, local custom rules, offline license, signed rule-pack updates |
| Agency | Small technical agency | $249/year for up to five users | Pro features, shared export-profile templates, team rule-pack distribution, priority support |

The first revenue goal is not scale. It is proof that users pay for reduced risk and saved time: five paid Pro customers and two paid agencies after a private beta.

### Unit-economics assumptions to validate

- Distribution and processing are almost entirely local, so marginal artifact-processing cost should be near zero.
- The meaningful recurring costs are payment processing, license delivery, signed update hosting, support, code signing, and development.
- Annual pricing aligns with low operational overhead and avoids turning a local utility into a heavy monthly procurement decision.
- A team plan is justified only if agencies demonstrate demand for consistent profiles and rules; do not build centralized management before that signal.

## Go-to-Market Plan

### Phase 1 — Problem validation (weeks 1–2)

- Conduct 15 conversations with indie founders, agency engineers, and support engineers.
- Ask for the latest real artifact they hesitated to share, the recipient, how they redacted it, time spent, and what they feared leaking.
- Success threshold: at least 8 of 15 describe manual redaction or avoidance as a recurring problem; at least 5 agree to test a private beta.

### Phase 2 — Private beta (weeks 3–6)

- Build the macOS drop → review → export workflow and free CLI.
- Recruit 10 beta users from founder communities, agency contacts, and developer-security communities.
- Observe whether users complete a scrub without guidance and whether they trust the local-only claim.
- Success threshold: at least 60% complete an export in under five minutes, and at least three users return with a second real artifact.

### Phase 3 — Paid validation (weeks 7–12)

- Add licensing, signed rule updates, custom local rules, and a polished export summary.
- Offer annual Pro purchase to beta users and a small agency plan to agencies with multi-client workflows.
- Publish practical content: “How to safely share logs with AI,” “Safe incident-bundle checklist,” and “What to redact before vendor support.”
- Success threshold: five paid individual customers or two paid agencies without discount-driven pressure.

### Primary acquisition channels

- Free open-source-adjacent CLI and helpful documentation as the trust and discovery surface.
- Founder and developer communities where AI-assisted debugging is common.
- Direct outreach to small agencies that regularly share client logs with third parties.
- Technical content targeting high-intent searches around redacting logs, safely sharing production errors, and preparing support bundles.
- Integration-free referral: every exported summary can include an optional, user-controlled “Prepared with SecretScrub” line only when the user enables it.

## Metrics

### Leading product metrics

- Time from input to exported safe copy.
- Scan completion rate and review-to-export rate.
- Unsupported-file rate and most-requested formats.
- Repeat use within 30 days.
- Custom-rule creation rate.

### Trust metrics

- Percentage of users who understand that processing is local in an onboarding comprehension test.
- False-positive and false-negative reports per detector release.
- Export cancellation rate caused by uncertainty.
- Number of support requests that require original artifacts; target is zero.

### Business metrics

- Beta-to-paid conversion.
- Individual versus agency annual revenue.
- Customer acquisition channel by paid conversion.
- Annual renewal intent and expansion from Pro to Agency.

## Risks and Mitigations

Operational playbook: `ODDS-IMPROVEMENT.md`. Honesty rules: `TRUST.md`. Positioning: `COMPETITIVE.md`. Validation log: `VALIDATION.md`.

| Risk | Severity | Mitigation | Proof we watch |
| --- | --- | --- | --- |
| Users expect perfect detection | Critical | Explicit “review required”; banned marketing claims; export summary states detector scope and pack version | Zero “guaranteed safe” copy; onboarding comprehension |
| False negatives create harm | Critical | Precision-first detectors; FN intake without real secrets; every FN → fixture; no silent rule widening | FN fixture count; time-to-fix for reported misses |
| Enterprise scanners look redundant | High | Own **safe-share export**, not org detection; battle card in every pitch | Win rate when buyer already has GG/TruffleHog |
| Free regex/scripts are “good enough” | High | Beat them on multi-file placeholders, structure preserve, review UX, honest unsupported | Time-to-safe-copy vs script on standard fixture |
| Low-frequency need reduces retention | High | ICP = weekly AI/debug sharers first; profiles + custom rules for return visits | 30-day second-use rate in beta |
| Detector false positives damage trust | Medium | Side-by-side review; one-export exceptions; fixture regression | FP reports per release; export cancel-for-doubt rate |
| Support burden from formats | Medium | Narrow v1 formats; label unsupported instead of unsafe transforms | Unsupported-file rate; support tickets needing originals (target 0) |
| Consumer distribution is hard | Medium | Free CLI + communities + intent content; no app-store dependency early | Channel → beta → paid attribution |
| macOS signing / WebView platform risk | Medium | CLI-first beta; early notarization dry-run; default-deny Tauri capabilities | Signed offline install works before public download |
| Price signal wrong ($39 too low/high) | Low | Interview price ladder; revisit after first 5 paid | Van Westendorp notes; conversion at list price |
| Build before demand | Critical | 15 interviews + gates before heavy desktop spend | Gate table in `ODDS-IMPROVEMENT.md` / `VALIDATION.md` |

## Sources and Evidence

- [OWASP Top 10 for LLM Applications v2.0](https://owasp.org/www-project-top-10-for-large-language-model-applications/assets/PDF/OWASP-Top-10-for-LLMs-v2025.pdf) identifies sensitive-information disclosure as an LLM risk.
- [GitGuardian’s 2026 State of Secrets Sprawl page](https://www.gitguardian.com/waitlist) describes AI adoption and secret-sprawl trends; treat vendor metrics as directional evidence, not independent market sizing.
- [TruffleHog terminology](https://trufflesecurity.com/docs/terminology) documents scanning across logs, configs, filesystems, and other sources, confirming the artifact types that require safe sharing.
- [GitHub Copilot data handling](https://github.com/features/copilot) describes plan-dependent interaction-data handling; SecretScrub’s value does not depend on any one AI provider’s policy because it removes sensitive values before sharing.

## Decision

Build SecretScrub only if private-beta users demonstrate that they repeatedly need to prepare production artifacts for external sharing and will pay for a local-first workflow. The first product goal is not a broad security platform; it is the fastest trustworthy path from “I have a sensitive log” to “I can safely share this useful copy.”
