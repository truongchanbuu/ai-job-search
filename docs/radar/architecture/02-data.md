# Architecture — persistence & data contracts

**Primary storage:** PostgreSQL (shared cloud state). **Editable source documents:** private Markdown. **Defaults:** YAML. All runtime changes are audited DB overrides.

## Tables (minimum viable)

| Table | Essential fields / constraints |
|---|---|
| `jobs` | `id`, `source`, `source_id`, `url`, `title`, `description`, `budget_* NULLABLE`, `posted_at NULLABLE`, `fetched_at`, unique `(source,source_id)` when available |
| `job_ingestions` | job, run, source payload hash, fetched at, outcome |
| `evaluations` | job, profile version, config version, breakdown JSON, score, confidence, verdict, created_at |
| `feedback` | job, user action, reason, timestamp, idempotency key |
| `applications` | job, current status, verified submitted_at, proposal version, Connects spent NULLABLE |
| `proposal_drafts` | job, content, screening answers, evidence refs, version, approval state |
| `config_overrides` | key/namespace, typed value, revision, actor, updated_at |
| `scan_runs` | run key unique, provider, scheduled time, state, error, counters, started/completed |
| `outbox_events` | unique event id, type, safe payload, created/processed time |
| `notification_deliveries` | event id + destination unique, attempt count, state, provider message id |
| `audit_log` | actor, action, entity, timestamp, redacted before/after |

## State model

- Job: discovered → normalized → evaluated, optionally shortlisted; `expired` is a distinct flag/status.
- Feedback: save/interested/skip/not-available is separate from *application lifecycle*.
- Application: none → drafted → submitted → interviewing → hired/rejected; completion may be tracked after hired.
- Never mark a proposal `submitted` because a file was generated or a draft was created.

## Data governance

- Public repo contains **only templates/synthetic fixtures**.
- `private/profiles/*.md` is personal, gitignored. Local-to-cloud sync requires an explicitly designed encrypted/authenticated transfer, not Git.
- Do not expose client message bodies, tokens or private profiles in Discord notifications by default.
- Schema migrations are additive/rollback-aware; a database restart preserves data.
- Include optional `retention_days` per data class; actual default values defined when production configuration is implemented.

## Implemented Phase 3 coordination

SQLx migrations 0003–0007 add scheduler/run coordination, mode request ledger,
notification outbox/delivery budgets/cooldowns, encrypted interaction receipts,
exact outgoing card snapshots and worker heartbeats. Jobs/evaluations/events
commit with scan completion. External delivery occurs after a committed claim;
ambiguous sending claims become uncertain rather than automatically duplicating
a remote POST. Interaction tokens use authenticated encryption with a private
environment key and are wiped when finished/expired. See
[background operation](../plan/background.md) for leases, catch-up, configuration
precedence and the verified migration/restart tests.


## Implemented Phase 6 learning traceability

Migration 0010 extends existing feedback/application enums and feedback attributes.
`feedback_learning_revisions` retains correction/exclusion/actual-effort history;
`preference_models` stores immutable structured fits, feature/config versions,
training cutoff/window, calibration and prefix backtests. `preference_state` holds
owner activation, suppression, control revision and feedback-correction epoch.
Model admission/rollback and corrections coordinate through a schema-scoped
PostgreSQL advisory lock and transactional revisions. Job scoring facts and
proposal/application history are preserved. See [model and controls](../plan/phase6-learning.md)
for precedence, invalidation, export, reset and historical limitations.
