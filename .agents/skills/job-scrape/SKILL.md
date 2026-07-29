---
name: job-scrape
description: Search configured job portals, deduplicate results, and present fit-screened matches. Use when the user wants to find new jobs.
allowed-tools: Bash(bun run .agents/skills/job-scrape/cli/src/cli.ts *)
---

# Job Scrape

Read `agent-guidance/project-guide.md` and `agent-guidance/workflows/job-scrape.md` before acting.

Prefer existing portal skills in `.agents/skills/*-search` over generic web search. Do not fabricate postings or score inaccessible postings from title alone.

## Vietnam portal orchestration

For Vietnam searches, use the orchestration CLI so CareerViet, Vieclam24h,
TopCV, ITviec, and VietnamWorks each receive an explicit status:

```bash
bun run .agents/skills/job-scrape/cli/src/cli.ts search \
  --query "Java backend ReactJS VueJS" \
  --location "Vietnam" \
  --max-experience 1 \
  --jobage 30 \
  --format table
```

- TopCV and ITviec are low-volume public discovery sources.
- CareerViet is manual-only under the current published access policy.
- Vieclam24h and VietnamWorks are restricted/manual-required under the current
  access review.
- Manual/restricted sources return official review links; never turn them into
  `no_matches`.
- Search cards are discovery leads. Only a permitted, active, sufficiently
  detailed posting may receive a fit recommendation.
- Never use credentials, private/mobile APIs, proxy rotation, CAPTCHA solving,
  or authenticated portal actions.
