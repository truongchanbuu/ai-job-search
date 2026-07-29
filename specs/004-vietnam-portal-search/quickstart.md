# Quickstart: Vietnam Portal Search Validation

These commands describe the expected workflow after implementation.

## Prerequisites

- Bun and TypeScript dependencies installed in each new CLI package.
- Python repository test dependencies installed.
- No authenticated portal cookies or credentials are required or used.
- Sanitized fixtures contain no personal data and no full mirrored job pages.

## 1. Validate each portal adapter

From each of:

```text
.agents/skills/careerviet-search/cli
.agents/skills/vieclam24h-search/cli
.agents/skills/topcv-search/cli
.agents/skills/itviec-search/cli
.agents/skills/vietnamworks-search/cli
```

run:

```powershell
bun run typecheck
bun test --timeout 30000
```

Expected:

- Fixture and mocked-fetch tests pass.
- Unknown fields are `null`.
- 403/login/challenge fixtures are `restricted`, not `no_matches`.
- Closed/404 details are not active.
- Manual-only adapters do not issue prohibited unattended search requests.

## 2. Inspect manual/restricted modes

```powershell
bun run .agents/skills/careerviet-search/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --format json
bun run .agents/skills/vieclam24h-search/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --format json
bun run .agents/skills/vietnamworks-search/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --format json
```

Expected:

- CareerViet returns `manual_required` and an official search URL.
- Vieclam24h and VietnamWorks return `manual_required` or `restricted`.
- All exit cleanly for expected access modes.
- None claims zero matches or full automated coverage.

## 3. Run low-volume public adapter smoke checks

Only after confirming the access review recorded in each `url-reference.md` is
still current:

```powershell
bun run .agents/skills/topcv-search/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --limit 5 --format json
bun run .agents/skills/itviec-search/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --limit 5 --format json
```

Expected:

- When public access is available, canonical attributed discovery links are
  returned and search records are `unverified_lead`.
- A challenge or access block is reported as `restricted`; it is never treated
  as an empty result and is not bypassed.
- Applied and unsupported filters are explicit.
- A successful recognized zero-result page is distinguishable from a block or
  parse failure.

Observed on 2026-07-28: one low-volume request to each source returned a
challenge page. Both adapters stopped and reported `RESTRICTED`.

## 4. Verify a detail before fit

```powershell
bun run .agents/skills/itviec-search/cli/src/cli.ts detail "<public-job-url>" --format json
```

Expected:

- The response reports active/detail/description sufficiency separately.
- Experience evidence comes from the detail, not “fresher” in the title.
- Missing salary/deadline/work mode stays `null`.
- Expired, removed, restricted, and insufficient-detail jobs receive no fit
  recommendation.

If public detail access is challenged, validate this scenario with the
sanitized detail fixtures and record the live deviation instead of bypassing
the control.

## 5. Run the five-source workflow

```powershell
bun run .agents/skills/job-scrape/cli/src/cli.ts search --query "Java backend ReactJS VueJS" --location "Vietnam" --max-experience 1 --jobage 30 --limit 10 --format table
```

Expected:

- Exactly one status row exists for each named portal.
- ITviec and TopCV are attempted as enabled public sources.
- CareerViet is manual-only.
- Vieclam24h and VietnamWorks are manual/restricted unless their access policy
  has been explicitly and permissibly updated.
- Failure of one source does not remove other results.
- Raw, verified, duplicate, expired/removed, inaccessible, and actionable counts
  are distinct.

Observed on 2026-07-28: the workflow returned all five rows; CareerViet,
Vieclam24h, and VietnamWorks were `manual_required`, while TopCV and ITviec
were `restricted`. The workflow exited successfully with no false
`no_matches` claim.

## 6. Validate verification and experience gates

Use fixtures containing:

- A “junior” title whose description requires more than one year.
- A fresher role explicitly accepting no professional experience.
- An active detail with no experience statement.
- A title-only indexed lead.
- An expired/closed detail.

Expected:

- The over-limit role is ineligible.
- The explicit fresher exception is evidence-backed.
- Unknown experience cannot produce an under-one-year recommendation.
- Title-only and expired listings receive no score.

## 7. Validate deduplication and conflicts

Run the orchestration tests with fixtures containing:

- The same employer/role/location on ITviec and TopCV.
- Two same-title roles for different locations or teams.
- Conflicting salary, deadline, and experience facts.
- A source URL already present in seen state and tracker history.

Expected:

- The true cross-portal duplicate becomes one canonical job with both links.
- Distinct same-title roles remain separate.
- Conflicting values and evidence remain visible.
- Existing seen/application status is attached.

## 8. Validate graceful degradation

Mock one adapter as timeout, one as 403, one as zero results, and two as
successful.

Expected:

- The command exits after bounded source timeouts.
- All five status rows appear.
- Successful results remain available.
- 403 is `restricted`, timeout is `failed` or `unavailable`, and only the
  recognized empty page is `no_matches`.

## 9. Validate private persistence

```powershell
bun run .agents/skills/job-scrape/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --source careerviet --source vieclam24h --source vietnamworks --save-session --format json
python tools/security_guards.py
```

Expected:

- A report is created under `job_scraper/sessions/`.
- `seen_jobs.json` and tracker data remain gitignored.
- Existing unrelated state is preserved.
- Search does not rewrite `job_search_tracker.csv`.

## 10. Run repository regression checks

```powershell
python tools/lint_skills.py
python tools/security_guards.py
python -m pytest tests
```

Also confirm `.github/workflows/ci.yml` typechecks all five new portal packages
and the `job-scrape` orchestration package without running live portal requests.
