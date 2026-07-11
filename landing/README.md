# SecretScrub marketing landing page

Static conversion landing page for SecretScrub (local-first safe-share).

## Design read

**Terminal instrument** — warm dark canvas, editorial serif display (Newsreader),
Source Sans body, IBM Plex Mono for product proof. Brass signal accent (`#D4A017`).
Product demo (terminal chrome) is the hero — not stock photography.

Anti-slop rules applied:
- No soft-blue “trust SaaS” palette
- No purple/pink AI gradients
- No equal feature-card grids as the visual system
- No emoji icons
- Asymmetric problem/FAQ layouts; mono metadata for trust strip

**Dials:** variance 7 · motion 2 · density 4

Design system source of truth: `../design-system/secretscrub/MASTER.md`

## Preview

```bash
# From repo root
open landing/index.html
# or serve locally
npx --yes serve landing
```

Then open the printed URL (usually `http://localhost:3000`).

## Primary conversion goals

1. **Get free CLI** (GitHub) — try before buy, no account  
2. **Request private beta / Pro waitlist** — email capture path  

## Copy principles

- Benefits over features; specific over vague  
- Trust non-claims from product `TRUST.md` (no “guaranteed safe,” no military-grade)  
- One primary CTA intent label: **Get free CLI**  
- Secondary intent: waitlist / beta (not the same as CLI download)  

## Files

| File | Role |
| --- | --- |
| `index.html` | Page structure and copy |
| `styles.css` | Layout, tokens, responsive rules |
| `main.js` | Mobile nav, copy install, year, reduced-motion-safe reveal |
| `assets/hero-desk.jpg` | Optional legacy asset (not used in current hero) |

## Related

- Product PRD: `../.scratch/secretscrub/PRD.md`  
- Trust policy: `../.scratch/secretscrub/TRUST.md`  
- CLI verify guide: `../docs/HOW-TO-VERIFY.md`  
