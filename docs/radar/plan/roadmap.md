# Roadmap — small, verifiable delivery slices

Every phase must compile and demonstrate a real vertical slice; do not generate empty architecture stubs.

| Phase | Deliverable | Exit criteria |
|---|---|---|
| **0 — Repo audit** | Inspect upstream fork, document conflicts, create Cargo workspace + `AGENTS.md` without breaking Claude tools | Existing tests still pass or existing failure documented |
| **1 — Local mock core** | Rust domain, YAML config, profiles loader, deterministic scorer, mock source, in-memory/repository abstraction, CLI smoke | `cargo test` passes; prints ranked synthetic jobs |
| **2 — Cloud sync + TUI** | Axum REST, SQLx/Postgres, migrations, Ratatui jobs+config+feedback, authenticated API | UI actions round-trip and survive server restart |
| **3 — Scheduler + Discord** | Rust scheduler, run locks, outbox, Discord HTTP bot, verified commands/buttons, routing | Scheduled mock job yields one notification; feedback syncs to TUI |
| **4 — Upwork feasibility** | Headless MCP/terms POC; only integrate permitted actions and auth flow | Signed-off evidence or explicit disabled-provider status |
| **5 — Codex proposal flow** | Bounded AI adapter, profile evidence, draft review, proposal tracking | Verified truthful draft; no marketplace write |
| **6 — Personalization** | Audited feedback-driven suggestions and rollback | User can approve/reject weight change with explanation |

## Definition of done per phase

- Feature implemented and exercised; not just a trait or placeholder.
- Automated tests include happy path, provider failure, auth and idempotency where applicable.
- Sensitive data not committed; no breaking unrelated `.claude` features.
- README/config/docs updated only in the owning files.
- Known limitations stated explicitly.

The initial implementation was scoped to Phase 0 and Phase 1; later authorized
phases extend those foundations without enabling Phase 4/5 adapters.

## Phase 3 verified implementation

Scheduler, PostgreSQL coordination/recovery, multi-instance Discord outbox,
signed HTTP commands/buttons/modals and existing TUI synchronization are
implemented. Local validation passed 46 Rust tests, including 18 opt-in real
PostgreSQL tests. The full mock → scorer → PostgreSQL → fake Discord → signed
feedback → PostgreSQL → TUI workflow passes. Existing Phase 2 behavior was
reverified before extending it.

[Operation and limits](background.md) · [Manual Discord verification](discord-setup.md).
Real Discord delivery and public ingress remain unverified; automated success
does not establish that external deployment check. Phase 4 remains gated on
authorized Upwork feasibility; no Upwork adapter was enabled.

## Phase 4 manual integration and remaining gate

A read-only official MCP adapter, Radar-owned interactive OAuth/refresh storage,
account/tool discovery, reviewed live-schema binding, local search filters,
manual API/TUI queueing and shared import/scoring/outbox persistence are
implemented. Deterministic tests use synthetic MCP/OAuth servers and isolated
PostgreSQL. See [Phase 4 operation](upwork-mcp.md).

**Manual path verified (2026-11-04):** Radar-owned stored credentials,
live account/tool/help discovery, schema binding, a bounded read-only search,
Supabase ingestion/scoring and feedback, and terminal-client reads after backend
restart succeeded. The one job scored `skip` using the bundled synthetic profile;
no notification was warranted. See [current acceptance](acceptance.md).

Fresh consent UI, real refresh and Discord delivery/interactions remain external
checks. This is not approval for unattended use. Scheduled Upwork polling stays
disabled in execution and database constraints. Phase 5 can build on the verified
manual source; proposal claims still require a completed verified personal
profile. Phase 5 manual AI evaluation/proposal work is described below.


## Phase 5 implemented and offline verified

Manual AI evaluation, curated evidence snapshots, review/approval, grounded draft
assembly, screening answers, immutable edits/regeneration, quotas/cancellation,
PostgreSQL coordination and Job Details controls are implemented. AI is disabled
by default. Mock and tool-free Responses API adapters extend the existing port;
Codex CLI is not registered without a proven tool/filesystem isolation boundary.

All 79 tests, including 23 isolated PostgreSQL tests, passed. The standalone
synthetic HTTP workflow and actual backend restart were demonstrated. Production
Supabase was not used for tests or migrated. No real AI inference was performed;
that remains an explicit credentialed opt-in check. See
[operation, privacy and limitations](phase5-ai.md) and [acceptance](acceptance.md).

## Phase 6 implemented and synthetic acceptance verified

Explicit structured feedback now drives immutable, regularized preference models
and bounded display ranking above unchanged deterministic eligibility. Manual
rebuild, shadow comparison, chronological holdout, owner activation/rollback,
suppression/rejection, correction invalidation and confirmed reset are exposed
through the existing authenticated API and Ratatui Learning view. Phase 5 AI
assessments and actual reported effort/outcomes feed counted calibration only.

Verification: 63 credential-free tests passed with 28 database tests skipped;
all 91 tests passed against isolated local PostgreSQL, including actual process
restart with activated weights/order preserved. Fmt, all-target/all-feature
Clippy, nine Python tests and repository guards passed. Synthetic A–F acceptance
covers neutral cold start, bounded positive/negative ranking, hard veto, exact
rollback and disabled baseline behavior. See [learning guide](phase6-learning.md)
and [acceptance](acceptance.md).

Personalization stays disabled by default. Migration 0010 and enablement were
not applied to production Supabase. Longitudinal owner validation and deployment
remain opt-in follow-ups. No external AI call, Upwork polling, notification-rule
change, automatic prompt adjustment or marketplace write was introduced.
