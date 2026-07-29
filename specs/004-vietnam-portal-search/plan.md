# Implementation Plan: Vietnam Job Portal Search Coverage

**Feature Branch Identifier**: `004-vietnam-portal-search` (no Git branch hook configured; current working branch is `codex-migration`) | **Date**: 2026-07-28 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-vietnam-portal-search/spec.md`

## Summary

Add CareerViet, Vieclam24h, TopCV, ITviec, and VietnamWorks as first-class,
independently reportable sources in the existing `job-scrape` workflow. Each
portal receives its own TypeScript/Bun search skill and access policy, while a
new `job-scrape` CLI owns cross-source execution, normalization, verification,
conservative deduplication, history reconciliation, and the coverage report.

The implementation is access-aware. ITviec and TopCV use low-volume public
search/detail retrieval. CareerViet defaults to manual-only because its current
terms prohibit crawler/robot searching. Vieclam24h and VietnamWorks default to
restricted/manual-required because ordinary unauthenticated requests currently
return 403. Manual and restricted sources still produce explicit coverage
statuses and official discovery links, and user-supplied public job URLs can be
verified without bypassing protected controls.

## Technical Context

**Language/Version**: TypeScript 5.x (currently `^5.4.0`) executed by Bun for
portal and orchestration CLIs; Python 3.13 for repository validation.

**Primary Dependencies**: Bun runtime and built-in `fetch`, `URL`,
`URLSearchParams`, child-process, and file APIs; TypeScript and `@types/bun` for
development; Python standard library and pytest-based repository tests. New
portal packages remain zero-runtime-dependency unless a portal fixture proves a
small HTML parser is necessary.

**Storage**: File-backed, private search state under gitignored `job_scraper/`,
including session reports and `seen_jobs.json`; optional read-only
reconciliation with gitignored `job_search_tracker.csv`. Source skill code,
sanitized fixtures, contracts, and documentation remain tracked.

**Testing**: Bun unit/contract tests with mocked fetch and sanitized fixtures;
`tsc --noEmit` per new CLI; opt-in low-volume live smoke tests; repository
validation with `python tools/lint_skills.py`,
`python tools/security_guards.py`, and `python -m pytest tests`.

**Target Platform**: Local Codex job-search workspace, primarily Windows
PowerShell, accessing permitted public HTTPS pages without authenticated
sessions.

**Project Type**: Codex skill workspace with independent portal CLIs and one
cross-source orchestration CLI.

**Performance Goals**: Attempt every enabled source and produce its status even
when another source fails; run independent public adapters concurrently with
bounded per-source timeouts; produce an actionable shortlist within the
specification's five-minute user-effort target for a normal single-user search.

**Constraints**: Low-volume personal use only. Do not bypass login, CAPTCHA,
403, paid access, robots restrictions, or portal terms. Do not store full copied
portal pages when normalized evidence is sufficient. Do not infer missing job
facts. Do not score title/snippet-only, inaccessible, expired, or
insufficient-detail listings. Preserve Claude compatibility and keep personal
outputs gitignored.

**Scale/Scope**: One user; five new Vietnam portal sources plus existing
LinkedIn, FreeHire, and Google discovery; typically tens of discoveries and a
small verified shortlist per run. Automated application, account management,
credential storage, and private/mobile API reverse engineering are out of scope.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution still contains template placeholders and defines no
enforceable project-specific gates. Repository rules from `AGENTS.md` and
`agent-guidance/project-guide.md` therefore govern this plan:

- Candidate facts, company facts, job requirements, and availability are never
  invented.
- Company-specific claims require source verification.
- Personal and generated search outputs stay in gitignored paths.
- Existing Claude compatibility files remain intact.
- Portal-specific skills are preferred over generic discovery.
- Protected access is not bypassed, and source failure degrades independently.

Gate status before research: **PASS**. No violations.

## Project Structure

### Documentation (this feature)

```text
specs/004-vietnam-portal-search/
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   |-- portal-adapter-cli.md
|   `-- job-scrape-output.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md                         # created later by /speckit-tasks
```

### Source Code (repository root)

```text
.agents/skills/
|-- careerviet-search/
|   |-- SKILL.md
|   |-- url-reference.md
|   `-- cli/
|       |-- package.json
|       |-- tsconfig.json
|       |-- src/{cli.ts,helpers.ts,commands/}
|       `-- tests/{fixtures/,*.test.ts}
|-- vieclam24h-search/               # same portal package shape
|-- topcv-search/                    # same portal package shape
|-- itviec-search/                   # same portal package shape
|-- vietnamworks-search/             # same portal package shape
`-- job-scrape/
    |-- SKILL.md
    `-- cli/
        |-- package.json
        |-- tsconfig.json
        |-- src/
        |   |-- cli.ts
        |   |-- contracts.ts
        |   |-- portal-registry.ts
        |   |-- run-portal.ts
        |   |-- normalize.ts
        |   |-- verify.ts
        |   |-- deduplicate.ts
        |   |-- history.ts
        |   |-- render.ts
        |   `-- commands/search.ts
        `-- tests/
            |-- contract.test.ts
            |-- orchestration.test.ts
            |-- graceful-degradation.test.ts
            |-- verification.test.ts
            |-- deduplication.test.ts
            `-- history.test.ts

agent-guidance/workflows/job-scrape.md
guide.md
.github/workflows/ci.yml

job_scraper/                        # gitignored runtime output
|-- seen_jobs.json
`-- sessions/<timestamp>.json

job_search_tracker.csv              # optional gitignored existing state
```

Each portal package follows the current lightweight `linkedin-search` and
`freehire-search` pattern: `search` and `detail` commands, JSON/table/plain
formats, structured stderr errors, null for unknown facts, fixture-driven
tests, and a portal-specific `url-reference.md`. Manual/restricted packages use
the same contract but return an expected source status and official review URL
instead of attempting prohibited access.

**Structure Decision**: Keep source markup, URLs, and access rules isolated in
five independent portal skills. Extend the existing `job-scrape` skill with the
only shared orchestration package, which adapts both new rich envelopes and
legacy LinkedIn/FreeHire output. This avoids rewriting working adapters while
making verification, deduplication, state handling, and coverage reporting
deterministic and testable.

## Phase 0 Research

See [research.md](./research.md).

Research resolved:

- current portal access modes and their policy implications;
- the five-skill plus one-orchestrator architecture;
- the normalization boundary for legacy and new adapter outputs;
- verification and experience-evidence gates;
- conservative cross-portal deduplication and conflict preservation;
- private state schemas and existing tracker reconciliation;
- deterministic fixture testing plus opt-in live smoke checks.

All research questions are resolved. Live discovery targets remain conditional
on each portal's access and terms: success criterion SC-002 is
exercised only when its stated precondition (public matching listings on each
tested source) is true; restricted sources otherwise report their honest status
and can accept user-supplied public detail URLs.

## Phase 1 Design

See:

- [data-model.md](./data-model.md)
- [contracts/portal-adapter-cli.md](./contracts/portal-adapter-cli.md)
- [contracts/job-scrape-output.md](./contracts/job-scrape-output.md)
- [quickstart.md](./quickstart.md)

The design defines portal capabilities and access modes, search-run and
source-listing records, verification states, source-backed field evidence,
deduplication conflicts, fit gates, history links, adapter/orchestrator CLI
contracts, private output files, and acceptance validation scenarios.

## Agent Context Update

No agent-context update script exists under `.specify/scripts/`; the available
scripts are limited to prerequisites, common helpers, feature creation, plan
setup, and task setup. The implementation will update the repository's actual
agent context surfaces—`guide.md`, `.agents/skills/job-scrape/SKILL.md`, and
`agent-guidance/workflows/job-scrape.md`—when the new skills are added.

## Post-Design Constitution Check

Gate status after design: **PASS**.

- Personal search sessions and history remain in existing gitignored locations.
- Public adapters are low-volume, link-first, and source-attributed.
- Manual/restricted modes prevent policy or access bypass.
- Unknown and conflicting facts stay explicit.
- The fit gate requires active, accessible, sufficient source evidence.
- Existing Claude compatibility remains unchanged.

## Complexity Tracking

No constitution violations require justification.
