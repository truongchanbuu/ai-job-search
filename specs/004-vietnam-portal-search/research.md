# Research: Vietnam Job Portal Search Coverage

**Date**: 2026-07-28

Research used ordinary public requests only. No login, CAPTCHA, authenticated
cookie, private endpoint, mobile API, or access-control bypass was attempted.
Because portal behavior and terms can change, every source record carries an
access review date and the implementation must fail closed when current
behavior differs from its recorded capability.

## Decision 1: Use five source skills and one shared orchestrator

**Decision**: Add an independent `*-search` skill/CLI for each named portal and
add deterministic cross-source orchestration to the existing `job-scrape`
skill.

**Rationale**: The repository already isolates portal-specific fetch and parse
logic under `.agents/skills/<portal>-search/cli/`. A shared `job-scrape` CLI is
the missing boundary for per-source failure isolation, verification,
deduplication, seen/tracker reconciliation, and one coverage report.

**Alternatives considered**:

- One monolithic Vietnam scraper: rejected because one portal change would
  couple all five sources and obscure source-level failures.
- Prompt-only orchestration: rejected because deduplication, verification gates,
  state reconciliation, and measurable output contracts need deterministic
  tests.
- Rewrite existing LinkedIn/FreeHire CLIs: rejected because the orchestrator can
  normalize their current output without destabilizing working integrations.

## Decision 2: Make access mode a first-class portal capability

**Decision**: Register each portal as `enabled_public`, `manual_only`, or
`restricted`, separate from the outcome of an individual search run.

**Rationale**: A static access policy and a dynamic run result answer different
questions. A restricted portal is not equivalent to a successful search with
zero matches.

**Alternatives considered**:

- Treat every portal as an automated HTML scraper: rejected because it would
  conflict with published terms or current 403 behavior.
- Omit restricted portals: rejected because the specification requires an
  explicit outcome for every enabled named source.
- Use search-engine snippets as verified listings: rejected because snippets
  are incomplete and may be stale.

## Decision 3: Portal-specific access plan

### CareerViet — `manual_only`

CareerViet exposes useful public browser filters for keyword, industry, city,
salary, level, recency, employment type, and experience. Public details show
location, employment type, salary, experience, level, deadline, description,
and requirements. However, its current terms explicitly prohibit
machine/crawler/robot-style searching outside its own tools and ordinary
browsers.

**Implementation**: Generate an official search URL and accept a user-supplied
public detail URL for normalization/verification. Do not perform unattended
search-page crawling without written permission.

**Evidence**:

- [CareerViet search example](https://careerviet.vn/viec-lam/fresher-java-developers-k-vi.html)
- [CareerViet under-one-year filter example](https://careerviet.vn/viec-lam/fresher-java-developers-kn3-vi.html)
- [CareerViet public detail example](https://careerviet.vn/vi/tim-viec-lam/lap-trinh-vien-fullstack-developer.35C7D09E.html)
- [CareerViet terms](https://careerviet.vn/vi/jobseekers/use)

### TopCV — `enabled_public`, low-volume and link-first

TopCV exposes public search and canonical detail pages. Observed filters include
keyword, location, specialization, experience, salary, company, and level.
Canonical detail URLs end in a numeric ID and show description, requirements,
location, experience, salary, schedule, and deadline. Account actions remain
out of scope. Its terms restrict copying/reuse of TopCV content.

**Implementation**: Use low-volume public search/detail requests after a policy
check at implementation time. Persist canonical URLs and normalized factual
evidence, not raw HTML or full mirrored descriptions.

**Evidence**:

- [TopCV Java search](https://www.topcv.vn/tim-viec-lam-java-developer)
- [TopCV canonical detail example](https://www.topcv.vn/viec-lam/fresher-java-6-thang-1-nam-kinh-nghiem-thu-nhap-8-12tr-thang/1954519.html)
- [TopCV robots.txt](https://www.topcv.vn/robots.txt)
- [TopCV terms](https://www.topcv.vn/terms-of-service)

### ITviec — `enabled_public`, low-volume

ITviec exposes public skill/title/category search slugs, city variants, and
filters for level, workplace model, salary, domain, industry, and company type.
Public details show description, skills, expertise, domain, location, work
model, accepted levels, and posting age. Salary/apply actions may require login
and therefore remain unknown/out of scope.

**Implementation**: Enable low-volume public search/detail retrieval, preserve
canonical attribution, and store extracted decision facts rather than bulk page
content.

**Evidence**:

- [ITviec Java search](https://itviec.com/it-jobs/java)
- [ITviec fresher search](https://itviec.com/it-jobs/fresher)
- [ITviec HCMC fresher search](https://itviec.com/it-jobs/fresher/ho-chi-minh-hcm)
- [ITviec public detail example](https://itviec.com/it-jobs/software-developer-fresher-junior-good-english-netcompany-3413)
- [ITviec robots.txt](https://itviec.com/robots.txt)
- [ITviec terms](https://itviec.com/blog/terms-and-conditions/)

### Vieclam24h — `restricted`

Ordinary unauthenticated requests to its public home and search pages returned
403 in the research environment. The official mobile listing advertises
keyword/industry, region, salary, level, and experience filters, but no current
public detail identifier could be verified without crossing the restriction.

**Implementation**: Return `manual_required` with the official portal/app link.
Accept user-supplied URLs or pasted descriptions. Do not reverse-engineer the
mobile app, reuse tokens, or retry around 403/CAPTCHA.

**Evidence**:

- [Vieclam24h home](https://vieclam24h.vn/)
- [Vieclam24h search URL](https://vieclam24h.vn/tim-kiem-viec-lam-nhanh?q=java)
- [Official Vieclam24h Google Play listing](https://play.google.com/store/apps/details?hl=vi&id=com.vieclam24h)
- [Official Vieclam24h App Store listing](https://apps.apple.com/vn/app/vieclam24h-t%C3%ACm-vi%E1%BB%87c-nhanh/id6615065541?l=vi)

### VietnamWorks — `restricted`

Ordinary unauthenticated requests to the home, Java search, and robots page
returned 403 in the research environment. Its official app describes keyword,
location, industry, salary, and level search plus employer, description, and
benefit details. No dependable current public web detail identifier was
verified.

**Implementation**: Return `manual_required` with the official portal/app link.
Accept user-supplied URLs or pasted descriptions. Do not infer identifiers or
reverse-engineer private/mobile traffic.

**Evidence**:

- [VietnamWorks home](https://www.vietnamworks.com/)
- [VietnamWorks Java search](https://www.vietnamworks.com/viec-lam?q=java)
- [Official VietnamWorks Google Play listing](https://play.google.com/store/apps/details?hl=vi&id=com.vietnamworks.vietnamworks)

## Decision 4: Normalize at the orchestration boundary

**Decision**: New adapters emit the richer envelope in
`contracts/portal-adapter-cli.md`; `job-scrape` also accepts the existing
LinkedIn/FreeHire `{meta, results}` shape and maps it into that envelope.

**Rationale**: Current CLIs do not include top-level source status, filter
coverage, or verification state. A compatibility adapter avoids a broad
migration while giving new sources a precise contract.

**Alternatives considered**:

- Force every existing source to migrate in this feature: rejected as unrelated
  scope and regression risk.
- Let every adapter emit arbitrary records: rejected because the coverage and
  fit gates could not be tested reliably.

## Decision 5: Separate discovery, verification, and fit

**Decision**: Search produces discovery records. Detail retrieval produces
field-level evidence and a verification state. Fit scoring is allowed only for
`verified_active` records with enough description/requirement evidence to test
the user's explicit constraints.

**Rationale**: Titles such as “fresher” or “junior” can conflict with explicit
experience requirements. Search pages and snippets can also be stale.

**Alternatives considered**:

- Score every search card: rejected because it violates FR-009 through FR-011.
- Require every optional field: rejected because salary, deadline, and work mode
  are often legitimately absent and must remain unknown.

## Decision 6: Treat portal filters as hints

**Decision**: Preserve the user's English, Vietnamese, and mixed-language terms;
map only supported filters into each portal; record applied and unsupported
constraints; then re-evaluate all hard constraints from detail evidence.

**Rationale**: Filter vocabularies differ, technology names should not be
translated destructively, and observed CareerViet results included unrelated
and senior roles for a fresher query.

**Alternatives considered**:

- Force one common URL schema: rejected because portal filters are not
  equivalent.
- Infer experience from portal category labels: rejected because explicit
  description evidence takes precedence.

## Decision 7: Use conservative layered deduplication

**Decision**: Match in ordered layers: exact normalized URL/source ID; existing
application/source reference; normalized employer + title + location; then
description/evidence similarity only when strong enough. Preserve all source
copies and field conflicts.

**Rationale**: The existing freelance canonical key includes platform/URL and
cannot collapse cross-portal copies. Conversely, title-only matching would merge
distinct roles.

**Alternatives considered**:

- URL-only matching: rejected because syndicated jobs have different portal
  URLs.
- Employer/title-only matching: rejected because employers can advertise
  multiple similar roles.
- Choose one “winning” record and discard others: rejected because source
  evidence and conflicts must remain visible.

## Decision 8: Define private state concretely

**Decision**: Store search sessions under `job_scraper/sessions/`, define
`job_scraper/seen_jobs.json`, and read `job_search_tracker.csv` when present.
State readers accept missing files and preserve unrelated records on updates.

**Rationale**: These paths are already gitignored and referenced by guidance,
but no current schema or implementation exists. The freelance record store
provides a reusable safe-upsert pattern.

**Alternatives considered**:

- Add a database: rejected for a single-user, file-backed workspace.
- Store state inside portal packages: rejected because deduplication and history
  are cross-source concerns.

## Decision 9: Use fixtures for CI and opt-in live smoke tests

**Decision**: Test parsers and commands with sanitized fixtures/mocked fetch in
CI; add new packages to the CI typecheck matrix; keep live checks explicit,
low-volume, and local.

**Rationale**: Live portal tests are nondeterministic, can create load, and may
encounter access changes. Fixtures cover parsing and error semantics without
network use.

**Alternatives considered**:

- Live-only tests: rejected as flaky and policy-unfriendly.
- Fixture-only validation forever: rejected because opt-in smoke checks are
  still needed to detect portal changes before a real search.

## Decision 10: Fail closed on access changes

**Decision**: On unexpected redirects, login, CAPTCHA/challenge pages, 401/403,
or repeated 429/5xx responses, stop that source after a small bounded retry for
transient failures and report `restricted`, `unavailable`, or `failed`.

**Rationale**: A source access change must not become a parser success, an empty
result, or an invitation to bypass controls.

**Alternatives considered**:

- Rotate proxies, solve CAPTCHA, or use credentials: rejected as explicitly out
  of scope.
- Retry indefinitely: rejected because it can create load and block the
  cross-source workflow.
