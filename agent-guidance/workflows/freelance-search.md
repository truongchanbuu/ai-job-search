# Freelance Search Workflow

Canonical workflow for discovering and triaging freelance opportunities.

## Inputs

- Default candidate profile with verified skills and experience.
- Freelance profile extension with target services, rates, availability, portfolio links, and exclusions.
- Fiverr and Upwork account readiness notes.
- Source skills such as `freelance-search`, `upwork-search`, `fiverr-search`, and `google-search`.

## Steps

1. Load the default profile and freelance profile extension.
2. Check Fiverr and Upwork readiness before platform-specific action.
3. Search enabled freelance sources and collect source statuses.
4. Treat Google, social, and platform links as discovery leads until original content is verified or user-supplied.
5. Normalize and deduplicate opportunities.
6. Rank by skill fit, budget/rate fit, timeline, readiness, language, and exclusions.
7. Flag scam-like, unpaid, non-freelance, stale, private, or low-confidence opportunities.
8. Present proposal-worthy opportunities and next actions.

## Rules

- Do not store platform passwords, payment credentials, private messages, or secret tokens.
- Do not invent freelancer facts, client facts, project requirements, platform ratings, or earnings.
- Do not draft ready-to-send proposals from unverified discovery leads.
- Keep generated state and proposal material in gitignored personal-output paths.

## Account Readiness

- Upwork: record profile completeness, verification status, service categories, and known blockers such as Connects balance.
- Fiverr: record seller profile completeness, gig/service readiness, portfolio references, and setup blockers.
- Unknown readiness should produce `setup_first` or caveated recommendations, not `proposal_worthy`.

## Mixed-Source Ranking

High-priority opportunities should match target services and verified skills,
have acceptable budget/timeline fit, avoid high-risk flags, and have enough
verified source context for proposal drafting.
