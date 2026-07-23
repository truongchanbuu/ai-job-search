# Quickstart: Freelance Job Search

This guide describes validation scenarios for the freelance job search feature. It assumes implementation tasks have added the planned skills, workflow guidance, and file-backed records described in [plan.md](./plan.md), [data-model.md](./data-model.md), and [contracts/](./contracts/).

## Prerequisites

- A default profile exists with verified skills and experience.
- A freelance profile extension exists with target services, rate preferences, availability, portfolio references, and exclusions.
- Fiverr and Upwork account readiness notes exist or are intentionally marked `not_configured`.
- At least one source can provide fixture or live opportunity data.
- Personal outputs are written only to gitignored paths.

## Validation Scenario 1: Search Across Freelance Sources

1. Load the default profile and freelance profile extension.
2. Enable Upwork, Fiverr, Google public-page discovery, and one user-supplied/social source.
3. Run a profile-driven freelance search with a small per-source limit.
4. Confirm the output includes source statuses for every enabled source.
5. Confirm results include title, platform/source, source reference, budget or unknown marker, language, skill matches, risk flags, discovered date, and verification status.
6. Confirm unavailable or setup-required sources do not block results from available sources.

Expected outcome: The user receives a ranked shortlist from at least two available source categories within the success criteria window when source data is available.

## Validation Scenario 2: Account Readiness Blocks Unsafe Action

1. Mark Fiverr as `not_configured`.
2. Mark Upwork as `limited` with a known blocker such as unknown Connects balance.
3. Run a search that returns Fiverr and Upwork opportunities.
4. Confirm Fiverr opportunities recommend setup before action.
5. Confirm Upwork opportunities show the blocker and do not recommend immediate proposal submission unless readiness supports it.

Expected outcome: 100% of Fiverr and Upwork opportunities show readiness or setup-required status before action is recommended.

## Validation Scenario 3: Discovery Lead Verification

1. Add a Google-discovered or social-media opportunity as a discovery lead.
2. Run opportunity fit evaluation.
3. Confirm confidence is limited and proposal material cannot be marked ready.
4. Provide original posting details or user-supplied verified content.
5. Re-run detail parsing and fit evaluation.

Expected outcome: Discovery leads remain clearly marked until original content is verified or supplied by the user.

## Validation Scenario 4: Fit Ranking and Risk Handling

1. Prepare a fixture set with high-fit, low-budget, scam-like, duplicate, and unrelated full-time results.
2. Run ranking against the freelance profile extension.
3. Confirm full-time jobs and irrelevant ads are filtered or flagged.
4. Confirm scam-like and unpaid items are not proposal-worthy.
5. Confirm the top results match target services and profile skills.

Expected outcome: At least 80% of top-ranked opportunities match profile skills or target services, and high-risk items are not recommended as proposal-worthy.

## Validation Scenario 5: Proposal Draft and History

1. Select a verified high-fit opportunity.
2. Generate a proposal draft or response outline.
3. Confirm the draft uses only verified profile facts and verified opportunity details.
4. Save the opportunity with status `proposal_drafted`.
5. Run the same search again and confirm the existing record status appears.
6. Update the record to `applied`, `won`, `lost`, or another lifecycle state.

Expected outcome: The user can save and retrieve a complete opportunity record, and generated material contains no unsupported claims.

## Fixture Guidance

Use small local JSON fixtures when live platform access is unavailable. Fixtures
should include:

- One verified high-fit opportunity with Flutter or another profile skill.
- One Google/social discovery lead with `requiresVerification: true`.
- One low-budget or scam-like opportunity.
- One duplicate opportunity with the same URL or title/platform pair.
- One Fiverr or Upwork opportunity with setup-required readiness.

## Validation Commands

Run repository-level checks after implementation:

```powershell
python tools/lint_skills.py
python tools/security_guards.py
```

Run focused checks for each new TypeScript/Bun source CLI from that skill's `cli/` directory:

```powershell
bun test --timeout 30000
bun run typecheck
```

Run any Python utility tests added for shared ranking, deduplication, or tracker behavior:

```powershell
pytest
```

## Validation Notes

- 2026-07-14: Focused Bun tests and TypeScript checks passed for `freelance-search`, `upwork-search`, and `fiverr-search` CLIs.
- 2026-07-14: `python tools/lint_skills.py` and `python tools/security_guards.py` passed.
- 2026-07-14: Smoke validation confirmed limited Upwork readiness produces `setup_first`, not `proposal_worthy`.
