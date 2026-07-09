# Job Scrape Workflow

Canonical workflow for discovering job postings.

## Inputs

- Candidate profile and target role preferences.
- Search query guidance.
- Portal CLI skills under `.agents/skills/*-search`.

## Steps

1. Load profile and search preferences.
2. Prefer portal CLI skills over generic web search.
3. Query configured portals and collect results.
4. Fetch only promising postings for details.
5. Deduplicate against `job_scraper/seen_jobs.json` and existing tracker data.
6. Estimate quick fit using the evaluation framework.
7. Present high-match postings and next actions.

## Rules

- Only present jobs found through actual search results.
- Do not score expired or inaccessible postings from title alone.
- Keep generated scrape state in gitignored locations.
