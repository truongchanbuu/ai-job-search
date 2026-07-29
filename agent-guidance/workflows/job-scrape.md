# Job Scrape Workflow

Canonical workflow for discovering job postings.

## Inputs

- Candidate profile and target role preferences.
- Search query guidance.
- Portal CLI skills under `.agents/skills/*-search`.

## Steps

1. Load profile and search preferences.
2. Prefer portal CLI skills over generic web search.
3. For Vietnam searches, run the `job-scrape` orchestration CLI so CareerViet,
   Vieclam24h, TopCV, ITviec, and VietnamWorks each receive an explicit status.
4. Treat TopCV and ITviec as low-volume public sources while their recorded
   access review remains current.
5. Treat CareerViet as manual-only and Vieclam24h/VietnamWorks as
   restricted/manual-required unless a later permitted access review changes
   their registry mode.
6. Use `google-search` as supplemental discovery when portal results are sparse,
   overly senior, or the user asks for Google coverage.
7. Fetch only promising permitted public postings for details.
8. Verify active status, sufficient requirements, and source-backed experience
   evidence before fit screening.
9. Deduplicate against `job_scraper/seen_jobs.json` and existing tracker data
   while preserving every source copy and conflict.
10. Present actionable matches separately from unverified/non-actionable leads,
    followed by per-source coverage and next actions.

## Rules

- Only present jobs found through actual search results.
- Do not score expired or inaccessible postings from title alone.
- Treat Google results as discovery leads until the original posting page is opened and verified.
- Treat all portal search cards as discovery leads until permitted detail
  verification succeeds.
- Never translate login, CAPTCHA, 401/403, timeout, or parse failure into
  `no_matches`.
- Explicit description requirements override title/category labels such as
  "junior" or "fresher".
- A maximum-experience recommendation requires a preserved source evidence
  reference.
- Do not use credentials, authenticated cookies, private/mobile APIs, rotating
  proxies, or CAPTCHA solving.
- Store normalized evidence and canonical links, not full mirrored portal
  pages.
- Keep generated scrape state in gitignored locations.
