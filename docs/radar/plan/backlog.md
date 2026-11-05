# Delivery status and follow-on backlog

## Phase 2 — shared cloud state and terminal UI (implemented)

See [cloud sync](cloud-sync.md) for behavior, commands, test coverage and limits.
Implemented: authenticated Axum API; SQLx/PostgreSQL versioned migrations;
persisted jobs, scoring evidence, config revisions and feedback; approved
namespace overrides with concurrency checks; six Ratatui screens; real-server
restart, forged credentials, stale updates and offline-client tests.
Production hosting, TLS ingress, least-privilege DB deployment, retention and
backups still need environment-specific setup before using real personal data.

## Phase 3 — scheduler and Discord (implemented; external verification pending)

See [background operation](background.md) and [Discord setup](discord-setup.md).
Automated fake-provider/real-PostgreSQL acceptance passes. Live Discord and public
HTTPS ingress need manual verification. Operator resolution of uncertain sends,
history pagination and retention remain operational follow-ups.

## Phase 4 — manual integration implemented; live feasibility pending

See [Upwork MCP](upwork-mcp.md) for the manual adapter, OAuth, discovery/binding
and tested pipeline. Remaining checks: live Radar consent/refresh, actual search
schemas/results and permitted cloud usage. Periodic Upwork scans stay disabled.
No browser scraping, session extraction or marketplace-write adapter.

## Phase 5 — bounded proposal adapter

- Prove supported noninteractive Codex/API authentication and costs independently.
- Add bounded tasks, timeouts, usage auditing and restricted tools.
- Match only verified profile/portfolio evidence with truthful project context.
- Persist draft versions, review, screening answers and application state.
- Test malicious job text, invented claims and cost exhaustion; drafts never
  become submitted automatically. Marketplace writes remain out of scope.


## Phase 6 — manual adaptive ranking (implemented, deployment opt-in)

Structured reason/exclusion revisions, deterministic feature extraction/shrinkage,
bounded display ranking, immutable candidate models, chronological backtests,
shadow comparison, owner activation/rollback/reset and counted calibration are
implemented with API/TUI controls. See [operation and limits](phase6-learning.md)
and [verified acceptance](acceptance.md). Follow-ups: real-owner longitudinal
validation, decision-time impression logging, larger-history retention/export,
and optional richer feature/rubric review. None authorize automatic training,
rule changes or marketplace operations. Production migration/enablement remains
an explicit deployment step; no live Supabase tests are required.
