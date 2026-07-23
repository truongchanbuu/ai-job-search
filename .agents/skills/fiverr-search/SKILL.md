---
name: fiverr-search
version: 1.0.0
description: >
  Generate Fiverr marketplace discovery links and manual lead records while
  emphasizing seller profile and gig readiness before client action.
context: fork
allowed-tools: Bash(bun run .agents/skills/fiverr-search/cli/src/cli.ts *)
---

# Fiverr Search Skill

Use this skill to create Fiverr discovery links and normalize manually reviewed
Fiverr leads. Fiverr is seller/gig oriented, so readiness checks should cover
seller profile completeness, gig/service categories, portfolio references, and
known setup blockers.

## Rules

- Do not store Fiverr passwords, payment credentials, private messages, or
  secret tokens.
- Treat generated links as discovery leads until the user verifies the
  marketplace page, buyer request, or user-supplied context.
- Recommend gig/profile setup before client action when readiness is unknown or
  incomplete.

## Commands

```bash
bun run .agents/skills/fiverr-search/cli/src/cli.ts queries -q "Flutter app" --service "mobile app development" --format table
bun run .agents/skills/fiverr-search/cli/src/cli.ts lead -q "Flutter app" --budget "$200" --format json
```
