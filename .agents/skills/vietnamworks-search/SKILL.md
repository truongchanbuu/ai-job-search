---
name: vietnamworks-search
version: 1.0.0
description: Report VietnamWorks coverage and generate official manual review links for Vietnam job searches when protected public access prevents verified automation.
context: fork
allowed-tools: Bash(bun run .agents/skills/vietnamworks-search/cli/src/cli.ts *)
---

# VietnamWorks Search

Use this source when VietnamWorks coverage is requested. Current mode is
restricted/manual-required because ordinary unauthenticated requests returned
403 at the latest review.

```bash
bun run .agents/skills/vietnamworks-search/cli/src/cli.ts search --query "Java fresher" --format json
```

Never infer private identifiers, call mobile/private APIs, reuse credentials,
or bypass a challenge. Follow `agent-guidance/workflows/job-scrape.md`.
