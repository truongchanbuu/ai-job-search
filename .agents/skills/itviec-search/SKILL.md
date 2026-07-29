---
name: itviec-search
version: 1.0.0
description: Search low-volume public ITviec Vietnam technology jobs and verify public job details with evidence-backed fields and canonical attribution.
context: fork
allowed-tools: Bash(bun run .agents/skills/itviec-search/cli/src/cli.ts *)
---

# ITviec Search

Use this skill for Vietnam technology-job discovery on public ITviec pages.

```bash
bun run .agents/skills/itviec-search/cli/src/cli.ts search --query "Java" --location "Ho Chi Minh" --seniority fresher --limit 5 --format json
bun run .agents/skills/itviec-search/cli/src/cli.ts detail "<public-url>" --format json
```

Keep requests reasonable and low-volume. Do not sign in, retrieve login-only
salary/apply data, mirror pages, or continue through a challenge. Follow
`agent-guidance/workflows/job-scrape.md`.
