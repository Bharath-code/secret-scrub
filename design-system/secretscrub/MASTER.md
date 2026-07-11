# Design System Master File

> **PROJECT:** SecretScrub
> **Generated:** 2026-07-11
> **Direction:** Terminal instrument — editorial dark craft for a local-first security CLI

---

## Product Context

- **Type:** Developer tool / local-first privacy preflight
- **Audience:** Indie founders, small product teams, agencies, support engineers
- **Primary CTA:** Get free CLI
- **Secondary CTA:** Pro waitlist / private beta

---

## Visual Direction

| Axis | Choice |
|------|--------|
| Mood | Warm dark, precise, trustworthy — not neon cyberpunk |
| Pattern | Product-demo hero + problem narrative + workflow + proof + pricing |
| Style | Terminal instrument (editorial serif + mono product chrome) |
| Density | Medium-high (4) — technical audience can handle density |
| Motion | Low (2) — subtle reveal only, 150–300ms interactions |

### Avoid (anti-slop)

- Soft-blue “security SaaS” pastels and sky-blue trust theater
- Purple/pink AI gradients
- Equal three-column feature-card grids as the main visual system
- Stock desk photography as the hero
- Emoji icons, playful illustrations
- Oversized border-radius (12px+ everywhere)
- Fake social proof / “military-grade” claims

---

## Color Palette

| Role | Hex | Usage |
|------|-----|-------|
| Background | `#0C0C0D` | Page canvas |
| Elevated | `#121214` | Strips, elevated panels |
| Surface | `#161618` | Cards, nav glass |
| Surface 2 | `#1C1C1F` | Hover fills |
| Border | `#2C2C30` | Hairlines |
| Text | `#F2EFE8` | Primary (warm ivory) |
| Muted | `#9A958C` | Body secondary |
| Faint | `#6B675F` | Meta / mono labels |
| Accent (brass) | `#D4A017` | CTA, signal, section labels |
| Accent ink | `#1A1405` | Text on accent buttons |
| Safe / OK | `#6FBF7A` | Placeholders, success status |
| Danger | `#E07A6A` | Secret tokens in demos |
| Warning | `#E0A454` | Findings count |

---

## Typography

| Role | Family | Notes |
|------|--------|-------|
| Display | **Newsreader** | Editorial authority; italic for emphasis |
| Body | **Source Sans 3** | Readable UI sans |
| Mono | **IBM Plex Mono** | CLI, install bar, meta labels |

### Scale

- Hero H1: `clamp(2.35rem, 5vw, 3.6rem)`, weight 600, tracking −0.025em
- Section H2: `clamp(1.85rem, 3.2vw, 2.55rem)`, weight 600
- Body: 17px / 1.6
- Labels: mono 0.72rem, uppercase, letter-spacing 0.08–0.1em

---

## Layout

- Max width: `1120px`
- Floating nav: inset shell with blur, not full-bleed sticky bar
- Hero: 2-col desktop (copy + terminal), stacked mobile
- Problem / FAQ: sticky left head + content stack on large screens
- Steps / audience: bordered lists, not soft card grids
- Radius: 4–10px max (sharp craft, not bubbly)

---

## Components

### Buttons

- Primary: brass fill, dark ink text, 44px min height
- Secondary: transparent + strong border
- Ghost: muted text for nav GitHub
- Hover: color shift only — no layout-shifting scale on hover (active scale 0.98 ok)

### Terminal / demo

- Window chrome with traffic lights
- Mono body; safe tokens in green soft highlight; secrets in danger soft highlight
- Status pills in footer

### Install bar

- Mono command + copy button (clipboard API with fallback)

### Pricing

- Featured card: brass border + soft gradient, “Recommended” badge
- Full-width CTAs inside cards

---

## Effects

| Effect | Spec |
|--------|------|
| Nav glass | `backdrop-filter: blur(16px)`, bg `rgb(18 18 20 / 0.82)` |
| Shadows | Deep soft (`0 24px 64px rgb(0 0 0 / 0.45)`) on product panels only |
| Transitions | 150–200ms, `cubic-bezier(0.22, 1, 0.36, 1)` |
| Reveal | opacity + 12px translateY; disabled when `prefers-reduced-motion` |

---

## Accessibility

- Focus: 2px solid brass outline, 3px offset
- Touch targets ≥ 44px
- Contrast: ivory on near-black; muted text ≥ slate-level for body secondary
- Skip link present
- FAQ via native `<details>`
- No emoji icons — SVG only

---

## CTA Strategy

1. Sticky nav: Get free CLI  
2. Hero: Get free CLI + See the demo  
3. Install bar: copy cargo install  
4. Pricing cards: CLI / waitlist / talk  
5. Final panel: CLI + private beta  

---

## Hierarchical Override

When building a page:

1. Check `design-system/secretscrub/pages/[page].md`
2. If present, its rules override this file
3. Else use this MASTER exclusively
