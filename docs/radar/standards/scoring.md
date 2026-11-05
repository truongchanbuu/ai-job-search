# Standard — job scoring & transparency

**Editable numeric defaults:** `config/scoring.example.yaml`. This document defines meaning, not currently active numeric values.

## Evaluation order

1. Check source trust, job completeness and already-applied state.
2. Run hard vetoes (`policy_prohibited`, `over_capacity`, `unrealistic_required_availability`, `unsupported_skill_requirement`, `user_blocked_category`).
3. Calculate weighted scores only for eligible jobs.
4. Produce `score`, `verdict`, `confidence`, `risk`, `strengths`, `gaps`, `unknowns`, `config_revision`.
5. Emit `JobMatched` only when allowed by notification routing and per-day caps.

## Signals

| Signal | Interpretation |
|---|---|
| Technical | Can the user deliver the required stack, without pretending mastery? |
| Experience | Evidence from genuine projects; distinguish employment, personal, academic and training |
| Scope | Can clear, testable deliverables be inferred? |
| Budget/effort | Effective compensation after effort estimation, uncertainty and platform fees |
| Availability | Can work be performed outside full-time job commitments? |
| Client risk | Verified marketplace signals, ambiguity, potentially unreasonable requests |
| Interest | User's explicit likes/dislikes and preferred work types |

## Rules

- Unknown is not zero and not perfect. Store both factual score inputs and missing-data flags.
- A `hard_veto` always wins over overall weighted score.
- Base numeric score is deterministic; optional AI rationale must not silently mutate it.
- Scores are **decision aids**, not promises of successful hiring.
- Feedback may propose weight changes, but changes require owner acceptance and revision tracking.
