---
name: careerviet-search
version: 1.0.0
description: Generate official CareerViet browser search links and normalize user-supplied CareerViet references for Vietnam job discovery without unattended crawling.
context: fork
allowed-tools: Bash(bun run .agents/skills/careerviet-search/cli/src/cli.ts *)
---

# CareerViet Search

Use this source for Vietnam job discovery when CareerViet coverage is requested.
Current policy is manual-only: generate the official search link, ask the user
to review it in an ordinary browser, and accept a public reference they supply.

```bash
bun run .agents/skills/careerviet-search/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --format json
```

Do not crawl search pages, bypass controls, store credentials, or claim that a
manual source returned zero matches. Follow
`agent-guidance/workflows/job-scrape.md`.
