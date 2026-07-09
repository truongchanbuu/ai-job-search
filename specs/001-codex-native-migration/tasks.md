# Tasks: Codex-Native Migration With Dual Support

**Input**: Current migration plan from Codex conversation; repository inspection of `.claude/`, `.agents/skills/`, `.specify/`, docs, and tooling.

**Prerequisites**: Existing Claude workflows, existing Codex portal/spec-kit skills, agreed dual-support migration plan.

**Tests**: Include validation tasks for lint/security tooling, skill inventory completeness, and documentation parity.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files and has no dependency on incomplete tasks.
- **[Story]**: Maps task to one user story.
- All implementation tasks include explicit repository paths.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the shared migration structure and preserve current behavior before rewriting workflow surfaces.

- [X] T001 Create shared guidance directory structure in agent-guidance/
- [X] T002 [P] Create migration notes scaffold in specs/001-codex-native-migration/migration-notes.md
- [X] T003 [P] Inventory current Claude commands and skills in specs/001-codex-native-migration/claude-inventory.md
- [X] T004 [P] Inventory current Codex skills in specs/001-codex-native-migration/codex-inventory.md
- [X] T005 [P] Record current docs/tooling references to Claude and Anthropic in specs/001-codex-native-migration/reference-inventory.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish shared source material that both Codex and Claude wrappers can read.

**CRITICAL**: No user story work should begin until these shared guidance files exist.

- [X] T006 Extract candidate profile and repo operating guidance from CLAUDE.md into agent-guidance/project-guide.md
- [X] T007 Extract application workflow rules from .claude/commands/apply.md into agent-guidance/workflows/job-apply.md
- [X] T008 Extract setup workflow rules from .claude/commands/setup.md into agent-guidance/workflows/job-setup.md
- [X] T009 Extract scrape workflow rules from .claude/skills/job-scraper/SKILL.md into agent-guidance/workflows/job-scrape.md
- [X] T010 [P] Extract ranking workflow rules from .claude/commands/rank.md into agent-guidance/workflows/job-rank.md
- [X] T011 [P] Extract interview workflow rules from .claude/commands/interview.md into agent-guidance/workflows/job-interview.md
- [X] T012 [P] Extract outcome workflow rules from .claude/commands/outcome.md into agent-guidance/workflows/job-outcome.md
- [X] T013 [P] Extract expand workflow rules from .claude/commands/expand.md into agent-guidance/workflows/job-expand.md
- [X] T014 [P] Extract upskill workflow rules from .claude/skills/upskill/SKILL.md into agent-guidance/workflows/job-upskill.md
- [X] T015 [P] Extract add-template workflow rules from .claude/commands/add-template.md into agent-guidance/workflows/job-add-template.md
- [X] T016 [P] Extract add-portal workflow rules from .claude/commands/add-portal.md into agent-guidance/workflows/job-add-portal.md
- [X] T017 [P] Extract reset workflow rules from .claude/commands/reset.md into agent-guidance/workflows/job-reset.md
- [X] T018 Extract job application reference files from .claude/skills/job-application-assistant/ into agent-guidance/job-application-assistant/

**Checkpoint**: Shared guidance exists and still contains the existing behavior rules.

---

## Phase 3: User Story 1 - Codex Users Can Run Job Workflows (Priority: P1) MVP

**Goal**: Codex users can invoke job-search and job-application workflows through native skills.

**Independent Test**: Each new `.agents/skills/job-*/SKILL.md` has valid frontmatter, describes when to use it, and points to the corresponding shared guidance file.

### Implementation for User Story 1

- [X] T019 [US1] Create Codex project guide in AGENTS.md from agent-guidance/project-guide.md
- [X] T020 [P] [US1] Create Codex setup skill in .agents/skills/job-setup/SKILL.md
- [X] T021 [P] [US1] Create Codex apply skill in .agents/skills/job-apply/SKILL.md
- [X] T022 [P] [US1] Create Codex scrape skill in .agents/skills/job-scrape/SKILL.md
- [X] T023 [P] [US1] Create Codex rank skill in .agents/skills/job-rank/SKILL.md
- [X] T024 [P] [US1] Create Codex interview skill in .agents/skills/job-interview/SKILL.md
- [X] T025 [P] [US1] Create Codex outcome skill in .agents/skills/job-outcome/SKILL.md
- [X] T026 [P] [US1] Create Codex expand skill in .agents/skills/job-expand/SKILL.md
- [X] T027 [P] [US1] Create Codex upskill skill in .agents/skills/job-upskill/SKILL.md
- [X] T028 [P] [US1] Create Codex add-template skill in .agents/skills/job-add-template/SKILL.md
- [X] T029 [P] [US1] Create Codex add-portal skill in .agents/skills/job-add-portal/SKILL.md
- [X] T030 [P] [US1] Create Codex reset skill in .agents/skills/job-reset/SKILL.md
- [X] T031 [US1] Create Codex shared application assistant skill in .agents/skills/job-application-assistant/SKILL.md
- [X] T032 [US1] Verify all new Codex skills reference agent-guidance/ files and existing output paths in specs/001-codex-native-migration/codex-skill-parity.md

**Checkpoint**: MVP complete; Codex has native skill wrappers for all job workflows.

---

## Phase 4: User Story 2 - Claude Users Keep Existing Workflows (Priority: P1)

**Goal**: Claude slash commands and skills continue to work while sharing the same canonical guidance as Codex.

**Independent Test**: Existing `.claude/commands/*.md` and `.claude/skills/*/SKILL.md` still exist, keep their command names, and point to the shared guidance where practical.

### Implementation for User Story 2

- [X] T033 [P] [US2] Adapt Claude setup command wrapper in .claude/commands/setup.md
- [X] T034 [P] [US2] Adapt Claude apply command wrapper in .claude/commands/apply.md
- [X] T035 [P] [US2] Adapt Claude scrape skill wrapper in .claude/skills/job-scraper/SKILL.md
- [X] T036 [P] [US2] Adapt Claude rank command wrapper in .claude/commands/rank.md
- [X] T037 [P] [US2] Adapt Claude interview command wrapper in .claude/commands/interview.md
- [X] T038 [P] [US2] Adapt Claude outcome command wrapper in .claude/commands/outcome.md
- [X] T039 [P] [US2] Adapt Claude expand command wrapper in .claude/commands/expand.md
- [X] T040 [P] [US2] Adapt Claude reset command wrapper in .claude/commands/reset.md
- [X] T041 [P] [US2] Adapt Claude add-template command wrapper in .claude/commands/add-template.md
- [X] T042 [P] [US2] Adapt Claude add-portal command wrapper in .claude/commands/add-portal.md
- [X] T043 [P] [US2] Adapt Claude upskill wrapper in .claude/skills/upskill/SKILL.md
- [X] T044 [US2] Verify Claude compatibility command names and skill names in specs/001-codex-native-migration/claude-compatibility.md

**Checkpoint**: Claude compatibility remains intact.

---

## Phase 5: User Story 3 - Users Can Discover Skills From a Guide (Priority: P2)

**Goal**: Users can open a single `guide.md` and understand every available skill, its purpose, and its Claude mapping.

**Independent Test**: `guide.md` lists every `.agents/skills/*/SKILL.md` skill and every legacy Claude command mapping agreed in the plan.

### Implementation for User Story 3

- [X] T045 [US3] Create root skill catalog in guide.md
- [X] T046 [US3] Add Codex job workflow skills table to guide.md
- [X] T047 [US3] Add existing portal-search skills table to guide.md
- [X] T048 [US3] Add existing speckit skills table to guide.md
- [X] T049 [US3] Add legacy Claude command mapping table to guide.md
- [X] T050 [US3] Cross-check guide.md against .agents/skills/*/SKILL.md and record result in specs/001-codex-native-migration/guide-audit.md

**Checkpoint**: Skill discovery is complete and independently reviewable.

---

## Phase 6: User Story 4 - Documentation Presents Codex Primary With Dual Support (Priority: P2)

**Goal**: README and setup docs make Codex the primary path while preserving Claude compatibility instructions.

**Independent Test**: A new user can follow Codex setup from README.md, while Claude users can still find the old slash-command path.

### Implementation for User Story 4

- [X] T051 [US4] Update primary toolchain and positioning in README.md
- [X] T052 [US4] Update quick start and workflow invocation examples in README.md
- [X] T053 [US4] Update file structure and skill catalog references in README.md
- [X] T054 [US4] Add Claude compatibility section to README.md
- [X] T055 [US4] Update Codex-first installation and onboarding in SETUP.md
- [X] T056 [US4] Preserve Claude compatibility installation notes in SETUP.md
- [X] T057 [US4] Update contributor workflow and review expectations in CONTRIBUTING.md
- [X] T058 [US4] Update Claude/Codex comments and personal-data notes in .gitignore

**Checkpoint**: Documentation reflects the dual-support product decision.

---

## Phase 7: User Story 5 - CI and Guards Validate the New Surface (Priority: P3)

**Goal**: Lint/security tooling catches drift across Codex skills, Claude wrappers, shared guidance, and guide.md.

**Independent Test**: Local validation commands pass and fail clearly if a new skill is missing from `guide.md`.

### Implementation for User Story 5

- [X] T059 [US5] Update skill and command lint rules in tools/lint_skills.py
- [X] T060 [US5] Add guide.md completeness validation to tools/lint_skills.py
- [X] T061 [US5] Update security guard wording for Codex-primary dual support in tools/security_guards.py
- [X] T062 [US5] Add Codex skill guardrails to tools/security_guards.py
- [X] T063 [US5] Update lint and placeholder paths in .github/workflows/ci.yml
- [X] T064 [US5] Update CI comments from Claude-only to Codex-primary dual support in .github/workflows/ci.yml

**Checkpoint**: Tooling protects both supported workflow surfaces.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Complete parity checks and run validation.

- [X] T065 [P] Validate no unintended Anthropic-only runtime dependency remains for Codex usage in specs/001-codex-native-migration/dependency-audit.md
- [X] T066 [P] Validate all personal output paths remain gitignored in .gitignore
- [X] T067 Run skill lint validation with python tools/lint_skills.py
- [X] T068 Run security validation with python tools/security_guards.py
- [X] T069 Run Python test suite with python -m pytest tests
- [X] T070 Typecheck all portal CLIs under .agents/skills/*/cli
- [X] T071 Review git diff for accidental changes outside migration scope in specs/001-codex-native-migration/final-review.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; can start immediately.
- **Foundational (Phase 2)**: Depends on Phase 1; blocks user story work.
- **US1 and US2**: Depend on Phase 2; can run in parallel after shared guidance exists.
- **US3**: Depends on US1 enough to know final Codex skill names.
- **US4**: Depends on US1 and US2 enough to document both surfaces accurately.
- **US5**: Depends on US1, US2, US3, and US4 because tooling validates final paths and catalog content.
- **Polish**: Depends on selected user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Codex-native workflow skills; MVP.
- **US2 (P1)**: Claude compatibility; can be implemented alongside US1 after shared guidance.
- **US3 (P2)**: Skill guide; depends on final skill inventory from US1.
- **US4 (P2)**: Docs; depends on US1 and US2 naming decisions.
- **US5 (P3)**: Tooling; depends on final docs and skill locations.

### Parallel Opportunities

- T002-T005 can run in parallel.
- T010-T017 can run in parallel once shared directory structure exists.
- T020-T030 can run in parallel because each creates a different Codex skill.
- T033-T043 can run in parallel because each adapts a different Claude wrapper.
- T051-T058 can be split by documentation file.
- T059-T064 can be split between lint, security, and CI work.

---

## Parallel Example: User Story 1

```text
Task: "Create Codex setup skill in .agents/skills/job-setup/SKILL.md"
Task: "Create Codex apply skill in .agents/skills/job-apply/SKILL.md"
Task: "Create Codex scrape skill in .agents/skills/job-scrape/SKILL.md"
Task: "Create Codex rank skill in .agents/skills/job-rank/SKILL.md"
Task: "Create Codex interview skill in .agents/skills/job-interview/SKILL.md"
```

---

## Implementation Strategy

### MVP First

1. Complete Phase 1 and Phase 2.
2. Complete US1 so Codex has native workflow skills.
3. Validate every new skill has frontmatter and shared-guidance references.
4. Stop and review before adapting Claude wrappers.

### Incremental Delivery

1. Shared guidance.
2. Codex skill surface.
3. Claude compatibility wrappers.
4. `guide.md` skill catalog.
5. README/SETUP/CONTRIBUTING updates.
6. Lint/security/CI updates.

### Notes

- Do not delete `.claude/` because dual support is required.
- Do not change existing portal CLI behavior unless validation exposes a migration-specific issue.
- Keep generated personal artifacts and gitignore behavior unchanged.
- Keep `.specify/` Codex/spec-kit files unless documentation wording needs cleanup.
