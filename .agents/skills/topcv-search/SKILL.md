---
name: topcv-search
version: 1.0.0
description: Search low-volume public TopCV Vietnam job listings and verify a public posting while preserving canonical links and avoiding authenticated actions.
context: fork
allowed-tools: Bash(bun run .agents/skills/topcv-search/cli/src/cli.ts *)
---

# TopCV Search

Use this skill for TopCV Vietnam job discovery and permitted public detail
verification.

```bash
bun run .agents/skills/topcv-search/cli/src/cli.ts search --query "Java fresher" --max-experience 1 --limit 5 --format json
bun run .agents/skills/topcv-search/cli/src/cli.ts detail "<public-url>" --format json
```

Keep requests low-volume and link-first. Store normalized decision facts, not
mirrored pages. Stop on login, CAPTCHA, 401/403, or unexpected challenge pages.
Do not use authenticated actions. Follow `agent-guidance/workflows/job-scrape.md`.
