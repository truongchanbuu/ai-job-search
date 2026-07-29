# Tasks: Vietnam Job Portal Search Coverage

**Input**: Design documents from `/specs/004-vietnam-portal-search/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/portal-adapter-cli.md`, `contracts/job-scrape-output.md`,
`quickstart.md`

**Tests**: Included because the feature specification defines independent tests,
acceptance scenarios, and measurable validation criteria for every user story.
Write each story's tests first and confirm they fail before implementing that
story.

**Organization**: Tasks are grouped by user story so discovery, verification,
deduplication, and coverage reporting can be implemented and validated as
separate increments.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel after its stated prerequisites because it touches
  different files and does not depend on another incomplete parallel task.
- **[Story]**: Maps a task to US1, US2, US3, or US4 from `spec.md`.
- Every checklist item names the exact file or files it changes.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the six independent TypeScript/Bun package shells without
adding portal behavior.

- [X] T001 Create the Bun/TypeScript orchestration package with scripts for `start`, `test`, and `typecheck` in `.agents/skills/job-scrape/cli/package.json`, `.agents/skills/job-scrape/cli/tsconfig.json`, and `.agents/skills/job-scrape/cli/README.md`
- [X] T002 [P] Create the zero-runtime-dependency CareerViet package shell in `.agents/skills/careerviet-search/cli/package.json`, `.agents/skills/careerviet-search/cli/tsconfig.json`, and `.agents/skills/careerviet-search/cli/README.md`
- [X] T003 [P] Create the zero-runtime-dependency Vieclam24h package shell in `.agents/skills/vieclam24h-search/cli/package.json`, `.agents/skills/vieclam24h-search/cli/tsconfig.json`, and `.agents/skills/vieclam24h-search/cli/README.md`
- [X] T004 [P] Create the zero-runtime-dependency TopCV package shell in `.agents/skills/topcv-search/cli/package.json`, `.agents/skills/topcv-search/cli/tsconfig.json`, and `.agents/skills/topcv-search/cli/README.md`
- [X] T005 [P] Create the zero-runtime-dependency ITviec package shell in `.agents/skills/itviec-search/cli/package.json`, `.agents/skills/itviec-search/cli/tsconfig.json`, and `.agents/skills/itviec-search/cli/README.md`
- [X] T006 [P] Create the zero-runtime-dependency VietnamWorks package shell in `.agents/skills/vietnamworks-search/cli/package.json`, `.agents/skills/vietnamworks-search/cli/tsconfig.json`, and `.agents/skills/vietnamworks-search/cli/README.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define shared contracts, access policy, criteria parsing, child
process isolation, and compatibility normalization required by every story.

**Critical**: No user story implementation begins until this phase is complete.

- [X] T007 Add failing contract-shape tests for search criteria, portal results, listing evidence, verification states, canonical jobs, conflicts, fit evidence, seen state, and coverage summaries in `.agents/skills/job-scrape/cli/tests/contract.test.ts`
- [X] T008 Implement the versioned types and runtime guards from both feature contracts in `.agents/skills/job-scrape/cli/src/contracts.ts`
- [X] T009 Add failing registry tests for all five source IDs, access modes, supported constraints, review dates, timeouts, and the prohibition on unattended manual/restricted search in `.agents/skills/job-scrape/cli/tests/portal-registry.test.ts`
- [X] T010 Implement the five portal capability records and validate their access policy at startup in `.agents/skills/job-scrape/cli/src/portal-registry.ts`
- [X] T011 [P] Implement bounded Bun child-process execution, JSON/stderr capture, timeout handling, and source-local failure conversion in `.agents/skills/job-scrape/cli/src/run-portal.ts`
- [X] T012 [P] Implement canonical URL cleanup and normalization for both rich Vietnam envelopes and legacy LinkedIn/FreeHire `{meta, results}` output in `.agents/skills/job-scrape/cli/src/normalize.ts`
- [X] T013 [P] Implement reusable spawned-CLI, mocked-clock, fixture-loader, and stdout-capture helpers in `.agents/skills/job-scrape/cli/tests/helpers.ts`
- [X] T014 Implement shared flag parsing and validation for query, location, maximum experience, seniority, recency, work mode, employment type, source allowlist, limits, profile/state paths, save-session, and output format in `.agents/skills/job-scrape/cli/src/cli.ts`

**Checkpoint**: Shared contracts and source isolation are ready; story work may
begin.

---

## Phase 3: User Story 1 - Search Major Vietnam Job Portals (Priority: P1) MVP

**Goal**: One request attempts CareerViet, Vieclam24h, TopCV, ITviec, and
VietnamWorks, preserving mixed Vietnamese/English criteria and returning a
source-attributed result or explicit source status for every portal.

**Independent Test**: Run a role/skills/location/experience search against
mocked source responses and confirm that all five portals have exactly one
status, public adapters return attributed leads, manual/restricted adapters
return official review links, a valid empty page is `no_matches`, and one
failure does not remove sibling results.

### Tests for User Story 1

- [X] T015 [P] [US1] Add failing tests that CareerViet search emits `manual_required`, an official criteria-preserving browser URL, zero automated results, and no network request in `.agents/skills/careerviet-search/cli/tests/commands.test.ts`
- [X] T016 [P] [US1] Add failing tests that Vieclam24h search emits `manual_required` or `restricted` without retrying around 403/challenge behavior in `.agents/skills/vieclam24h-search/cli/tests/commands.test.ts`
- [X] T017 [P] [US1] Add failing TopCV search parser and command tests for canonical IDs/URLs, Vietnamese diacritics, mixed-language queries, valid zero results, malformed pages, and missing fields in `.agents/skills/topcv-search/cli/tests/search.test.ts` and `.agents/skills/topcv-search/cli/tests/fixtures/search/`
- [X] T018 [P] [US1] Add failing ITviec search parser and command tests for skill/fresher/location slugs, work-mode filters, canonical IDs/URLs, valid zero results, malformed pages, and missing fields in `.agents/skills/itviec-search/cli/tests/search.test.ts` and `.agents/skills/itviec-search/cli/tests/fixtures/search/`
- [X] T019 [P] [US1] Add failing tests that VietnamWorks search emits `manual_required` or `restricted` without inferring job identifiers or calling private/mobile endpoints in `.agents/skills/vietnamworks-search/cli/tests/commands.test.ts`
- [X] T020 [P] [US1] Add failing orchestration tests for five status rows, concurrent public sources, bounded timeout, one-source failure, 403 versus zero-match semantics, and source attribution in `.agents/skills/job-scrape/cli/tests/orchestration.test.ts` and `.agents/skills/job-scrape/cli/tests/graceful-degradation.test.ts`
- [X] T021 [P] [US1] Add failing criteria tests proving technology tokens, exclusions, aliases, and Vietnamese diacritics retain their meaning while unsupported filters are disclosed in `.agents/skills/job-scrape/cli/tests/criteria.test.ts`

### Implementation for User Story 1

- [X] T022 [P] [US1] Implement CareerViet search as a no-fetch official-URL generator with structured JSON/table/plain output, flag errors, and policy documentation in `.agents/skills/careerviet-search/cli/src/cli.ts`, `.agents/skills/careerviet-search/cli/src/helpers.ts`, `.agents/skills/careerviet-search/cli/src/commands/search.ts`, `.agents/skills/careerviet-search/SKILL.md`, and `.agents/skills/careerviet-search/url-reference.md`
- [X] T023 [P] [US1] Implement Vieclam24h search as a restricted/manual adapter with structured statuses, official review links, no bypass retries, and policy documentation in `.agents/skills/vieclam24h-search/cli/src/cli.ts`, `.agents/skills/vieclam24h-search/cli/src/helpers.ts`, `.agents/skills/vieclam24h-search/cli/src/commands/search.ts`, `.agents/skills/vieclam24h-search/SKILL.md`, and `.agents/skills/vieclam24h-search/url-reference.md`
- [X] T024 [P] [US1] Implement low-volume TopCV public search, filter mapping, access-change detection, canonical result parsing, bounded transient retries, and link-first documentation in `.agents/skills/topcv-search/cli/src/cli.ts`, `.agents/skills/topcv-search/cli/src/helpers.ts`, `.agents/skills/topcv-search/cli/src/commands/search.ts`, `.agents/skills/topcv-search/SKILL.md`, and `.agents/skills/topcv-search/url-reference.md`
- [X] T025 [P] [US1] Implement low-volume ITviec public search, slug/filter mapping, access-change detection, canonical result parsing, bounded transient retries, and link-first documentation in `.agents/skills/itviec-search/cli/src/cli.ts`, `.agents/skills/itviec-search/cli/src/helpers.ts`, `.agents/skills/itviec-search/cli/src/commands/search.ts`, `.agents/skills/itviec-search/SKILL.md`, and `.agents/skills/itviec-search/url-reference.md`
- [X] T026 [P] [US1] Implement VietnamWorks search as a restricted/manual adapter with structured statuses, official review links, no identifier inference or bypass retries, and policy documentation in `.agents/skills/vietnamworks-search/cli/src/cli.ts`, `.agents/skills/vietnamworks-search/cli/src/helpers.ts`, `.agents/skills/vietnamworks-search/cli/src/commands/search.ts`, `.agents/skills/vietnamworks-search/SKILL.md`, and `.agents/skills/vietnamworks-search/url-reference.md`
- [X] T027 [US1] Implement criteria-preserving parallel source execution and one `PortalRunResult` per enabled source in `.agents/skills/job-scrape/cli/src/commands/search.ts`
- [X] T028 [US1] Wire the `job-scrape search` command to source allowlists, profile criteria, JSON/table/plain formats, structured errors, and exit behavior in `.agents/skills/job-scrape/cli/src/cli.ts`
- [X] T029 [US1] Document the five-source orchestration command, source precedence, manual links, and no-bypass behavior in `.agents/skills/job-scrape/SKILL.md`

**Checkpoint**: US1 is independently usable as a discovery MVP. All five
sources are attempted and reported, but no discovery lead is yet recommended.

---

## Phase 4: User Story 2 - Verify Listings Before Fit Screening (Priority: P1)

**Goal**: Retrieve or accept permitted detail evidence, classify listing
availability and sufficiency, and allow fit recommendations only when active
detail evidence supports the user's constraints.

**Independent Test**: Feed active, expired, removed, incomplete, restricted, and
title-only details—including a “junior” title requiring more than one year—and
confirm only sufficiently detailed active listings receive recommendations,
experience decisions cite source evidence, and absent facts remain `null`.

### Tests for User Story 2

- [X] T030 [P] [US2] Add failing TopCV detail tests for canonical URL/ID parsing, description and requirements evidence, experience ranges, deadline/closed markers, missing values, 404, 403, and challenge pages in `.agents/skills/topcv-search/cli/tests/detail.test.ts` and `.agents/skills/topcv-search/cli/tests/fixtures/detail/`
- [X] T031 [P] [US2] Add failing ITviec detail tests for description, skills, accepted level, work model, relative posting age, login-only salary, missing deadline, 404, 403, and challenge pages in `.agents/skills/itviec-search/cli/tests/detail.test.ts` and `.agents/skills/itviec-search/cli/tests/fixtures/detail/`
- [X] T032 [P] [US2] Add failing CareerViet manual-detail tests that normalize a user-supplied reference without unattended page retrieval and reject unsupported identifiers in `.agents/skills/careerviet-search/cli/tests/detail.test.ts`
- [X] T033 [P] [US2] Add failing Vieclam24h manual-detail tests that preserve user-supplied URL/text attribution while returning restricted or insufficient-detail status without protected requests in `.agents/skills/vieclam24h-search/cli/tests/detail.test.ts`
- [X] T034 [P] [US2] Add failing VietnamWorks manual-detail tests that preserve user-supplied URL/text attribution while returning restricted or insufficient-detail status without private/mobile requests in `.agents/skills/vietnamworks-search/cli/tests/detail.test.ts`
- [X] T035 [P] [US2] Add failing verification transition tests for active, insufficient, expired, removed, restricted, failed, title-only, and description-sufficiency cases in `.agents/skills/job-scrape/cli/tests/verification.test.ts`
- [X] T036 [P] [US2] Add failing fit-gate tests for explicit experience limits, fresher exceptions, title/description conflicts, unknown facts, hard exclusions, source evidence references, and zero scores for unverified listings in `.agents/skills/job-scrape/cli/tests/fit.test.ts`

### Implementation for User Story 2

- [X] T037 [P] [US2] Implement low-volume TopCV public detail retrieval, canonicalization, normalized decision facts, bounded evidence summaries, expiry detection, and structured verification output in `.agents/skills/topcv-search/cli/src/commands/detail.ts` and `.agents/skills/topcv-search/cli/src/helpers.ts`
- [X] T038 [P] [US2] Implement low-volume ITviec public detail retrieval, canonicalization, normalized decision facts, bounded evidence summaries, expiry detection, and structured verification output in `.agents/skills/itviec-search/cli/src/commands/detail.ts` and `.agents/skills/itviec-search/cli/src/helpers.ts`
- [X] T039 [P] [US2] Implement CareerViet manual detail/reference intake that performs no unattended search or protected retrieval in `.agents/skills/careerviet-search/cli/src/commands/detail.ts` and `.agents/skills/careerviet-search/cli/src/cli.ts`
- [X] T040 [P] [US2] Implement Vieclam24h user-supplied detail/reference intake with explicit restricted and insufficient-detail outcomes in `.agents/skills/vieclam24h-search/cli/src/commands/detail.ts` and `.agents/skills/vieclam24h-search/cli/src/cli.ts`
- [X] T041 [P] [US2] Implement VietnamWorks user-supplied detail/reference intake with explicit restricted and insufficient-detail outcomes in `.agents/skills/vietnamworks-search/cli/src/commands/detail.ts` and `.agents/skills/vietnamworks-search/cli/src/cli.ts`
- [X] T042 [US2] Implement listing verification, active/closed evidence evaluation, description sufficiency, and fit eligibility gates in `.agents/skills/job-scrape/cli/src/verify.ts`
- [X] T043 [US2] Implement source-backed skill, experience, location, work-mode, employment-type, and exclusion screening without inferred matches in `.agents/skills/job-scrape/cli/src/fit.ts`
- [X] T044 [US2] Integrate promising-result detail retrieval, unverified-lead separation, verification, and nullable fit output into `.agents/skills/job-scrape/cli/src/commands/search.ts`

**Checkpoint**: US2 is independently testable with detail fixtures and makes the
US1 discovery flow safe for actionable recommendations.

---

## Phase 5: User Story 3 - Receive One Deduplicated, Transparent Shortlist (Priority: P2)

**Goal**: Consolidate true duplicate vacancies while preserving every source
copy, disagreement, seen-job state, and existing application status.

**Independent Test**: Supply a cross-portal duplicate, distinct same-title
roles, conflicting fields, existing seen state, and an application row; confirm
one canonical duplicate retains both links and conflicts, distinct jobs remain
separate, and history appears without modifying unrelated records.

### Tests for User Story 3

- [X] T045 [P] [US3] Add failing layered-deduplication tests for exact IDs/URLs, existing source references, employer-title-location identity, evidence similarity, reposts, conflicts, and deliberately distinct same-title roles in `.agents/skills/job-scrape/cli/tests/deduplication.test.ts`
- [X] T046 [P] [US3] Add failing seen-state and tracker reconciliation tests for missing files, raw-array and envelope JSON, CSV matches, preserved unrelated rows, and attached existing statuses in `.agents/skills/job-scrape/cli/tests/history.test.ts`
- [X] T047 [P] [US3] Add failing private session and seen-state persistence tests for explicit `--save-session`, stable IDs, source unions, preserved `firstSeenAt`, and no tracker writes in `.agents/skills/job-scrape/cli/tests/storage.test.ts`

### Implementation for User Story 3

- [X] T048 [US3] Implement conservative layered canonical-key matching, source-reference unions, distinct-role safeguards, preferred evidence selection, and unresolved field conflicts in `.agents/skills/job-scrape/cli/src/deduplicate.ts`
- [X] T049 [P] [US3] Implement missing-safe `seen_jobs.json` and read-only `job_search_tracker.csv` readers plus source/identity status reconciliation in `.agents/skills/job-scrape/cli/src/history.ts`
- [X] T050 [P] [US3] Implement private search-session writes and safe seen-state upserts that preserve unrelated records and `firstSeenAt` in `.agents/skills/job-scrape/cli/src/storage.ts`
- [X] T051 [US3] Integrate deduplication, conflicts, seen/application statuses, and opt-in session persistence into canonical job output in `.agents/skills/job-scrape/cli/src/commands/search.ts`

**Checkpoint**: US3 produces one history-aware canonical shortlist without
discarding source evidence or merging distinct roles.

---

## Phase 6: User Story 4 - Understand Portal Coverage and Search Limitations (Priority: P2)

**Goal**: Present a concise source-by-source account of applied/unsupported
filters, access limitations, and counts from discovery through actionable
matches.

**Independent Test**: Use sources with different filter support, one valid empty
result, one restriction, one failure, and discoveries eliminated at different
stages; confirm the JSON/table/plain report explains every source and
distinguishes raw, verified, duplicate, expired/removed, inaccessible, and
actionable counts without internal logs.

### Tests for User Story 4

- [X] T052 [P] [US4] Add failing coverage aggregation tests for exactly one row per source, applied/unsupported constraints, internally consistent stage counts, manual URLs, and zero-match/restricted/unavailable/failed distinctions in `.agents/skills/job-scrape/cli/tests/coverage.test.ts`
- [X] T053 [P] [US4] Add failing JSON/table/plain rendering tests for the portal coverage section, actionable shortlist, unverified leads, conflicts, missing facts, and access warnings in `.agents/skills/job-scrape/cli/tests/render.test.ts`

### Implementation for User Story 4

- [X] T054 [US4] Implement derived per-source and total coverage counts without independently mutable totals in `.agents/skills/job-scrape/cli/src/coverage.ts`
- [X] T055 [US4] Implement JSON/table/plain coverage, shortlist, unverified-lead, conflict, and warning sections in `.agents/skills/job-scrape/cli/src/render.ts`
- [X] T056 [US4] Integrate final coverage derivation and human rendering with every search exit path in `.agents/skills/job-scrape/cli/src/commands/search.ts`

**Checkpoint**: All four user stories are independently testable and compose
into the complete five-portal workflow.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Register the feature, enforce CI/privacy rules, and execute the
documented validation without broadening portal access.

- [X] T057 [P] Register all five portal skills and the new orchestration behavior in `guide.md` and `agent-guidance/workflows/job-scrape.md`
- [X] T058 [P] Add offline typecheck/test jobs for the five portal packages and `job-scrape` while excluding live portal calls in `.github/workflows/ci.yml`
- [X] T059 Run `bun run typecheck` and `bun test --timeout 30000` in all six new CLI packages and fix package-local failures in `.agents/skills/careerviet-search/cli/`, `.agents/skills/vieclam24h-search/cli/`, `.agents/skills/topcv-search/cli/`, `.agents/skills/itviec-search/cli/`, `.agents/skills/vietnamworks-search/cli/`, and `.agents/skills/job-scrape/cli/`
- [X] T060 Run repository lint, security, and pytest regression commands and fix feature-caused failures in `tools/lint_skills.py`, `tools/security_guards.py`, and `tests/`
- [X] T061 Verify private outputs remain ignored, fixtures contain no personal data or full mirrored pages, and no Claude compatibility file was removed by inspecting `.gitignore`, `job_scraper/`, `.agents/skills/*-search/cli/tests/fixtures/`, and `.claude/`
- [X] T062 Recheck current TopCV/ITviec public access and all five policy review dates, run only the opt-in low-volume smoke commands, and update observed behavior in `.agents/skills/topcv-search/url-reference.md`, `.agents/skills/itviec-search/url-reference.md`, `.agents/skills/careerviet-search/url-reference.md`, `.agents/skills/vieclam24h-search/url-reference.md`, and `.agents/skills/vietnamworks-search/url-reference.md`
- [X] T063 Execute every applicable validation scenario, record access-dependent deviations without bypassing controls, and keep commands/expected results current in `specs/004-vietnam-portal-search/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 — Setup**: No dependencies. T002–T006 can run in parallel with each
  other after package conventions are confirmed from T001.
- **Phase 2 — Foundational**: Depends on Phase 1 and blocks all stories. T007
  precedes T008; T009 precedes T010; T011–T013 can proceed in parallel after
  T008; T014 depends on the shared contract.
- **Phase 3 — US1**: Depends on Phase 2. Tests T015–T021 are written first;
  portal implementations T022–T026 can then run in parallel; T027 depends on
  those adapters and the orchestration tests; T028–T029 finish the MVP.
- **Phase 4 — US2**: Core detail/verification work can start after Phase 2 using
  fixtures. T030–T036 precede their implementations. T037–T041 can run in
  parallel, T042 and T043 follow their tests, and T044 integrates with US1.
- **Phase 5 — US3**: Core dedupe/history/storage work can start after Phase 2
  using normalized fixtures. T045–T047 precede T048–T050; T051 integrates with
  the US1/US2 pipeline.
- **Phase 6 — US4**: Coverage/rendering can start after Phase 2 with fixture
  `PortalRunResult` records. T052–T053 precede T054–T055; T056 integrates after
  the desired US1–US3 increments.
- **Phase 7 — Polish**: Depends on all stories selected for release.

### User Story Dependencies

```text
Setup -> Foundational
                   |-> US1 Discovery ----------------------|
                   |-> US2 Verification -- integrates US1 -|
                   |-> US3 Dedupe/History - integrates US1+US2
                   `-> US4 Coverage ------- integrates selected stories
                                                        |
                                                        `-> Polish
```

- **US1 (P1)**: No story dependency; delivers source discovery/status MVP.
- **US2 (P1)**: Verification library and portal details are independently
  fixture-testable; T044 depends on the US1 search pipeline.
- **US3 (P2)**: Dedupe/history libraries are independently fixture-testable;
  T051 depends on canonical results from US1 and verification from US2.
- **US4 (P2)**: Coverage/rendering is independently fixture-testable; T056
  integrates the completed story outputs selected for release.

### Within Each User Story

1. Write the listed tests and confirm they fail for the intended missing
   behavior.
2. Implement source-specific models/parsers before orchestration integration.
3. Preserve source attribution and access restrictions at every boundary.
4. Run that story's test files before moving to its checkpoint.

## Parallel Opportunities

- Setup shells T002–T006 touch different portal directories.
- Foundational runner, normalizer, and test helpers T011–T013 touch different
  files after the shared contract exists.
- US1 source tests T015–T019 and criteria/orchestration tests T020–T021 can be
  written in parallel; adapters T022–T026 can be implemented in parallel.
- US2 detail tests T030–T034, shared rule tests T035–T036, and portal detail
  implementations T037–T041 can run by portal in parallel.
- US3 test files T045–T047 can run in parallel; history and storage T049–T050
  can run in parallel after their tests.
- US4 coverage and renderer tests T052–T053 can run in parallel.
- Registration and CI tasks T057–T058 touch different files.

## Parallel Example: User Story 1

```text
Parallel tests:
- T015 CareerViet manual-search contract
- T016 Vieclam24h restriction contract
- T017 TopCV search parser
- T018 ITviec search parser
- T019 VietnamWorks restriction contract
- T020 Orchestration/graceful degradation
- T021 Mixed-language criteria preservation

Parallel implementation after the matching tests fail:
- T022 CareerViet adapter
- T023 Vieclam24h adapter
- T024 TopCV adapter
- T025 ITviec adapter
- T026 VietnamWorks adapter
```

## Parallel Example: User Story 2

```text
Parallel tests:
- T030 TopCV detail evidence
- T031 ITviec detail evidence
- T032 CareerViet manual detail
- T033 Vieclam24h manual detail
- T034 VietnamWorks manual detail
- T035 Verification transitions
- T036 Fit evidence gates

Parallel implementation after the matching tests fail:
- T037 TopCV detail
- T038 ITviec detail
- T039 CareerViet manual intake
- T040 Vieclam24h manual intake
- T041 VietnamWorks manual intake
```

## Parallel Example: User Story 3

```text
Parallel tests:
- T045 Layered deduplication/conflicts
- T046 Seen/tracker reconciliation
- T047 Private persistence

Parallel implementation after the matching tests fail:
- T049 History readers/reconciliation
- T050 Session and seen-state storage
```

## Parallel Example: User Story 4

```text
Parallel tests:
- T052 Coverage aggregation
- T053 JSON/table/plain rendering
```

## Implementation Strategy

### Discovery MVP (US1)

1. Complete Setup and Foundational phases.
2. Complete US1 tests and implementations.
3. Stop and validate the five-source status/discovery workflow.
4. Present all results as discovery leads; do not emit fit recommendations yet.

### Safe Actionable MVP (US1 + US2)

1. Add permitted public/manual detail handling.
2. Add verification and evidence-backed fit gates.
3. Validate that inaccessible, expired, removed, and title-only leads receive no
   recommendation.
4. Release actionable recommendations only after this checkpoint.

### Incremental Delivery

1. US1 adds complete, honest portal attempts.
2. US2 adds safe actionable screening.
3. US3 adds deduplication and history awareness.
4. US4 adds the complete transparent coverage report.
5. Polish registers and validates the selected release.

## Notes

- CareerViet remains `manual_only` unless written permission changes the
  recorded policy.
- Vieclam24h and VietnamWorks remain `restricted/manual_required` until a
  permitted public interface is verified.
- TopCV and ITviec remain low-volume, link-first, and fail closed when access
  behavior changes.
- Do not add credentials, authenticated cookies, CAPTCHA solving, proxy
  rotation, private/mobile endpoints, or full page archives.
- `[P]` means file-level parallelism, not permission to skip stated test or
  foundation prerequisites.
- Commit after each task or cohesive task group and validate at every story
  checkpoint.
