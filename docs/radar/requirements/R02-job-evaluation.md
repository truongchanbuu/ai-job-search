# R02 — Explainable job evaluation

**Owner:** Evaluator · **Priority:** Must · **Related:** R04, scoring standard

## Requirements

- **FR-010** Apply hard eligibility constraints before weighted ranking (hours, policy, impossible deadline, prohibited work).
- **FR-011** Score technical fit, related evidence, scope clarity, budget/effort, availability, client risk and personal interest (0–100 overall).
- **FR-012** Record score breakdown, confidence, evidence source, assumptions, unknown fields and reasons for each recommendation.
- **FR-013** Support `Apply candidate`, `Review`, `Skip` outcomes; do not equate a high score with consent to apply.
- **FR-014** Evaluate only full job details when available; when missing, flag `needs_review` instead of fabricating detail.
- **FR-015** Respect separate caps for fetched jobs, deep AI evaluations and user notifications.

## Business rules

- **BR-010** Hard constraints veto weighted scores; they cannot be overridden by a model-generated score.
- **BR-011** Estimated effort must be labeled as an estimate, never a confirmed fact.
- **BR-012** Unknown client history, Connects cost or budget reduces confidence; do not assume favorable values.
- **BR-013** Ratings from a different source/version must be reproducible from stored configuration snapshots.

## Acceptance

- **AC-010** A well-matched but 40-hour/week job is rejected when availability is 10 hours/week and overlap is required.
- **AC-011** A job missing budget shows `unknown`, never `$0` or a made-up amount.
- **AC-012** Two runs on identical inputs, rules and config produce the same deterministic base score.
