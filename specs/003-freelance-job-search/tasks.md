# Tasks: Freelance Job Search

**Input**: Design documents from `/specs/003-freelance-job-search/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: The feature spec does not request TDD. Validation tasks are included in the final phase and should run the quickstart scenarios, skill linting, security guards, and focused CLI checks.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files and has no dependency on incomplete tasks.
- **[Story]**: Maps implementation work to user stories from [spec.md](./spec.md).
- Every task includes an exact file path.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the file layout and register the freelance feature surface.

- [X] T001 Verify freelance personal-output ignore coverage in .gitignore
- [X] T002 Create freelance output placeholder directory with .gitkeep in documents/applications/freelance/.gitkeep
- [X] T003 [P] Create freelance workflow guidance stub in agent-guidance/workflows/freelance-search.md
- [X] T004 [P] Create cross-source freelance skill manifest in .agents/skills/freelance-search/SKILL.md
- [X] T005 [P] Create Upwork source skill manifest in .agents/skills/upwork-search/SKILL.md
- [X] T006 [P] Create Fiverr source skill manifest in .agents/skills/fiverr-search/SKILL.md
- [X] T007 Register freelance-search, upwork-search, and fiverr-search in guide.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared contracts, data models, storage, and safety rules needed by every user story.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T008 Create freelance CLI package manifest in .agents/skills/freelance-search/cli/package.json
- [X] T009 Create freelance CLI TypeScript config in .agents/skills/freelance-search/cli/tsconfig.json
- [X] T010 Create freelance CLI entrypoint and command router in .agents/skills/freelance-search/cli/src/cli.ts
- [X] T011 [P] Define freelance TypeScript model types in .agents/skills/freelance-search/cli/src/models.ts
- [X] T012 [P] Implement personal-output path helpers in .agents/skills/freelance-search/cli/src/paths.ts
- [X] T013 [P] Implement JSON storage helpers for profile, readiness, opportunities, and records in .agents/skills/freelance-search/cli/src/storage.ts
- [X] T014 Implement secret redaction and forbidden-field validation in .agents/skills/freelance-search/cli/src/safety.ts
- [X] T015 Implement shared output renderers for json, table, and plain formats in .agents/skills/freelance-search/cli/src/render.ts
- [X] T016 Update job application guidance with freelance factuality rules in agent-guidance/job-application-assistant/freelance.md
- [X] T017 Update workflow index references for freelance search in agent-guidance/project-guide.md

**Checkpoint**: Foundation ready - user story implementation can now begin in priority order or in parallel where staffing allows.

---

## Phase 3: User Story 1 - Find Freelance Opportunities Across Sources (Priority: P1) MVP

**Goal**: Search Fiverr, Upwork, Google public-page discovery, social/user-supplied sources, and return a ranked freelance shortlist with source status and deduplication.

**Independent Test**: Create or load a freelance-ready profile, run one freelance search, and confirm results from available sources are ranked by fit while blocked/setup-required sources report status without aborting the search.

### Implementation for User Story 1

- [X] T018 [P] [US1] Implement freelance profile loading and validation in .agents/skills/freelance-search/cli/src/profile.ts
- [X] T019 [P] [US1] Implement freelance source configuration and status normalization in .agents/skills/freelance-search/cli/src/sources.ts
- [X] T020 [P] [US1] Create Upwork CLI package manifest in .agents/skills/upwork-search/cli/package.json
- [X] T021 [P] [US1] Create Fiverr CLI package manifest in .agents/skills/fiverr-search/cli/package.json
- [X] T022 [P] [US1] Implement Upwork query-link and manual discovery command in .agents/skills/upwork-search/cli/src/cli.ts
- [X] T023 [P] [US1] Implement Fiverr public/manual discovery command in .agents/skills/fiverr-search/cli/src/cli.ts
- [X] T024 [US1] Implement user-supplied opportunity parser in .agents/skills/freelance-search/cli/src/user-supplied.ts
- [X] T025 [US1] Integrate existing Google public-page discovery as a freelance source in .agents/skills/freelance-search/cli/src/google-source.ts
- [X] T026 [US1] Implement freelance opportunity normalization in .agents/skills/freelance-search/cli/src/normalize.ts
- [X] T027 [US1] Implement duplicate opportunity detection in .agents/skills/freelance-search/cli/src/dedupe.ts
- [X] T028 [US1] Implement source-level aggregation and fallback behavior in .agents/skills/freelance-search/cli/src/search.ts
- [X] T029 [US1] Implement profile-driven opportunity ranking in .agents/skills/freelance-search/cli/src/rank.ts
- [X] T030 [US1] Add search command output matching contracts/freelance-search-cli.md in .agents/skills/freelance-search/cli/src/commands/search.ts
- [X] T031 [US1] Document freelance discovery workflow and source limitations in agent-guidance/workflows/freelance-search.md

**Checkpoint**: User Story 1 is independently functional when a profile-driven freelance search returns ranked results and source statuses.

---

## Phase 4: User Story 2 - Manage Fiverr and Upwork Account Readiness (Priority: P1)

**Goal**: Track platform account readiness and block unsafe platform-specific recommendations when setup, verification, Connects, seller approval, or other readiness details are missing.

**Independent Test**: Record Fiverr and Upwork readiness, search platform opportunities, and verify setup gaps appear before platform-specific action is recommended.

### Implementation for User Story 2

- [X] T032 [P] [US2] Implement PlatformAccountReadiness validation in .agents/skills/freelance-search/cli/src/readiness-model.ts
- [X] T033 [US2] Implement readiness storage and update operations in .agents/skills/freelance-search/cli/src/readiness-store.ts
- [X] T034 [US2] Add account readiness CLI command in .agents/skills/freelance-search/cli/src/commands/readiness.ts
- [X] T035 [US2] Integrate readiness blockers into opportunity ranking in .agents/skills/freelance-search/cli/src/rank.ts
- [X] T036 [US2] Add Upwork Connects and setup blocker guidance to .agents/skills/upwork-search/SKILL.md
- [X] T037 [US2] Add Fiverr seller profile and gig readiness guidance to .agents/skills/fiverr-search/SKILL.md
- [X] T038 [US2] Document account readiness workflow in agent-guidance/workflows/freelance-search.md
- [X] T039 [US2] Add readiness output examples to .agents/skills/freelance-search/cli/README.md

**Checkpoint**: User Story 2 is independently functional when Fiverr/Upwork opportunities show readiness or setup-required state before action.

---

## Phase 5: User Story 3 - Evaluate Freelance Fit and Proposal Priority (Priority: P2)

**Goal**: Evaluate each opportunity by skill fit, budget, timeline, platform readiness, reliability, and risk so the user can choose proposal-worthy work.

**Independent Test**: Review a mixed shortlist and confirm each opportunity has recommendation, confidence, matched skills, budget/timeline fit, risk flags, and next action.

### Implementation for User Story 3

- [X] T040 [P] [US3] Implement source reliability note model in .agents/skills/freelance-search/cli/src/reliability.ts
- [X] T041 [P] [US3] Implement freelance risk flag detection in .agents/skills/freelance-search/cli/src/risk.ts
- [X] T042 [US3] Implement FreelanceFitMatch creation in .agents/skills/freelance-search/cli/src/fit.ts
- [X] T043 [US3] Integrate reliability and risk into ranking in .agents/skills/freelance-search/cli/src/rank.ts
- [X] T044 [US3] Add fit evaluation CLI command in .agents/skills/freelance-search/cli/src/commands/evaluate.ts
- [X] T045 [US3] Add mixed-source ranking examples to agent-guidance/workflows/freelance-search.md
- [X] T046 [US3] Update quickstart validation fixture guidance in specs/003-freelance-job-search/quickstart.md

**Checkpoint**: User Story 3 is independently functional when a mixed result set identifies proposal-worthy, review, setup-first, watch, and skip opportunities.

---

## Phase 6: User Story 4 - Prepare Freelance Responses and Track Outcomes (Priority: P2)

**Goal**: Draft truthful freelance response material and save/update opportunity records with status, source references, material references, budget, notes, and outcomes.

**Independent Test**: Select one verified opportunity, generate a proposal outline, save it with status, run search again, and confirm the existing record is detected.

### Implementation for User Story 4

- [X] T047 [P] [US4] Implement ProposalMaterial validation in .agents/skills/freelance-search/cli/src/proposal-model.ts
- [X] T048 [P] [US4] Implement FreelanceOpportunityRecord validation in .agents/skills/freelance-search/cli/src/record-model.ts
- [X] T049 [US4] Implement evidence-aware proposal drafting in .agents/skills/freelance-search/cli/src/proposal.ts
- [X] T050 [US4] Add proposal CLI command in .agents/skills/freelance-search/cli/src/commands/proposal.ts
- [X] T051 [US4] Implement opportunity history save/update operations in .agents/skills/freelance-search/cli/src/records.ts
- [X] T052 [US4] Add save-record CLI command in .agents/skills/freelance-search/cli/src/commands/save.ts
- [X] T053 [US4] Integrate existing-record matching into search output in .agents/skills/freelance-search/cli/src/search.ts
- [X] T054 [US4] Document proposal and outcome tracking workflow in agent-guidance/workflows/freelance-apply.md
- [X] T055 [US4] Add proposal and record output examples to .agents/skills/freelance-search/cli/README.md

**Checkpoint**: User Story 4 is independently functional when proposal material and opportunity history can be created, updated, and matched on later searches.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, docs, security checks, and consistency updates across stories.

- [X] T056 [P] Add focused Bun tests for freelance CLI helpers in .agents/skills/freelance-search/cli/tests/helpers.test.ts
- [X] T057 [P] Add focused Bun tests for Upwork discovery helpers in .agents/skills/upwork-search/cli/tests/helpers.test.ts
- [X] T058 [P] Add focused Bun tests for Fiverr discovery helpers in .agents/skills/fiverr-search/cli/tests/helpers.test.ts
- [X] T059 Add freelance skill README documentation in .agents/skills/freelance-search/cli/README.md
- [X] T060 Update generated skill catalog and legacy command guidance in guide.md
- [X] T061 Run skill lint and fix reported issues in .agents/skills/freelance-search/SKILL.md
- [X] T062 Run security guards and fix secret-handling issues in .agents/skills/freelance-search/cli/src/safety.ts
- [X] T063 Run quickstart validation scenarios and record final notes in specs/003-freelance-job-search/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion - blocks all user stories.
- **User Stories (Phase 3+)**: All depend on Foundational completion.
- **Polish (Phase 7)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - no dependency on other stories and is the MVP.
- **User Story 2 (P1)**: Can start after Foundational - no dependency on US1, but its blockers should be integrated into US1 ranking when both are present.
- **User Story 3 (P2)**: Can start after Foundational - benefits from US1 normalized opportunities and US2 readiness, but fit evaluation can be tested with fixtures.
- **User Story 4 (P2)**: Can start after Foundational - depends on verified opportunity records and profile facts, but can be tested with fixture opportunities.

### Within Each User Story

- Models and validators before storage/services.
- Source adapters before cross-source aggregation.
- Aggregation before ranking.
- Ranking/readiness before proposal-worthy recommendations.
- Proposal drafting before history integration.

### Parallel Opportunities

- Setup manifest tasks T003-T006 can run in parallel.
- Foundational model/storage/rendering tasks T011-T013 can run in parallel.
- US1 source skill package and discovery tasks T018-T023 can run in parallel before aggregation.
- US3 reliability and risk tasks T040-T041 can run in parallel.
- US4 proposal and record model tasks T047-T048 can run in parallel.
- Final focused test tasks T056-T058 can run in parallel.

---

## Parallel Example: User Story 1

```text
Task: "Implement freelance profile loading and validation in .agents/skills/freelance-search/cli/src/profile.ts"
Task: "Implement freelance source configuration and status normalization in .agents/skills/freelance-search/cli/src/sources.ts"
Task: "Create Upwork CLI package manifest in .agents/skills/upwork-search/cli/package.json"
Task: "Create Fiverr CLI package manifest in .agents/skills/fiverr-search/cli/package.json"
Task: "Implement Upwork query-link and manual discovery command in .agents/skills/upwork-search/cli/src/cli.ts"
Task: "Implement Fiverr public/manual discovery command in .agents/skills/fiverr-search/cli/src/cli.ts"
```

## Parallel Example: User Story 2

```text
Task: "Implement PlatformAccountReadiness validation in .agents/skills/freelance-search/cli/src/readiness-model.ts"
Task: "Add Upwork Connects and setup blocker guidance to .agents/skills/upwork-search/SKILL.md"
Task: "Add Fiverr seller profile and gig readiness guidance to .agents/skills/fiverr-search/SKILL.md"
```

## Parallel Example: User Story 3

```text
Task: "Implement source reliability note model in .agents/skills/freelance-search/cli/src/reliability.ts"
Task: "Implement freelance risk flag detection in .agents/skills/freelance-search/cli/src/risk.ts"
```

## Parallel Example: User Story 4

```text
Task: "Implement ProposalMaterial validation in .agents/skills/freelance-search/cli/src/proposal-model.ts"
Task: "Implement FreelanceOpportunityRecord validation in .agents/skills/freelance-search/cli/src/record-model.ts"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate: run a freelance profile search with fixture/user-supplied opportunities and confirm ranked output plus source statuses.
5. Demo if the MVP returns useful opportunities and does not store secrets.

### Incremental Delivery

1. Setup + Foundational: establish shared models, storage, safety, rendering, and skill registration.
2. US1: deliver freelance search and ranking MVP.
3. US2: add platform readiness so Fiverr/Upwork action recommendations are safe.
4. US3: add deeper fit, reliability, and risk scoring.
5. US4: add proposal drafting and opportunity history.
6. Polish: run validation and update docs.

### Parallel Team Strategy

With multiple implementers:

1. Complete Setup and Foundational together.
2. Assign US1 source adapters and aggregation separately.
3. Assign US2 readiness, US3 scoring, and US4 proposal/history after shared models stabilize.
4. Integrate story outputs through the cross-source freelance CLI.

## Notes

- [P] tasks touch different files and can be parallelized after their phase prerequisites are complete.
- Each story phase has an independent checkpoint and can be validated with fixture data if live platform access is unavailable.
- Google and social results remain verification-required discovery leads until original content is verified or user-supplied.
- Never store platform passwords, payment credentials, private messages, secret tokens, or unsupported freelancer claims.
