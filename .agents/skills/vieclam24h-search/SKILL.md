---
name: vieclam24h-search
version: 1.0.0
description: Report Vieclam24h coverage and generate official manual review links for Vietnam job searches when protected public access prevents verified automation.
context: fork
allowed-tools: Bash(bun run .agents/skills/vieclam24h-search/cli/src/cli.ts *)
---

# Vieclam24h Search

Use this source when Vieclam24h coverage is requested. Current mode is
restricted/manual-required because ordinary unauthenticated requests returned
403 at the latest review.

```bash
bun run .agents/skills/vieclam24h-search/cli/src/cli.ts search --query "Java fresher" --format json
```

Never reuse tokens, reverse-engineer mobile/private APIs, retry around
CAPTCHA/403, or claim full coverage. Follow
`agent-guidance/workflows/job-scrape.md`.
