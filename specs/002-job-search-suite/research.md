# Research: Job Search Suite

## Decision: Keep the workspace skill-based instead of adding a web application

**Rationale**: The repository already works through Codex skills, shared workflow guidance, portal CLIs, LaTeX templates, Python validation scripts, and gitignored personal outputs. Extending those surfaces keeps implementation aligned with existing usage and avoids an unrelated application shell.

**Alternatives considered**:

- New web app: rejected because the current user workflow is agent/CLI/document driven.
- Single monolithic scraper: rejected because each source has different access rules, result shapes, and failure modes.

## Decision: Use source-specific adapters with a shared normalized job record

**Rationale**: LinkedIn, Vietnamese portals, Google-discovered pages, and Facebook pages/groups have different access patterns. A shared normalized record lets downstream ranking, deduplication, ATS checks, history, and interview prep operate consistently while each adapter reports its own limitations.

**Alternatives considered**:

- Force every source into one CLI: rejected because failures and authentication constraints would be harder to isolate.
- Treat search results as unstructured markdown only: rejected because deduplication, ranking, and history matching require stable fields.

## Decision: Treat LinkedIn as a constrained personal-use source

**Rationale**: The existing `linkedin-search` skill uses public job pages and already warns that automated access is for personal use only. Official LinkedIn Talent Solutions documentation focuses on partner ATS/recruiting integrations such as Apply Connect, Apply with LinkedIn, and Job Posting ingestion, not an open job-seeker search API. The feature should keep LinkedIn source support explicit about limitations and graceful failure.

**Sources**:

- LinkedIn Talent Solutions overview: https://learn.microsoft.com/en-us/linkedin/talent/
- Existing local skill: `.agents/skills/linkedin-search/cli/README.md`

**Alternatives considered**:

- Official LinkedIn Talent APIs for job search: rejected for this single-user workspace because the documented APIs are partner/recruiting-oriented.
- Remove LinkedIn: rejected because the user explicitly requested it and an existing personal-use skill already exists.

## Decision: Do not rely on new Google Custom Search JSON API onboarding

**Rationale**: Google documentation says the Custom Search JSON API retrieves Programmable Search Engine results in JSON and requires a configured search engine plus API key, but also states it is closed to new customers and existing customers have a transition deadline of January 1, 2027. Google search support should therefore be planned as an optional adapter with user-provided credentials where available, plus a manual/user-supplied fallback for public result URLs.

**Source**:

- Google Custom Search JSON API overview: https://developers.google.com/custom-search/v1/overview

**Alternatives considered**:

- Require Google Custom Search JSON API for all users: rejected because it is not available to new customers.
- Scrape general Google result pages: rejected as a default because it is brittle and likely to violate access expectations.

## Decision: Scope Facebook support to accessible pages/groups and user-supplied posts

**Rationale**: Facebook page/group content access is permission-sensitive and can change by content visibility, group privacy, and platform rules. For this workspace, Facebook should provide context only when the user configures accessible sources or supplies post content. The system must label post-derived content as interview-preparation context rather than verified company facts.

**Alternatives considered**:

- Automated scraping of private groups: rejected because it is outside the safe and supportable scope.
- No Facebook support: rejected because public/user-supplied posts are useful for interview prep when clearly labeled.

## Decision: Add Vietnamese portals as separate skills after source investigation

**Rationale**: VietnamWorks, TopCV, ITviec, and Vieclam24h should each have its own source adapter because search URLs, detail pages, anti-abuse behavior, and result fields differ. The `job-add-portal` workflow already defines the expected local pattern for scaffolding portal skills.

**Alternatives considered**:

- One "vietnam-portals" adapter: rejected because source-specific tests and failure messages would be weaker.
- Generic web search only: rejected because dedicated portal skills can preserve source attribution and richer job fields.

## Decision: Make the default profile the search and generation anchor

**Rationale**: The feature depends on main skills, target roles, language preferences, verified experience, exclusions, and constraints. A single default profile avoids repeated search prompts and gives CV generation, ATS review, and interview prep a common factual boundary.

**Alternatives considered**:

- Multiple named profiles in v1: rejected to keep scope focused on one user and one default profile.
- Ad hoc prompt-only profile: rejected because history, ATS evidence, and factuality checks need persistent profile data.

## Decision: Implement ATS review as evidence-aware keyword analysis

**Rationale**: The ATS checker must improve truthful tailoring, not keyword stuffing. Recommendations should distinguish matched terms, missing job terms, weakly represented terms, and terms unsupported by the candidate profile.

**Alternatives considered**:

- Raw keyword count only: rejected because it encourages misleading CV edits.
- Fully automated CV rewrite without review: rejected because factuality and user approval matter.

## Decision: Extend application history instead of creating a separate tracker

**Rationale**: The repo already uses `job_search_tracker.csv` and archives application materials under `documents/applications/`. Extending this concept minimizes migration and keeps outcome tracking compatible with existing workflows.

**Alternatives considered**:

- New database: rejected for a single-user local workspace.
- Separate tracker per source: rejected because users need one application history across all sources.
