---
name: google-search
version: 1.0.0
description: >
  Search Google for job postings and job-board result pages using the official
  Programmable Search JSON API when credentials are configured, or generate
  targeted Google search URLs for manual review. Use as a supplemental discovery
  source after portal-specific skills.
context: fork
allowed-tools: Bash(bun run .agents/skills/google-search/cli/src/cli.ts *)
---

# Google Search Skill

Use this skill as a supplemental discovery source for job searches when portal
skills miss good postings. It is especially useful for finding company career
pages, ATS postings, and local job-board pages that are indexed by Google.
Do not use this skill to reproduce or scrape the Google Jobs UI; Google results
are discovery leads until the original posting page is opened and verified.

## Access Model

This skill does not scrape Google's result HTML. It supports:

- Official API search through Google Programmable Search JSON API.
- Query-link generation for manual Google review when API credentials are absent.
- Filtering out Google Jobs UI links so downstream workflows focus on original
  public posting pages.

Set these environment variables for live API results:

- `GOOGLE_API_KEY`
- `GOOGLE_CSE_ID`

## Commands

### Generate Google Search URLs

```bash
bun run .agents/skills/google-search/cli/src/cli.ts queries -q "Backend Engineer Node.js NestJS" -l "Ho Chi Minh City" --jobage 30 --format table
```

### Search With Google Programmable Search API

```bash
bun run .agents/skills/google-search/cli/src/cli.ts search -q "Backend Engineer Node.js NestJS" -l "Ho Chi Minh City" --jobage 30 --limit 10 --format json
```

Key flags:

- `--query <text>` / `-q <text>`: role, technology, or keyword query.
- `--location <text>` / `-l <text>`: city, country, or remote location text.
- `--jobage <days>`: adds recency wording to generated queries and maps to API `dateRestrict=dN`.
- `--site <domain>`: repeatable site filter, e.g. `--site linkedin.com/jobs --site careers.smartrecruiters.com`.
- `--limit <n>` / `-n <n>`: result cap, max 10 for one API call.
- `--format json|table|plain`: default `json`.

## Usage Guidance

- Prefer dedicated portal skills first when they exist.
- Use Google to discover candidate URLs, then fetch/verify the original posting
  before scoring or drafting application material.
- Treat every Google result as a verification-required discovery lead, not a
  final source record.
- Keep query volume low and prefer targeted site filters.
