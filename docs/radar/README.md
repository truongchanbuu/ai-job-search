# Documentation map

Read the [BRD](BRD.md), then change only the smallest file that owns your concern. This directory is the **product and design contract**; live values reside in YAML / server state, not this BRD.

## Requirements (one topic per file)

| Topic | File |
|---|---|
| Job sources & discovery | [R01](requirements/R01-job-discovery.md) |
| Matching & risk evaluation | [R02](requirements/R02-job-evaluation.md) |
| Modes, quotas & schedule | [R03](requirements/R03-work-modes.md) |
| Profiles, portfolio & learning | [R04](requirements/R04-personalization.md) |
| Proposal preparation / approval | [R05](requirements/R05-proposals.md) |
| Discord-first multiple bots | [R06](requirements/R06-multi-bot.md) |
| Terminal app & cloud sync | [R07](requirements/R07-tui-and-sync.md) |
| Security & compliance | [R08](requirements/R08-security.md) |

## Architecture & quality

- [Simple local web interface](plan/web-ui.md) — HTML/CSS/JS on the existing authenticated Axum API.

- [System boundaries](architecture/01-system.md) · [Storage model](architecture/02-data.md) · [Integration contracts](architecture/03-integrations.md)
- [Scoring standard](standards/scoring.md) · [Proposal standard](standards/proposals.md) · [Editing conventions](standards/documentation.md)
- [Phases](plan/roadmap.md) · [Acceptance tests](plan/acceptance.md) · [Official references](references.md)

- [Phase 4 manual Upwork MCP](plan/upwork-mcp.md) — live authentication/search verification pending
- [Phase 3 scheduling and outbox](plan/background.md) · [Discord setup](plan/discord-setup.md)
- [Phase 2 cloud sync and terminal UI](plan/cloud-sync.md) · [Database reliability](plan/database-reliability.md)
- [Phase 0–1 local core](plan/local-core.md) · [Follow-on backlog](plan/backlog.md)

## Personal files and configuration

- [Open Radar from the Windows desktop](plan/desktop-launcher.md) — one shortcut for the local backend and terminal UI.

- **Human prose:** `templates/radar-profile/*.md` → copy to `private/profiles/*.md` (**gitignored**).
- **Operational settings:** `config/radar.example.yaml`, `config/scoring.example.yaml`, `config/notifications.example.yaml` → copy as `private/config/radar.yaml`, `scoring.yaml`, and `notifications.yaml` (**gitignored**) for local overrides; see [setup](../../config/README.md).
- **Secrets:** environment / secret manager only. Never put tokens in MD/YAML or public Git.

## Editing discipline

1. If changing a rule, edit the owning `Rxx` file and update the linked acceptance case.
2. If changing a value (time, max jobs, threshold, channel), edit YAML or TUI settings **without rewriting the BRD**.
3. If changing personal preferences, edit the private profile Markdown; facts must be human verified.
4. If changing a long-term technical decision, document it in the relevant architecture file.
5. Avoid duplicating defaults between BRD, YAML, code, and TUI: code and UI read one normalized *effective configuration*.


Phase 5: [manual AI evaluation and evidence-based proposals](plan/phase5-ai.md)
includes provider selection, private evidence setup, API/TUI controls, cost/privacy,
synthetic demonstration, opt-in live test and the exact changed-file inventory.


Phase 6: [adaptive ranking, preference learning and calibration](plan/phase6-learning.md)
includes the deterministic model, structured feedback, shadow/backtest controls,
manual activation/rollback, privacy, limits and synthetic demonstration.

Profiles: [CV, skills and portfolio setup in the terminal](plan/profiles.md)
adds labelled forms, evidence preview, explicit confirmation and immutable private
profile revisions used by the existing scoring/AI pipeline.

Proposal templates: [research, minimal patterns and profile-grounded generation](plan/proposal-templates.md).
