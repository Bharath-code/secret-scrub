# SecretScrub — Local Safe-Share Workspace

Status: needs-triage

## Problem Statement

Developers, founders, and support engineers need to share production logs, configuration files, error reports, and code snippets with AI tools, vendors, or teammates. Those artifacts often contain API keys, passwords, tokens, email addresses, IP addresses, and customer identifiers.

Today the user either performs slow, error-prone manual redaction or avoids sharing useful debugging context. Existing secret-scanning products are optimized to detect and remediate leaks in repositories and delivery pipelines; they do not make a ready-to-share, structurally valid copy of an artifact the primary outcome. The result is delayed debugging, accidental disclosure risk, and lost trust.

## Solution

SecretScrub is a local-first desktop application and companion CLI that converts sensitive technical artifacts into safe-to-share copies. The user drops, pastes, or selects an artifact; SecretScrub detects sensitive values, replaces them consistently with semantic placeholders, preserves useful debugging structure, previews the result, and exports a sanitized copy.

The product promise is: **share production context in seconds without sending the original artifact to a third party.** All scanning and transformation happen on the user’s device in the initial release.

## Design System

### Product principles

- **Trust is visible.** Every workspace makes local-only processing, pending review, and export safety legible without relying on a settings page or marketing copy.
- **Evidence stays readable.** Code, logs, findings, and structural validation are primary content; decoration must never compete with them.
- **Safe actions are obvious; destructive actions are recoverable.** The primary action is export, originals are never modified, and clearing a workspace requires an understandable consequence.
- **Minimal, not empty.** Use generous whitespace and a single dominant action, while keeping the context needed to make a safe decision within one screen.

### Visual language

Use functional, high-contrast minimalism in the desktop product. Subtle glass may be used on the marketing site only. The product itself must remain crisp, dense enough for technical artifacts, and free from decorative gradients, glassmorphism, terminal cosplay, noisy charts, or animated security theatrics.

Use Lucide or another single SVG icon set. Do not use emoji as interface icons.

### Color tokens

| Token | Value | Use |
| --- | --- | --- |
| Canvas | `#F0F9FF` | Application background |
| Surface | `#FFFFFF` | Cards, dialogs, code panes |
| Surface subtle | `#E0F2FE` | Selected rows and low-emphasis grouping |
| Primary | `#0369A1` | Primary actions, active focus, key navigation |
| Primary hover | `#075985` | Hover and pressed primary actions |
| Secondary | `#0EA5E9` | Informational accents and progress |
| Protected accent | `#22C55E` | Success icons, progress, and non-text protected accents |
| Protected accessible | `#15803D` | Success text and controls that require contrast against a light surface |
| Warning | `#B45309` | Review-required and partial-support states |
| Critical | `#B42318` | Blocking errors and unsafe export states |
| Text strong | `#0C4A6E` | Headings and high-emphasis text |
| Text muted | `#475569` | Supporting text and metadata |
| Border | `#BAE6FD` | Dividers, inactive inputs, and pane boundaries |

Color must never be the sole status signal: pair it with a text label and a distinct icon. Normal body text must meet a 4.5:1 contrast ratio. Green means a completed, reviewed export state; it must not imply that every possible sensitive value was detected.

### Typography and layout

- Use **Satoshi** for headings and high-emphasis interface labels.
- Use **General Sans** for body and code-adjacent interface text, with **DM Sans** as the fallback. Use a 16px base body size, 1.5–1.6 line height, 14px metadata, and 20–28px screen titles.
- Use **JetBrains Mono** only for raw logs, source text, placeholders, file paths, detection details, and CLI output. Default code size is 13–14px with a 1.55 line height.
- Use a 4px spacing scale; common gaps are 8, 12, 16, 24, 32, and 48px.
- Use 12px rounded surfaces, 8px controls, 1px visible borders, and subtle shadows only for overlays.
- Keep primary desktop content to a comfortable reading width. Review mode may use the full window width for side-by-side artifact comparison.
- Ensure the product remains usable at 375px, 768px, 1024px, and 1440px. On narrow displays, replace the split review pane with a segmented original/safe-copy switcher.

### Component rules

| Component | Required behavior |
| --- | --- |
| Primary button | 44px minimum height; clear verb such as “Scan locally” or “Export safe copy”; disabled with progress feedback while work is active. |
| Drop zone | Entire area is clickable and keyboard reachable; visibly states supported inputs and “processed locally.” |
| Findings row | Includes detector type, semantic placeholder, confidence/review state, occurrence count, and a direct jump to the relevant preview location. |
| Review panes | Monospaced content, synchronized line navigation where possible, fixed line-number gutter, and semantic redaction highlighting that does not expose the original value outside the original pane. |
| Status badge | Icon plus text; use “Review required,” “Structure valid,” “Unsupported,” or “Safe copy ready,” never color alone. |
| Dialog | Used only for destructive clear, export-location conflict, and custom-rule decisions; focus is trapped appropriately and return focus is predictable. |
| Toast | Used only for nonblocking confirmations; blocking scan or export failures remain adjacent to the affected action and are announced to assistive technology. |

### Interaction, motion, and accessibility

- All actions are keyboard accessible; tab order follows the visible scan → review → export workflow. Provide visible focus rings using the primary color.
- Use labels rather than placeholder-only inputs and accessible names for all icon-only controls.
- Announce scan completion, export completion, and errors through an appropriate live region. Error messages must explain the next recovery action.
- Use a progress bar and processed-file count for operations over 300ms. Preserve layout space so scanning does not cause content to jump.
- Use only opacity and transform for micro-interactions, with a 150–250ms duration. Respect reduced-motion preferences by removing nonessential animation.
- Do not allow an export button to trigger more than once while an export is active. Cancellation must be reachable by keyboard and must clearly report whether temporary output was removed.

## UI and UX Flow

### 1. First-run trust and input flow

1. The user opens SecretScrub and sees one dominant action: **Drop files, select a folder, or paste text**.
2. The input surface names supported formats and shows a persistent “Processed locally — nothing uploaded” trust message.
3. The user selects a recipient profile: AI prompt, vendor support, or incident bundle. The product shows the profile’s high-level protection scope without forcing rule configuration.
4. The user starts the scan with a primary **Scan locally** action. A nontechnical user can complete this flow without learning a rule language.

### 2. Scan-progress flow

1. The scan view replaces the primary action with a progress bar, current file name, processed-file count, elapsed time, and a visible **Cancel** action.
2. The original input remains untouched throughout processing.
3. For a completed scan, the product transitions directly to review with a concise result: findings count, supported/unsupported file count, and structural-validation status.
4. If scanning fails or encounters an unsupported format, the result names the affected artifact, explains why it cannot be marked safe, and offers retry, exclude-from-export, or return-to-workspace actions.

### 3. Review-and-decision flow

1. The review screen has a concise header: artifact name, recipient profile, local-only indicator, and export safety status.
2. A findings rail groups results by type and severity. Selecting a finding moves both preview panes to the relevant occurrence.
3. The main area shows original content and safe copy side by side on wide screens; narrow screens use a clearly labeled segmented switcher.
4. The safe pane shows semantic placeholders such as `[STRIPE_SECRET#1]`, preserving repeated-value correlation without revealing the source value.
5. The user can mark a finding as “keep for this export” only after seeing an explicit warning; the scope is one export, not a silent global detector change.
6. Unsupported or partially transformed files keep the overall status at **Review required** and prevent a misleading “safe” claim.

### 4. Export flow

1. Once supported artifacts have been reviewed, the user chooses **Export safe copy**.
2. The export dialog lets the user choose destination, naming policy, and whether to include a plain-language safety summary. It never proposes overwriting the original input.
3. During export, the action is disabled, progress remains visible, and cancellation is available.
4. Completion shows the output location, the number and types of replacements, structural-validation status, and a single action to reveal the exported copy in the operating system.
5. The workspace then offers **Start another scrub** or **Clear original workspace**; clearing explains that temporary original content will be removed from the application workspace.

### 5. Rules and recovery flow

1. Advanced users reach rules through a secondary settings area, never as a requirement in the main workflow.
2. Custom rules show a human-readable name, match scope, sample redacted output, and validation result before being enabled.
3. Rule-pack updates display version, signature status, and a readable changelog; they do not transmit local artifacts.
4. A failed export or unreadable file always preserves the completed review workspace and gives a concrete next step instead of discarding work.

### Primary-screen information architecture

| Screen | User question answered | Primary action |
| --- | --- | --- |
| Home workspace | “What can I safely scrub?” | Add files, folder, or pasted text |
| Scan progress | “Is my artifact being processed locally?” | Cancel scan |
| Review | “What changed, and is the result safe to share?” | Export safe copy |
| Export complete | “Where is my safe artifact and what was protected?” | Reveal export or start another scrub |
| Rules | “How do I protect my organization-specific values?” | Validate and enable a custom rule |

## User Stories

1. As a developer, I want to drop a log file into SecretScrub, so that I can create a shareable copy without manually editing it.
2. As a developer, I want to paste a stack trace directly into SecretScrub, so that I can sanitize a small incident artifact immediately.
3. As a developer, I want to scan a folder of logs, so that I can prepare a complete support bundle in one operation.
4. As a developer, I want detected AWS, GitHub, Stripe, OpenAI, JWT, and generic API secrets to be labeled by type, so that I understand what was protected.
5. As a developer, I want the same original value to receive the same placeholder everywhere, so that the causal relationships in a log remain debuggable.
6. As a developer, I want placeholders such as `[STRIPE_SECRET#1]` instead of an anonymous mask, so that I can distinguish different values without revealing them.
7. As a developer, I want emails, IP addresses, and configurable customer identifiers redacted, so that I do not expose personal or customer information.
8. As a developer, I want valid JSON to remain valid JSON after sanitization, so that I can share it with tools that parse structured data.
9. As a developer, I want valid YAML and environment files to retain their useful structure, so that a recipient can understand configuration context.
10. As a developer, I want raw stack traces, timestamps, error codes, and request relationships retained by default, so that redaction does not destroy debugging value.
11. As a developer, I want a clear side-by-side original and safe-copy preview, so that I can inspect the transformation before export.
12. As a developer, I want every redaction shown as a reviewable finding, so that I can catch false positives and adjust a rule when needed.
13. As a developer, I want to exclude a finding for this export without changing other projects, so that one-off safe values do not block my workflow.
14. As a developer, I want to choose a named rule profile such as "AI prompt", "vendor support", or "incident bundle", so that redaction fits the recipient and purpose.
15. As a developer, I want an unambiguous local-processing indicator, so that I know the original content is not uploaded.
16. As a privacy-conscious user, I want SecretScrub to work offline, so that sensitive artifacts never require a network connection.
17. As a user, I want the application to avoid retaining my original artifacts after I close or clear a workspace, so that local risk is minimized.
18. As a user, I want to export sanitized files to a chosen destination, so that I control where shareable copies are stored.
19. As a user, I want the export to include a concise safety summary, so that I can confidently state what was redacted.
20. As a user, I want an obvious warning when a file type cannot be safely transformed, so that I do not mistake an incomplete scan for a safe artifact.
21. As a user, I want large files processed without the application becoming unresponsive, so that I can use the product during an incident.
22. As a user, I want cancellation to leave original input untouched and avoid partial exports, so that I can recover safely from a mistaken operation.
23. As a CLI user, I want to sanitize standard input or a specified path, so that I can use SecretScrub in scripted support workflows.
24. As a CLI user, I want a machine-readable findings summary and meaningful exit status, so that automation can decide whether a review is required.
25. As a team lead, I want to distribute signed rule-pack updates, so that the team recognizes new common token formats without sending artifacts to a service.
26. As a team lead, I want to maintain custom local rules for company-specific identifiers, so that SecretScrub protects data that generic detectors cannot know.
27. As a security-conscious team lead, I want custom rules to be visible and reviewable, so that an overbroad rule does not silently remove important evidence.
28. As a buyer, I want a free CLI for trying standard protection, so that I can validate detection before purchasing the desktop workflow.
29. As a Pro customer, I want a polished desktop review and export experience, so that safe sharing is faster than manual redaction.
30. As a support engineer, I want a consistent safe artifact format, so that I can receive useful context without requesting repeated edits.
31. As a keyboard user, I want the entire scan-review-export flow to be keyboard accessible, so that the tool is fast during a time-sensitive incident.
32. As a user with reduced motion enabled, I want the interface to avoid nonessential animation, so that the application remains comfortable and accessible.
33. As a privacy-conscious user, I want to run SecretScrub without creating an account, so that my local artifacts are not tied to a hosted identity.
34. As a Pro user, I want my license to work offline after activation, so that incident response does not depend on internet access.
35. As a user, I want SecretScrub to clearly identify the files excluded from an export and why, so that I do not mistakenly share an incomplete bundle.
36. As a user, I want an interruption-safe export, so that a crash, cancellation, or full disk does not leave a misleading partial safe bundle.
37. As a user, I want a signed app and signed rule-pack updates, so that I can verify the software and detectors have not been altered.
38. As a user, I want an opt-in, sanitized diagnostic report, so that I can receive support without sending artifact contents or secret values.
39. As a user on a supported operating system, I want native installation and predictable updates, so that the application feels reliable rather than like a browser utility.

## Implementation Decisions

- Ship a local-first desktop product backed by a companion CLI. The first release must not transmit input artifacts, raw findings, or derived content to any hosted service.
- Treat transformation, not detection alone, as the core product outcome: users receive an exportable safe copy, a findings summary, and a review surface.
- Build a deep **redaction engine** with a stable interface that accepts artifact content, rule configuration, and an output policy; it returns a transformed artifact, typed findings, structural-validation results, and an export-safety status. It must be streaming-capable so large inputs are not fully held in memory.
- Build a deep **rule-pack module** that classifies built-in and custom detectors, assigns semantic placeholder types, handles value correlation within one workspace, and validates signed updates. The engine consumes this module through a narrow rule-evaluation interface.
- Build a deep **structured-artifact transformer** that preserves validity for supported structured formats and explicitly reports unsupported or partially supported formats. Unsupported formats must never be silently marked safe.
- Build an **artifact workspace module** that owns input selection, temporary handling, preview state, cancellation, and export. It must keep originals immutable and clear temporary data when the user clears or closes a workspace.
- The desktop UI is command-first: one primary input action, a visible local-only trust indicator, a results summary, a side-by-side review, and a single export action. The UI must not resemble a dense security-operations dashboard.
- The CLI supports path and standard-input modes, noninteractive export, machine-readable summaries, and exit states that distinguish clean completion, review-required completion, unsupported input, and execution failure.
- Initial supported inputs are plain text, log files, environment files, JSON, YAML, and source-code-like text. Initial detection includes common provider secrets, JWTs, generic key/value secrets, emails, and IP addresses.
- Avoid network-based secret verification in the first release. SecretScrub identifies and redacts candidate sensitive values; it does not determine whether a credential is active.
- The initial commercial model is a free, limited CLI; an annual individual Pro desktop license; and an annual team policy-pack license. There is no hosted artifact storage tier in this PRD.

## Technical Stack

| Area | Decision | Purpose |
| --- | --- | --- |
| Core runtime | Stable Rust | One shared, memory-safe implementation of scanning, transformation, export, and CLI behavior. |
| Desktop shell | Tauri v2 | Small native desktop distribution with a Rust command boundary and system WebView. |
| Desktop UI | React and TypeScript | Productive implementation of the command-first, accessible review interface. The UI remains a thin client over the Rust core. |
| CLI | Rust with Clap | A standalone binary with shell-friendly help, completion, structured output, and stable exit states. |
| Async and I/O | Tokio with bounded worker queues | Responsive UI and streaming file work without unbounded memory or task creation. |
| CPU-parallel work | Bounded Rayon pool where profiling justifies it | Parallel folder scanning while preserving deterministic output and responsive cancellation. |
| Detection | Rust regular expressions plus Aho-Corasick-style literal matching | Linear-time, versioned built-in detectors; avoid regex engines vulnerable to catastrophic backtracking. |
| Structured formats | Serde-based JSON/YAML/TOML parsing and serialization | Preserve valid supported structures after redaction. |
| Local state | SQLite | Store only preferences, rule metadata, update metadata, and non-sensitive export summaries; never persist raw artifacts or secret values. |
| Cryptography | RustCrypto primitives and Ed25519 signatures | Verify rule packs, application update metadata, and offline license artifacts. |
| Secure temporary and export handling | OS-private temporary directories and atomic filesystem operations | Keep originals isolated and ensure only complete safe artifacts appear at the destination. |
| Logging | Structured local diagnostic events | Make failures diagnosable without recording artifact content, secret values, or unredacted snippets. |

The initial release has **no application backend**. Billing, license issuance, and release hosting may be external operational services, but the desktop product must remain fully functional for scanning, reviewing, and exporting while offline after activation.

## Application Architecture and Data Lifecycle

### Core boundaries

- **Core domain library:** owns content classification, candidate detection, placeholder allocation, rule evaluation, structured transformation, export-safety decisions, and typed findings. Both desktop and CLI consume this stable API; neither reimplements redaction behavior.
- **Input and workspace adapter:** enumerates selected files, enforces size and recursion limits, creates private temporary workspace state, streams content into the core library, and coordinates cancellation.
- **Rule-pack service:** loads built-in rules, validates custom rules, verifies signed updates, and exposes only compiled detector policy to the core domain library.
- **Export service:** writes a new safe bundle to a user-chosen destination, validates supported transformed artifacts, creates an optional safety summary, and uses atomic completion semantics.
- **Desktop adapter:** exposes the smallest possible set of typed Rust commands needed by the interface and renders UI state. It does not receive broad filesystem or shell privileges.
- **CLI adapter:** maps arguments, standard input, exit states, and machine-readable output to the same core domain library.

### Artifact data lifecycle

1. The user explicitly selects an artifact, folder, or pasted text.
2. The input adapter creates a private in-memory or OS-private temporary workspace and records only the minimum metadata required for review.
3. The core library classifies and scans content in streams, retaining original values only for the lifetime of the active workspace and only as needed for side-by-side review and consistent placeholder allocation.
4. The export service writes only transformed content and a non-sensitive summary to the selected destination.
5. Clearing the workspace removes temporary originals, in-memory value mappings, and preview state. Persistent local state retains no raw content, raw findings, or secret values.

### Supported platform strategy

- The first desktop release targets macOS 14+ to keep installation, permissions, and support manageable for a small team.
- The Rust core, CLI, and Tauri shell must remain portable by design; Windows and Linux desktop packages follow once macOS workflow, detector quality, and support operations are proven.
- The CLI is distributed independently from the desktop app for developer and CI-adjacent local workflows, but cloud CI integration remains out of scope for the first release.

## Security, Privacy, and Release Decisions

- Default-deny the desktop command boundary. Tauri capabilities and permissions must expose only the specific commands and filesystem scopes required for the active workspace.
- Ship bundled local assets only. Set a restrictive content-security policy; do not load remote scripts, fonts, analytics, or arbitrary web content in the desktop application.
- Do not execute shell commands, evaluate user-supplied code, or follow unbounded symbolic links while processing input.
- Treat archive extraction, malformed structured files, deeply nested inputs, oversized lines, and intentionally adversarial detector inputs as hostile. Enforce explicit depth, size, time, and memory limits with clear review-required outcomes.
- Generate placeholders deterministically only within an active workspace. Do not make them stable across separate scrubs, because cross-export correlation could create unintended privacy leakage.
- Use private temporary directories, restrictive file permissions, and atomic destination writes. Never overwrite the source artifact; require explicit resolution for an existing destination conflict.
- Do not collect telemetry by default. If a user opts into diagnostics, include product version, supported platform metadata, non-sensitive error category, and rule-pack version only—never paths, artifact names, content, findings, or secret fingerprints.
- Sign production installers and update metadata. Rule-pack and offline-license artifacts must be signature-verified before use and retain an auditable version identifier.
- Use reproducible dependency locking, automated vulnerability checks, license-policy checks, and a software bill of materials for every release candidate.
- Provide an in-product privacy statement that explains exactly what remains local, what optional update or licensing network calls occur, and how the user can disable those calls.

## Delivery, Operations, and Commercial Readiness

### Release phases

| Phase | Included outcome | Explicit boundary |
| --- | --- | --- |
| Private beta | macOS desktop, text/log/JSON/YAML/environment inputs, built-in rules, local review, safe-copy export, free CLI | No account, cloud storage, remote scanning, or automatic update checks. |
| Version 1 Pro | Offline license, signed rule-pack updates, custom local rules, folder bundles, safety summary, polished accessibility, crash-safe export | No team console, SSO, hosted policy management, or artifact retention. |
| Later validation | Windows/Linux packages, optional organization policy distribution, more structured formats, image/OCR redaction | Build only after repeated customer demand and security review. |

### Release and support process

- Use a protected release pipeline that runs formatting, linting, unit tests, integration tests, property tests, end-to-end UI tests, dependency checks, and packaging checks before signing an installer.
- Test installers on a clean supported operating-system image and verify that the application starts without a network connection.
- Publish checksums, signatures, a human-readable changelog, supported-platform matrix, and upgrade notes with every release.
- Maintain a detector-rule changelog that states added patterns, changed false-positive behavior, and any required customer review.
- Support uses sanitized diagnostic bundles only. A support request must never require the customer to send an original artifact to SecretScrub’s operators.

## Testing Decisions

- Tests must assert externally observable behavior: output content, structural validity, findings, safety status, cancellation behavior, and CLI exit behavior. They must not assert internal implementation or traversal order unless order is part of the public contract.
- The redaction engine will have extensive fixture-based tests covering token detection, consistent placeholders, overlapping candidates, false-positive exclusions, large streamed inputs, and unchanged non-sensitive content.
- The rule-pack module will have isolated tests for rule precedence, custom-rule validation, semantic placeholder allocation, signature validation, and compatibility between a rule pack and the engine.
- The structured-artifact transformer will have contract tests confirming that supported JSON and YAML remain parseable and that unsupported structures surface an explicit non-safe status.
- The artifact workspace will have integration tests covering import, scan, review, cancel, clear, and atomic export behavior without modifying the original input.
- The CLI will have black-box tests for path input, standard input, JSON summaries, output destinations, and exit statuses.
- The desktop UI will have accessibility tests for keyboard completion of the primary workflow, visible focus, semantic labels, loading/error feedback, and reduced-motion behavior.
- Use property tests and fuzzing for detector overlap, placeholder allocation, malformed structured input, Unicode boundaries, deeply nested documents, and cancellation at arbitrary streaming points.
- Use golden fixtures for representative incident artifacts to confirm that safe copies preserve the required debugging signal while replacing sensitive values consistently.
- Run desktop end-to-end tests against the packaged command boundary, not mocks of the redaction engine, to verify input selection, review state, export, cancellation, and permission failures.
- Add security regression tests for path traversal, symbolic-link handling, destination conflicts, CSP configuration, denied command access, signature rejection, and diagnostic-content exclusion.
- Add performance regression tests for representative single-file and folder-bundle inputs. The desktop UI must remain responsive while the scanner streams large inputs.
- The repository has no existing test prior art. Establish fixture-driven behavior tests and black-box CLI tests as the baseline for future modules.

## Out of Scope

- Cloud scanning, hosted artifact storage, organization-wide artifact retention, and server-side content analysis.
- Claiming that every sensitive value can be detected or that an exported artifact is legally or universally safe to disclose.
- Automatic revocation, rotation, or validation of discovered credentials.
- Repository, CI/CD, source-control, chat, cloud-storage, or SaaS-platform integrations.
- Binary document transformations, image OCR redaction, video redaction, and screenshot capture workflows.
- Full data-loss-prevention, compliance-management, security-information, or incident-management platforms.
- Real-time collaborative editing, remote policy administration, SSO, and audit trails beyond local export summaries.

## Further Notes

- The product’s differentiation is the safe-share workflow: scan, review, preserve context, and export. It should not compete head-on on number of detectors with enterprise secret-scanning platforms.
- The first success metric is time from dropped artifact to reviewed safe copy: under ten seconds for representative text-based incident files on a typical developer machine.
- The key trust metric is that users can verify the application processed artifacts locally and can understand what was changed before export.
- The visual direction is calm and evidence-oriented: security blue, protected green, high-contrast text, clear code comparison, and restrained motion. Avoid terminal cosplay, alarm-heavy dashboards, and decorative effects in the core workflow.
