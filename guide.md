# AI Job Search Skill Guide

This guide lists the skills available in this repository. Codex skills are the primary interface. Claude Code slash commands remain available as a compatibility layer.

## Codex Job Workflow Skills

| Skill | Purpose | When to Use | Legacy Claude Command |
|---|---|---|---|
| `job-setup` | Onboard or update the candidate profile. | Use when setting up the workspace from documents, a CV, or interview answers. | `/setup` |
| `job-apply` | Evaluate a posting and produce verified CV/cover-letter drafts. | Use when applying to a specific job URL or pasted posting. | `/apply` |
| `job-scrape` | Search job portals and present deduplicated matches. | Use when finding new jobs. | `/scrape` |
| `job-rank` | Batch-rank scraped postings into a shortlist. | Use when there are many matches to triage. | `/rank` |
| `job-interview` | Prepare interview materials from tracked application context. | Use before a scheduled interview. | `/interview` |
| `job-outcome` | Record application results and archive context. | Use after an application stage changes. | `/outcome` |
| `job-expand` | Add source-backed competency signals to the profile. | Use after adding documents or public profile links. | `/expand` |
| `job-upskill` | Analyze skill gaps and produce a learning plan. | Use for career planning from postings or tracker data. | `/upskill` |
| `job-add-template` | Register a custom LaTeX CV or cover-letter template. | Use when replacing the stock templates. | `/add-template` |
| `job-add-portal` | Scaffold a search skill for a new job portal. | Use when adding a local or niche job board. | `/add-portal` |
| `job-reset` | Reset profile or document state with confirmation. | Use when starting over or clearing personal data. | `/reset` |
| `job-application-assistant` | Shared profile, writing, CV, cover-letter, evaluation, and interview guidance. | Use as reference material for job workflows. | `Skill(job-application-assistant)` |
| `freelance-search` | Search, rank, evaluate, draft for, and track freelance opportunities. | Use for Fiverr, Upwork, Google/social freelance leads, proposal triage, and opportunity history. | None |

## Portal Search Skills

| Skill | Purpose | When to Use | Legacy Claude Command |
|---|---|---|---|
| `freehire-search` | Search the freehire.dev tech-job aggregator across markets. | Use for software, data, engineering, DevOps, and remote tech roles. | None |
| `google-search` | Generate targeted Google job-search links or search via Google Programmable Search API. | Use as a supplemental discovery source after portal skills, especially for company career pages and ATS postings. | None |
| `upwork-search` | Generate Upwork discovery links and manual freelance lead records. | Use for Upwork freelance discovery with readiness and Connects caveats. | None |
| `fiverr-search` | Generate Fiverr marketplace discovery links and manual freelance lead records. | Use for Fiverr gig/service discovery with seller profile readiness caveats. | None |
| `linkedin-search` | Search public LinkedIn job listings by role and location. | Use for broad job searches in any market. | None |
| `jobindex-search` | Search Jobindex.dk. | Use for Danish job listings. | None |
| `jobnet-search` | Search Denmark's public Jobnet portal. | Use for Danish public job listings. | None |
| `jobbank-search` | Search Akademikernes Jobbank. | Use for academic and highly educated roles in Denmark. | None |
| `jobdanmark-search` | Search Jobdanmark.dk. | Use for Danish job listings by category, location, or query. | None |

## Spec-Kit Skills

| Skill | Purpose | When to Use | Legacy Claude Command |
|---|---|---|---|
| `speckit-specify` | Create or update a feature specification. | Use when turning a feature idea into a spec. | None |
| `speckit-plan` | Create an implementation plan and design artifacts. | Use after a spec is ready. | None |
| `speckit-tasks` | Generate dependency-ordered implementation tasks. | Use after planning. | None |
| `speckit-implement` | Execute tasks from `tasks.md`. | Use when ready to implement generated tasks. | None |
| `speckit-clarify` | Resolve underspecified requirements. | Use when the spec has ambiguity. | None |
| `speckit-checklist` | Create a feature checklist. | Use for quality gates and review criteria. | None |
| `speckit-analyze` | Analyze consistency across spec, plan, and tasks. | Use before implementation or review. | None |
| `speckit-converge` | Add remaining unbuilt work to tasks. | Use when implementation and plans diverge. | None |
| `speckit-constitution` | Create or update project governance. | Use for project principles and constraints. | None |
| `speckit-taskstoissues` | Convert tasks into GitHub issues. | Use when tracking work in GitHub. | None |

## Compatibility Notes

- Codex is the primary workflow interface.
- Claude Code support remains under `.claude/`.
- Shared workflow guidance lives under `agent-guidance/`.
- Personal outputs remain gitignored.
