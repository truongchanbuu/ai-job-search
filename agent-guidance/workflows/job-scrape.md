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
4. Use `google-search` as a supplemental discovery source when portal results are sparse, overly senior, or the user asks for Google coverage.
5. Fetch only promising postings for details.
6. Deduplicate against `job_scraper/seen_jobs.json` and existing tracker data.
7. Estimate quick fit using the evaluation framework.
8. Present high-match postings and next actions.

## Rules

- Only present jobs found through actual search results.
- Do not score expired or inaccessible postings from title alone.
- Treat Google results as discovery leads until the original posting page is opened and verified.
- Keep generated scrape state in gitignored locations.
