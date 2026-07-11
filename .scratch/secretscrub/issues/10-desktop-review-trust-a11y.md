# 10 — Desktop review panes, trust chrome, and accessibility

Status: needs-triage  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Complete the private-beta **review** experience: findings rail, side-by-side original vs safe copy (segmented switcher on narrow widths), jump-to-finding, status badges (icon + text), keyboard-complete scan → review → export, reduced-motion respect, and live-region announcements for scan/export/errors. Apply PRD design tokens enough for a calm, evidence-first UI (not terminal cosplay).

End-to-end: keyboard-only user completes export; every redaction is inspectable before export.

## Acceptance criteria

- [ ] Findings rail: type, placeholder, occurrence count, jump to location in preview
- [ ] Wide: synchronized side-by-side panes with line numbers; narrow: original/safe segmented control
- [ ] Safe pane never needs to re-display raw secret outside original pane semantics (placeholders only in safe pane)
- [ ] Status badges: Review required / Structure valid / Unsupported / Safe copy ready — never color alone
- [ ] Full primary workflow keyboard accessible; visible focus rings; 44px primary controls
- [ ] Reduced-motion: nonessential animation disabled
- [ ] Scan/export completion and errors announced appropriately; errors include next recovery step
- [ ] Optional: one-export “keep this finding” with explicit warning (if deferred, document)

## User stories covered

11, 12, 13 (if keep-finding ships), 15, 31, 32

## Blocked by

- `09-desktop-scan-export-mvp`
- `02-placeholders-value-correlation`

## Comments

-
