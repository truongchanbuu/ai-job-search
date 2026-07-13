# AI Job Search Agent Guide

This repository is a job application workspace. Agents help the user evaluate job postings, tailor LaTeX CVs and cover letters, search job portals, prepare interviews, track outcomes, and identify skill gaps.

## Core Responsibilities

- Evaluate job fit before drafting application material.
- Tailor CVs in `cv/` and cover letters in `cover_letters/`.
- Use the profile and reference guidance in `agent-guidance/job-application-assistant/`.
- Search jobs using the Codex portal skills in `.agents/skills/*-search`; use `google-search` as supplemental discovery for indexed career pages and ATS postings.
- Keep personal outputs in gitignored locations such as `documents/applications/`, `job_search_tracker.csv`, generated CVs, and generated cover letters.

## Verification Rules

- Factual claims must match the candidate profile and source documents.
- Company-specific claims must be verified before inclusion in CVs, cover letters, or interview prep.
- CVs must compile with `lualatex`; cover letters must compile with `xelatex`.
- Generated PDFs must be visually inspected before delivery.
- If `pdftotext` is available, check CV text extraction for ATS readability.
- Do not fabricate skills, job requirements, postings, company facts, or study resources.

## Dual Support

Codex is the primary workflow surface. Claude Code remains supported through compatibility wrappers under `.claude/`.

Use `guide.md` for a catalog of available skills and legacy Claude command mappings.
