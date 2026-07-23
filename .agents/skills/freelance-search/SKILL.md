---
name: freelance-search
version: 1.0.0
description: >
  Search, rank, evaluate, draft for, and track freelance opportunities across
  Upwork, Fiverr, Google-discovered public pages, and user-supplied sources.
  Treats platform readiness and source verification as first-class signals.
context: fork
allowed-tools: Bash(bun run .agents/skills/freelance-search/cli/src/cli.ts *)
---

# Freelance Search Skill

Use this skill to find freelance projects, contract gigs, client requests, and
service leads from configured sources. It extends the default profile with
freelance preferences such as target services, rates, availability, portfolio
references, platform profiles, and exclusions.

## Safety Rules

- Do not store platform passwords, payment credentials, private messages, or
  secret tokens.
- Do not invent freelancer facts, client facts, project details, platform
  ratings, earnings, or availability.
- Treat Google and social results as discovery leads until the original posting
  or user-supplied content is verified.
- Fiverr and Upwork opportunities must surface account readiness or
  setup-required status before action is recommended.

## Commands

```bash
bun run .agents/skills/freelance-search/cli/src/cli.ts search --profile documents/freelance-profile.json --format table
bun run .agents/skills/freelance-search/cli/src/cli.ts readiness --platform upwork --account-status limited --known-blocker "Connects balance unknown"
bun run .agents/skills/freelance-search/cli/src/cli.ts evaluate --opportunity job_scraper/freelance/opportunity.json --profile documents/freelance-profile.json
bun run .agents/skills/freelance-search/cli/src/cli.ts proposal --opportunity job_scraper/freelance/opportunity.json --profile documents/freelance-profile.json
bun run .agents/skills/freelance-search/cli/src/cli.ts save --opportunity job_scraper/freelance/opportunity.json --status proposal_drafted
```

## Usage Guidance

- Start with `readiness` for Fiverr and Upwork.
- Use `search` for profile-driven discovery and ranked source statuses.
- Use `evaluate` before drafting when opportunity details are partial.
- Use `proposal` only for verified or user-supplied opportunity details.
- Use `save` to preserve status and avoid duplicate freelance outreach.
