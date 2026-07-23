# Implementation Plan: Freelance Job Search

**Branch**: `003-freelance-job-search` | **Date**: 2026-07-14 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-freelance-job-search/spec.md`

## Summary

Extend the existing job-search workspace so the same default candidate profile can drive freelance opportunity discovery, account-readiness checks, proposal preparation, and outcome tracking. The feature adds Fiverr and Upwork as explicit freelance platforms, keeps Google and social media as verification-required discovery sources, and introduces freelance-specific preferences such as target services, rate expectations, availability, portfolio links, platform blockers, and proposal history.

The technical approach keeps the current repository shape: Codex skills under `.agents/skills/`, shared workflow guidance under `agent-guidance/`, file-backed personal outputs in gitignored paths, TypeScript/Bun CLIs for source-specific discovery when feasible, and Python validation utilities for repository-level checks. Platform access must be conservative: private account data, credentials, messages, payment details, and restricted content stay user-controlled and are never stored in generated outputs.

## Technical Context

**Language/Version**: Python 3.13 for validation and data utilities; TypeScript 5.x with Bun-compatible CLIs for source-specific search helpers; LaTeX remains available for application materials, but freelance proposal outputs should initially be text/markdown artifacts.

**Primary Dependencies**: Existing Codex skill format, existing `.agents/skills/*-search` CLI pattern, Bun runtime and TypeScript compiler for source adapters, Python standard library plus existing pytest-based checks, existing `google-search` skill for public-page discovery.

**Storage**: File-based workspace storage in gitignored personal paths. Freelance profile extensions, account-readiness notes, proposal drafts, and opportunity history should live under designated personal-output locations such as `documents/`, `documents/applications/`, `job_scraper/`, and tracker files, without committing sensitive platform data.

**Testing**: Python `pytest` for shared utilities; `python tools/lint_skills.py`; `python tools/security_guards.py`; TypeScript `tsc --noEmit` and focused Bun unit tests for freelance source CLIs; manual/fixture quickstart scenarios for end-to-end workflow validation.

**Target Platform**: Local single-user job-search workspace on Windows PowerShell, with cross-platform-friendly CLI guidance where practical.

**Project Type**: Codex skill workspace with source-specific CLI tools, file-backed records, and document/proposal generation workflows.

**Performance Goals**: A freelance search should produce a ranked shortlist from at least two available source categories within 5 minutes of user effort; users should identify the top 5 proposal-worthy opportunities from a mixed-source result set within 10 minutes; deduplication should reach at least 90% accuracy in representative repeated listings.

**Constraints**: Do not invent freelancer facts, platform metrics, client facts, project requirements, earnings, ratings, or learning resources. Do not store platform passwords, payment credentials, private messages, or restricted account content. Treat Google/social results as discovery leads until original content is verified. Respect platform access restrictions and degrade gracefully when sources require login, setup, credits, verification, or user-supplied content. Preserve Claude compatibility wrappers.

**Scale/Scope**: Single-user freelance search extension. Initial scope covers Fiverr readiness/gig discovery, Upwork readiness/job discovery, Google-discovered public pages, user-configured social/community sources, user-supplied opportunities, ranking, proposal drafting, and freelance opportunity history.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The constitution file currently contains only template placeholders and no enforceable project-specific gates. The plan therefore applies repository guidance from `AGENTS.md` and `agent-guidance/project-guide.md`:

- Preserve Claude compatibility files unless explicitly removed.
- Do not invent candidate facts, company/client facts, job/project requirements, earnings, ratings, or learning resources.
- Verify platform/client/project-specific claims before using them in proposals or tracking decisions.
- Keep personal/generated outputs in gitignored paths.
- Prefer existing Codex skills and local workflow guidance over new abstractions.
- Respect external source availability and account-access limitations.

Gate status before research: PASS. No violations.

## Project Structure

### Documentation (this feature)

```text
specs/003-freelance-job-search/
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   |-- freelance-search-cli.md
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
|-- job-outcome/
|-- job-application-assistant/
|-- google-search/
|-- upwork-search/              # planned freelance source/readiness skill
|-- fiverr-search/              # planned freelance source/readiness skill
`-- freelance-search/           # planned cross-source freelance workflow skill

agent-guidance/
|-- workflows/
|   |-- job-scrape.md
|   |-- job-apply.md
|   `-- job-outcome.md
`-- job-application-assistant/

documents/
|-- applications/               # gitignored user application/freelance archive
|-- cv/                         # gitignored source profile/CV inputs
`-- linkedin/                   # gitignored profile exports

job_scraper/                    # gitignored seen jobs/opportunities and scrape notes
tools/                          # lint/security/data validation utilities
tests/                          # Python utility tests
```

**Structure Decision**: Extend the existing skill-based workspace. New Fiverr, Upwork, and cross-source freelance behavior should follow the `.agents/skills/*-search` pattern with `SKILL.md`, optional source references, and a `cli/` package only when automated search/detail retrieval is feasible and compliant. Cross-cutting profile, proposal, readiness, and history behavior should be captured in shared `agent-guidance/` workflow docs and invoked through Codex skills rather than a new web app.

## Phase 0 Research

See [research.md](./research.md).

Research resolved source-access and architecture questions:

- Freelance discovery should be adapter-based with source-level status reporting.
- Upwork support must account for account readiness and Connects before recommending proposal action.
- Fiverr support should start with account/gig readiness and public/manual discovery unless a supported access path is explicitly available.
- Google and social sources remain verification-required discovery leads, not final actionable records.
- Proposal drafting must be evidence-aware and must not fabricate freelancer claims, client facts, platform metrics, or project details.

No unresolved NEEDS CLARIFICATION items remain.

## Phase 1 Design

See:

- [data-model.md](./data-model.md)
- [contracts/freelance-search-cli.md](./contracts/freelance-search-cli.md)
- [contracts/workflow-outputs.md](./contracts/workflow-outputs.md)
- [quickstart.md](./quickstart.md)

The design defines file-backed freelance profile extensions, platform account readiness records, normalized freelance opportunities, fit scoring, proposal material, opportunity history, source reliability notes, CLI/workflow contracts, and validation scenarios.

## Agent Context Update

No `.specify` agent-context update script exists in this repository. The relevant agent context is expressed through `AGENTS.md`, `guide.md`, and `agent-guidance/`. Implementation tasks should update those files when new freelance skills or workflows are added.

## Post-Design Constitution Check

Gate status after design: PASS.

- Personal outputs remain assigned to gitignored locations.
- Existing Claude compatibility remains in scope.
- Source access constraints are explicit and require graceful degradation.
- The design preserves factuality rules for freelancer facts, client facts, project details, platform metrics, and proposal content.
- Sensitive account secrets are explicitly excluded from generated outputs.

## Complexity Tracking

No constitution violations require justification.
