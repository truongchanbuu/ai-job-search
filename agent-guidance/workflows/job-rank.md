# Job Rank Workflow

Canonical workflow for ranking scraped jobs before applying.

## Steps

1. Load scraped job results and candidate fit criteria.
2. Fetch posting content where available.
3. Score each posting against technical fit, experience fit, behavioral fit, logistics, and career alignment.
4. Exclude deal-breakers and expired postings.
5. Produce a ranked shortlist with reasons, gaps, and recommended next actions.

## Rules

- Ranking is triage only; `job-apply` performs the authoritative evaluation.
- Do not use company research or salary lookup during triage unless explicitly requested.
