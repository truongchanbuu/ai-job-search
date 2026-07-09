# Implementation Plan: Job Search Suite

**Branch**: `002-job-search-suite` | **Date**: 2026-07-09 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-job-search-suite/spec.md`

## Summary

Extend the existing Codex job-search workspace into a profile-driven job application suite. The implementation will add and update Codex skills and portal CLIs so a single default profile can drive multi-source job discovery, bilingual CV generation, ATS keyword review, application history tracking, and interview preparation from job descriptions plus related public-post context.

The technical approach keeps the repository's current shape: Codex skills under `.agents/skills/`, shared workflow rules under `agent-guidance/`, personal outputs under gitignored `documents/`, `cv/`, `cover_letters/`, and `job_search_tracker.csv`, Python validation tools under `tools/`, and TypeScript/Bun CLIs for source-specific job discovery.

## Technical Context

**Language/Version**: Python 3.13 for repository validation and data utilities; TypeScript 5.x with Bun-compatible CLIs for job source integrations; LaTeX for CV and cover-letter outputs.

**Primary Dependencies**: Existing Codex skill format, existing `*-search` portal CLI pattern, TypeScript compiler, Bun runtime for portal CLIs, Python standard library plus existing pytest-based tests, existing LaTeX compilation workflow.

**Storage**: File-based workspace storage in gitignored personal paths: default profile data under `documents/`, generated CVs under `cv/`, generated cover letters under `cover_letters/`, application archives under `documents/applications/`, scrape state under `job_scraper/`, and tracker state in `job_search_tracker.csv`.

**Testing**: Python `pytest` for repository utilities; `python tools/lint_skills.py`; `python tools/security_guards.py`; TypeScript `tsc --noEmit` for portal CLIs; focused CLI unit tests using each portal CLI's existing test pattern; manual/fixture quickstart scenarios for end-to-end workflow validation.

**Target Platform**: Local developer/job-search workspace on Windows PowerShell, with cross-platform-friendly scripts and path guidance where practical.

**Project Type**: Codex skill workspace with source-specific CLI tools and document-generation workflows.

**Performance Goals**: A profile-driven search should return a ranked shortlist within 5 minutes of user effort; ATS keyword review should complete within 2 minutes of user effort; deduplication should handle representative multi-source results with at least 90% accuracy.

**Constraints**: Do not invent candidate facts, company facts, job requirements, or public-post claims. Respect source availability and access restrictions. Keep personal outputs gitignored. Preserve Claude compatibility wrappers. Prefer source-specific skills over generic search when available. Any external source access must degrade gracefully when blocked, unavailable, or not authorized.

**Scale/Scope**: Single-user job-search workspace; initial scope covers LinkedIn, VietnamWorks, TopCV, ITviec, Vieclam24h, Google-discovered public pages, and user-supplied or accessible Facebook page/group content.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The constitution file currently contains only template placeholders and no enforceable project-specific gates. The plan therefore applies repository guidance from `AGENTS.md` and `agent-guidance/project-guide.md`:

- Preserve Claude compatibility files unless explicitly removed.
- Do not invent candidate facts, company facts, job requirements, or learning resources.
- Verify company-specific claims before using them in application materials.
- Keep personal/generated outputs in gitignored paths.
- Compile and inspect generated PDFs before presenting final application materials.
- Prefer existing Codex skills and local workflow guidance over new abstractions.

Gate status before research: PASS. No violations.

## Project Structure

### Documentation (this feature)

```text
specs/002-job-search-suite/
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   |-- job-search-suite-cli.md
|   `-- workflow-outputs.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
.agents/skills/
|-- job-scrape/
|-- job-apply/
|-- job-interview/
|-- job-outcome/
|-- job-application-assistant/
|-- linkedin-search/
|-- vietnamworks-search/        # planned new portal skill
|-- topcv-search/               # planned new portal skill
|-- itviec-search/              # planned new portal skill
|-- vieclam24h-search/          # planned new portal skill
|-- google-search/              # planned discovery/context skill
`-- facebook-search/            # planned public/user-supplied context skill

agent-guidance/
|-- workflows/
|   |-- job-scrape.md
|   |-- job-apply.md
|   |-- job-interview.md
|   `-- job-outcome.md
`-- job-application-assistant/

documents/
|-- applications/               # gitignored user application archive
|-- cv/                         # gitignored source profile/CV inputs
`-- linkedin/                   # gitignored profile exports

cv/                             # generated CV TeX, examples tracked
cover_letters/                  # generated cover-letter TeX, examples tracked
job_scraper/                    # gitignored seen jobs and scrape notes
tools/                          # lint/security/data validation utilities
tests/                          # Python utility tests
```

**Structure Decision**: Extend the existing skill-based workspace. New job sources should follow the existing `.agents/skills/*-search` pattern with a `SKILL.md`, optional `url-reference.md`, and a `cli/` package when automated search/detail retrieval is feasible. Cross-cutting profile, ATS, history, and interview behavior should live in shared `agent-guidance/` workflow docs and be invoked through existing job workflow skills.

## Phase 0 Research

See [research.md](./research.md).

Research resolved source-access and architecture questions:

- Source access must be adapter-based with source-level status reporting.
- LinkedIn support should keep the existing personal-use public-listing skill and avoid representing it as an official public jobs API.
- Google discovery cannot depend on new Custom Search JSON API onboarding because official docs state it is closed to new customers and has a transition deadline.
- Facebook support should accept user-configured accessible content and user-supplied posts; private/restricted content is out of scope unless supplied by the user.
- Vietnamese portals should be implemented as separate portal skills after per-site access investigation.
- Bilingual CV and interview prep should be extensions of existing application/interview guidance, not separate profile systems.

No unresolved NEEDS CLARIFICATION items remain.

## Phase 1 Design

See:

- [data-model.md](./data-model.md)
- [contracts/job-search-suite-cli.md](./contracts/job-search-suite-cli.md)
- [contracts/workflow-outputs.md](./contracts/workflow-outputs.md)
- [quickstart.md](./quickstart.md)

The design defines file-backed entities, CLI command contracts for source adapters and cross-source workflows, expected output records, state transitions for application history, and validation scenarios.

## Agent Context Update

No `.specify` agent-context update script exists in this repository. The relevant agent context is already expressed through `AGENTS.md`, `guide.md`, and `agent-guidance/`. Implementation tasks should update those files when new skills or workflows are added.

## Post-Design Constitution Check

Gate status after design: PASS.

- Personal outputs remain assigned to gitignored locations.
- Existing Claude compatibility remains in scope.
- Source access constraints are explicit and require graceful degradation.
- The design preserves factuality rules for candidate facts, company facts, job requirements, and public-post context.

## Complexity Tracking

No constitution violations require justification.
