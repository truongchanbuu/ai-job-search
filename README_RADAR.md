# Personal Freelance Radar

A Rust extension of the existing AI Job Search repository for a personal,
evidence-based freelance shortlist. Existing Claude Code workflows, portal CLIs,
Python tools and LaTeX templates remain intact.

**Implemented phases:** 0–1 local mock/manual core; 2 authenticated Axum API,
PostgreSQL sync and a Ratatui terminal client; 3 persistent mock scheduling,
multi-destination notification outbox and signed Discord HTTP interactions.
Phase 4 adds a manual, read-only Upwork MCP adapter with Radar-owned OAuth and
live schema validation. Its manual path was verified; fresh consent/refresh and
unattended operation remain separate gates. Phase 5 adds manual, evidence-based AI
evaluation/proposals. Phase 6 adds owner-controlled preference models, shadow
comparison, calibrated diagnostics and rollback. See [Upwork setup](docs/radar/plan/upwork-mcp.md).

The working vertical slice is:

```text
Mock/manual jobs → Phase 1 rules and scorer → PostgreSQL → Axum API
                                                      ↕
                                                Ratatui client
                                                      ↓
                                            feedback → PostgreSQL
```

## Run

The original local, credential-free demo:

```sh
mkdir -p private/config
for namespace in radar scoring notifications; do
  cp -n "config/$namespace.example.yaml" "private/config/$namespace.yaml"
done
cargo run --manifest-path rust/Cargo.toml -p radar-server -- demo --at 2026-10-30T12:00:00Z
```

For the API and terminal app, follow the
[Phase 2 deployment and demonstration guide](docs/radar/plan/cloud-sync.md).
It includes private credential generation, Docker Compose/native development,
migrations, TUI controls and the real PostgreSQL test commands. Docker engine
execution was unavailable locally; native database/API/client execution is tested.

The terminal provides Dashboard, Job Explorer, Details, Evaluation Breakdown,
Configuration, Feedback History, Alerts and Learning. It uses authenticated HTTP and never reads
PostgreSQL directly. Changes are confirmed by the server and persist across
server restarts; configuration conflicts require explicit reconciliation.

## Source ownership

| Concern | Source |
|---|---|
| Product goals and scope | [BRD](docs/radar/BRD.md) |
| Individual behavior | [R01–R08](docs/radar/README.md) |
| Architecture | [System](docs/radar/architecture/01-system.md), [data](docs/radar/architecture/02-data.md), [integrations](docs/radar/architecture/03-integrations.md) |
| Configuration templates | `config/*.example.yaml` → runtime `private/config/*.yaml` |
| Approved runtime changes | Versioned PostgreSQL config namespaces; TUI/API |
| Verified facts and preferences | Private copies of `templates/radar-profile/*.md` |
| Credentials | Private environment/secret manager; never public Git |
| Follow-on work | [Phase 4–6 backlog](docs/radar/plan/backlog.md) |

The [local core guide](docs/radar/plan/local-core.md) documents private Markdown,
deterministic ranking, mock/manual imports and JSON exports. Public fixtures
contain synthetic data only; complete the private templates with verified facts
before using a real profile.

## Boundaries

No Upwork scraping, browser-session extraction, automatic applications or
marketplace writes. Upwork is disabled by default and available only through explicit manual scans
after separate authentication, account selection and schema binding. Scheduled
Upwork polling remains disabled pending an authorized technical/policy proof. Feedback `Applied` is an explicit report of a submission already
made by the owner; it never submits anything. A high score and a generated file
are not consent to apply.

The local Compose stack is not a finished production cloud deployment. Remote
hosting requires HTTPS ingress, protected credentials, database permissions,
backups and retention controls. See the [Phase 3 guide](docs/radar/plan/background.md)
and [Discord setup](docs/radar/plan/discord-setup.md) for scheduler controls,
verification evidence and external checks. Live Discord has not been verified.


Phase 5 provides manual AI evaluation, reviewed evidence-based proposals and
versioned edits in Job Details. AI is disabled by default. The offline mock
workflow is tested; the real provider is the tool-free Responses API and requires
separate explicit configuration. See the [Phase 5 guide](docs/radar/plan/phase5-ai.md).


Phase 6 preserves base scores and hard eligibility while adding bounded,
explainable display ranking from explicit feedback. Candidates require manual
activation; disable, correction, confirmed reset and rollback preserve history.
No AI or Upwork connection is required. See the [learning guide](docs/radar/plan/phase6-learning.md)
for configuration, API/TUI controls, model/backtest limits, synthetic A–F workflow
and the exact Phase 6 changed-file inventory.

**Profiles:** screen **9** manages private CV text, skills, preferences and
portfolio evidence with add/edit forms, sharing controls, preview and confirmed
save. The approved DB profile survives restart and is used by future cloud
scans/evaluations. See [setup and controls](docs/radar/plan/profiles.md).

**Proposal templates:** Job Details **T** selects a short problem-first brief; **G** generates from the selected job and verified shareable profile evidence. See [research and setup](docs/radar/plan/proposal-templates.md).
