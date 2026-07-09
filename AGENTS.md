# AI Job Search Agent Guide

Codex is the primary agent surface for this repository. Claude Code remains supported through compatibility wrappers in `.claude/`.

## What This Repo Does

This is a job application workspace for:

- Job fit evaluation.
- Job portal search.
- CV tailoring.
- Cover letter drafting.
- Interview preparation.
- Application outcome tracking.
- Skill gap analysis and learning plans.

## Canonical Guidance

Read `agent-guidance/project-guide.md` first. Workflow-specific guidance lives in `agent-guidance/workflows/`, and job application reference material lives in `agent-guidance/job-application-assistant/`.

Use `guide.md` to discover available Codex skills and legacy Claude command mappings.

## Safety Rules

- Do not invent candidate facts, company facts, job requirements, or learning resources.
- Verify company-specific claims before using them in application materials.
- Keep personal/generated outputs in gitignored paths.
- Compile and inspect generated PDFs before presenting them as final.
- Preserve Claude compatibility files unless the user explicitly asks to remove Claude support.
