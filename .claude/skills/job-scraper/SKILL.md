---
name: scrape
description: >
  Claude compatibility skill for searching job portals, deduplicating results,
  and presenting fit-screened matches. Canonical guidance lives in
  agent-guidance/workflows/job-scrape.md.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash(bun --version), Bash(bun run .agents/skills/*/cli/src/cli.ts *), WebFetch, WebSearch, Agent, AskUserQuestion
---

# Job Scraper

Read `agent-guidance/project-guide.md` and `agent-guidance/workflows/job-scrape.md` before acting.

Preserve the legacy `/scrape` behavior: prefer existing portal CLI skills, fall back to web search only when needed, deduplicate results, and never fabricate postings.
