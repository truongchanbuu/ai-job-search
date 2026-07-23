---
name: upwork-search
version: 1.0.0
description: >
  Generate Upwork freelance discovery links and manual lead records for
  account-sensitive opportunity review. Use with freelance-search readiness
  checks before proposal recommendations.
context: fork
allowed-tools: Bash(bun run .agents/skills/upwork-search/cli/src/cli.ts *)
---

# Upwork Search Skill

Use this skill to create targeted Upwork search links and normalize manually
reviewed Upwork leads. Upwork actionability depends on account readiness, such
as profile completeness, verification state, and Connects availability.

## Rules

- Do not store Upwork passwords, payment credentials, private messages, or
  secret tokens.
- Treat generated links as discovery leads until the user verifies the original
  posting or supplies copied opportunity text.
- Surface Connects and setup blockers before recommending proposal submission.

## Commands

```bash
bun run .agents/skills/upwork-search/cli/src/cli.ts queries -q "Flutter Firebase bug fix" -l "Remote" --format table
bun run .agents/skills/upwork-search/cli/src/cli.ts lead -q "Flutter Firebase bug fix" --budget "$300 fixed" --format json
```
