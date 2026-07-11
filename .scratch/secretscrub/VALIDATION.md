# SecretScrub — Customer Validation Log

Status: ready-for-agent

Goal: prove demand **before** deep desktop investment. Target: **15 interviews in 14 days**.

---

## Pass / fail (from business plan + odds plan)

| Gate | Pass |
| --- | --- |
| Pain | ≥8 of 15 describe manual redaction or avoidance as recurring |
| Beta | ≥5 agree to test a private beta with a real artifact |
| Frequency | ≥5 encounter the problem at least monthly (weekly is stronger) |
| Pay signal | ≥3 say ~$39/year is plausible **or** ≥1 pre-pay/intent |

Fail any gate → do not expand beyond CLI spike; re-segment ICP or kill.

---

## Who to talk to (priority order)

1. Indie SaaS founders / solo technical founders who paste into ChatGPT/Claude when prod breaks  
2. Agency engineers who send client logs to hosts/plugins/payment vendors  
3. Support/solutions engineers who prepare vendor escalation bundles  

**Avoid for first 15:** enterprise AppSec buyers shopping DLP/SSO.

### Outreach sources

- Personal network (founders you already know)  
- Indie hacker / founder Slack-Discord communities  
- Agency owners via LinkedIn (short DM, not pitch deck)  
- People who recently posted “can’t paste this log into AI”

### Outreach template (short)

> Quick ask: when something breaks in prod, how do you share logs with AI or a vendor without leaking keys? Building a local tool for that — 15 min, no sales call. Open this week?

---

## Interview script (one question at a time; 15–20 min)

### Context (2 min)

- What do you build? Team size? Do you use AI tools for debugging?

### Last incident story (core — 8 min)

1. Tell me about the **last time** you hesitated to share a log, config, or stack trace.  
2. Who was the recipient (AI / vendor / teammate / client)?  
3. What did you fear was inside it?  
4. What did you actually do (manual redact, strip everything, share raw, refuse)?  
5. Roughly how long did that take?  
6. What broke about that process (time, uncertainty, incomplete context for the other side)?

### Frequency and alternatives (3 min)

7. How often does this come up (weekly / monthly / rare)?  
8. Have you tried any tool, script, or scanner for this? What failed?  
9. If a **local** app did drop → review → export with placeholders, would you try it on a real artifact?

### Willingness (2 min)

10. Would a free CLI be enough, or do you need a review UI?  
11. Annual price check: $0 / $39 / $79 / $149 — where does it get uncomfortable?  
12. What would make you **not** trust a tool like this?

### Close

13. Can I follow up with a private beta in 2–4 weeks?  
14. Anyone else who hits this weekly?

**Do not** demo vaporware for more than 60 seconds. Prefer their story over your slides.

---

## Interview log

Copy a block per conversation.

### Interview template

```
ID: I-XX
Date:
Name / role:
Company type: indie | agency | support | other
ICP fit: strong | medium | weak
Channel: network | community | cold | other

Story summary (recipient, fear, what they did, time):

Frequency: weekly | monthly | rare | never
Pain recurring?: yes | no
Would beta-test with real artifact?: yes | no | maybe
Pay signal: none | free-only | ~$39 ok | higher ok | unclear
Trust concern (quote if possible):
Alternative they use today:
Memorable quote:
Follow-up / beta email:
Notes:
```

### Log entries

<!-- Add I-01 … I-15 below -->

### Summary table

| ID | ICP | Pain? | Frequency | Beta? | Pay signal | Key quote |
| --- | --- | --- | --- | --- | --- | --- |
| I-01 |  |  |  |  |  |  |

### Gate score (update after 15)

| Metric | Count | Pass? |
| --- | --- | --- |
| Recurring pain | /15 |  |
| Beta yes | /15 |  |
| Monthly+ frequency | /15 |  |
| Pay ~$39+ | /15 |  |

**Decision:** proceed / narrow ICP / kill  
**Date:**  
**Rationale:**

---

## Synthetic artifact kit (for demos after interviews)

Use only fake values in demos and fixtures:

- AWS-like key shape that is clearly fake  
- Stripe `sk_test_…` style test keys  
- JWT with dummy payload  
- emails `@example.com`  
- RFC 5737 documentation IPs  

Never accept a user’s real production secret into your own chat logs or this repo.
