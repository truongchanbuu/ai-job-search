# Architecture — system boundaries

**Decision:** Rust monorepo as an extension of `ai-job-search`; no rewrite of legacy Claude/LaTeX components.

```text
Ratatui local client ────────┐
                              ├── HTTPS ──> Axum cloud API ──> PostgreSQL
Discord/other interaction ───┘                  │                 │
                                               ├── scheduler      │
                                               ├── ingest/normalize
                                               ├── rules/ranking  │
                                               ├── AI adapters    │
                                               └── outbox/router ─┴──> Discord/other bots

Authorized source (Upwork MCP after POC / manual / mock) --> JobSource port
Codex CLI or API (after POC) -----------------------------> AiProvider port
```

## Suggested Cargo workspace

```text
rust/
  Cargo.toml
  crates/
    radar-domain/           # pure models, events, scoring inputs, errors
    radar-application/      # use cases and ports, no HTTP or DB specifics
    radar-adapters/         # SQLx, job sources, bot providers, AI
  apps/
    radar-server/           # Axum API, cron, workers, webhooks
    radar-tui/              # Ratatui, Crossterm, authenticated API client
```

## Rules

The owner has also authorized a local HTML/CSS/JS client, served as embedded
assets by Axum. It uses the same authenticated application services as Ratatui;
see [web interface and browser security](../plan/web-ui.md). No separate JS
backend or cloud deployment is introduced.

- Domain does not import Axum, SQLx, Discord SDK or MCP internals.
- Worker and API can start in one server binary for MVP. Use a DB lock / unique run ID to avoid overlapping schedules.
- Bot endpoints enqueue work; they do not execute long AI tasks synchronously.
- TUI must never speak directly to PostgreSQL or access cloud secrets.
- Cloud remains functional when local TUI is closed; a failure of AI adapter does not prevent deterministic rules and manual review.
- Current authenticated runtime and provider availability must be shown clearly (e.g. `upwork: not configured`).

## Decisions pending validation

- Cloud deployment (VPS/container platform), TLS, secret handling, ongoing operating cost.
- Upwork unattended OAuth/MCP viability and allowed execution pattern.
- Codex CLI vs OpenAI API in unattended cloud and any associated usage/cost requirements.
