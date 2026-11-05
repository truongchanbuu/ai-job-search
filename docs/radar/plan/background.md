# Phase 3 — background scans and durable notifications

Phase 2 was reverified before extension: its existing 36 Rust tests, including
10 real PostgreSQL integration tests, passed. The Phase 1 scorer and import /
feedback services remain the shared implementation. No live marketplace or AI
adapter is enabled.

## Run and observe

Follow [cloud-sync](cloud-sync.md) for credentials, PostgreSQL, migrations and
Compose/native startup. `serve` now starts Tokio workers automatically;
`--no-workers` is an explicit API-development option. Use the current three YAML
examples when making a private config directory, renaming them to `radar.yaml`,
`scoring.yaml`, and `notifications.yaml` (see [setup](../../../config/README.md)).
No Discord token is needed to scan mock jobs; delivery history reports missing credentials without reporting
messages as sent. [Discord setup](discord-setup.md) enables real delivery.

```sh
# From the repository root, after loading private environment variables:
cargo run --manifest-path rust/Cargo.toml -p radar-server -- migrate
cargo run --manifest-path rust/Cargo.toml -p radar-server -- serve --bind 127.0.0.1:8080
# In a second shell with RADAR_API_TOKEN and RADAR_API_URL:
cargo run --manifest-path rust/Cargo.toml -p radar-tui -- --allow-local-http --check
cargo run --manifest-path rust/Cargo.toml -p radar-tui -- --allow-local-http
```

Existing screens and navigation remain. Dashboard adds worker heartbeat, next
scheduled scan, last execution and recent runs. `m` queues a mock scan; `p`
pauses/resumes using a revision-checked server mutation. `7` opens Alerts with
provider diagnostics, delivery status, attempts and safe errors. Auto-refresh
uses `scheduler.ui_refresh_seconds`; a dirty configuration draft is preserved.
Discord feedback appears in job decisions and Feedback History on refresh.

All following routes require the existing owner/read-only bearer authentication;
read-only credentials cannot POST. Mutations return persisted acknowledgments;
queued work is not reported completed merely because it was accepted.

| Route | Contract |
|---|---|
| GET `/v1/scheduler` | Mode, enabled state, next slot, last completed/failed run, recent 50 runs, active worker heartbeat |
| POST `/v1/scheduler/scan` | `{ "idempotency_key": "UUID" }`; 202 queues a bounded manual mock run |
| POST `/v1/scheduler/mode` | `{ "idempotency_key": "UUID", "expected_revision": 0, "paused": true }` |
| GET `/v1/notifications` | Latest 100 deliveries and provider configuration diagnostics |
| POST `/v1/notifications/digest` | `{ "idempotency_key": "UUID" }`; queues a digest event |

## Scheduling and recovery

`app.timezone` defaults to Asia/Ho_Chi_Minh in YAML. Existing work modes own
weekdays, scan times and daily job/request quotas. `scheduler` owns enabled,
polling, per-run cap, pending-run admission limit, recovery lease, catch-up window,
UI refresh and fallback resume mode. Employed/available/busy have separate
policies. Paused blocks new scans and ordinary pushes; resume restores the
previous mode. Disabling scheduled planning still permits explicit manual scans.

A persistent planning cursor considers the bounded catch-up window and coalesces
missed slots to the latest eligible slot. It never replays an unlimited backlog.
Local days determine quota/digest boundaries. DST gaps are skipped; an ambiguous
local scan time selects its earliest occurrence once. Slot keys are stable UTC
instants. Poll latency can delay execution; this is not a hard real-time timer.

PostgreSQL advisory locks, a single-running partial unique index, leases and
owner fencing coordinate workers. Claims are committed before execution. Job
imports, scoring, quota usage, notification events and completion commit together.
An expired interrupted claim is recovered when the coordination lock is free.
Completed run keys and canonical jobs are not repeated after restart. Failed
source execution records a failed run and a failure-alert event. Heartbeats
expire; readiness means DB connectivity, while Dashboard exposes worker liveness.

## Outbox and provider boundaries

`NotificationProvider` and `InteractionProvider` are application ports. The
registry builds named Discord instances; tests inject providers through the same
port. `NotificationRouter` chooses configured destinations by event, minimum
score and confidence. No domain service contains Discord HTTP behavior.

Events are stored before HTTP. Each event/destination has one durable delivery.
A job-match event is created once per newly qualifying canonical job. Routes are
snapshotted when the event is created; new destinations apply to future events.
A delivery snapshots the current card and job revision before sending. Stale
scoring config requires rescan before an old job alert can be delivered.
Digests use stable local-date keys; explicit digest requests use their request
keys. Scans reuse the existing deterministic scorer and MockJobSource.

Ordinary alerts respect quiet hours, current routes, mode, thresholds and a shared daily cap counting each destination delivery reservation. Safe retries reuse their same-day reservation;
a retry on a new day reserves that day's quota. Critical scan failures bypass
quiet hours, pause and ordinary caps, but respect the global notification toggle.
Deferred command responses have their own token expiry rather than proactive
alert quotas. Quiet start equal to end disables the quiet interval.

Delivery leases coordinate concurrent workers. Retryable failures use bounded
exponential backoff and honor larger provider retry-after values and global /
channel cooldowns. Ambiguous channel POST failures or expired sending claims
become **uncertain** and are not blindly resent. This preserves restart safety
but may leave an undelivered message requiring operator review. External HTTP
cannot provide an atomic transaction with PostgreSQL. Discord's short-lived nonce
deduplication is additional protection, not a permanent exactly-once guarantee.
Successful delivery requires a provider acknowledgment; history distinguishes
pending, retry, sending, sent, failed, cancelled and uncertain states.

## Schema and configuration

Existing migrations 0001/0002 are unchanged. Additive SQLx migrations:

| Version | Tables/change |
|---|---|
| 0003 | Scheduler cursor, scan runs, mode request ledger |
| 0004 | Outbox events, deliveries, budgets, provider cooldowns, digest cursor |
| 0005 | Discord interaction receipts, actions/replies, encrypted token and leases |
| 0006 | Exact outgoing delivery card snapshot |
| 0007 | Worker heartbeats |

Precedence is approved PostgreSQL namespace > deployed YAML. Newly introduced
optional scheduler/runtime sections missing in an older approved namespace are
backfilled from deployed YAML during startup reconciliation. All other approved
namespace values remain authoritative. No API writes deployment files.
Validated config revisions are read each worker tick; mode, schedules, routing,
quiet hours, caps and toggles can change without rebuilding. Secret *references*
are editable config; secret values come from the server environment/secret source.
Environment rotation requires restarting the process. Command registration reads
deployment YAML explicitly; use a reconciled private copy if runtime command
configuration differs. No numeric deployment policy is hidden in TUI controls.

## Verified tests and limits

From `rust/`, the following passed locally with native PostgreSQL 18:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RADAR_TEST_DATABASE_URL=postgres://USER:PASSWORD@127.0.0.1:5432/radar_test \
  cargo test --workspace -- --include-ignored
```

Default: 28 passed, 18 database tests ignored. With the isolated database:
46 passed, none ignored. `apps/radar-server/tests/cloud.rs` covers concurrency,
leases/restart, quotas, modes, missed slots, live configuration, routing, quiet
hours, rate limits, ambiguous delivery, signed commands/buttons/modals, replay,
expiry, authorization and feedback-to-TUI persistence. A local HTTP Discord stub
checks the real transport's request bodies, rate-limit parsing, reply editing and
command registration. TUI tests render all seven screens at four terminal sizes.
The end-to-end workflow uses a fake Discord provider and real PostgreSQL/HTTP.

A separate native smoke check ran an actual configured HCMC scan slot: six
canonical jobs, a completed scheduled run and three pending outbox deliveries.
The real terminal Alerts view, pause/resume and manual queue controls were
exercised. A second run completed without new canonical jobs/events. After
restarting the native server, six jobs, two completed runs, revision 3 and three
delivery records remained. With no bot credentials, no delivery was marked sent.
Nine Python tests, skill lint, security guards and both base/merged Compose
configuration validation passed. Existing Claude workflows were preserved.

Live Discord delivery/registration and public ingress have **not** been verified.
No credentials were available. Docker engine execution is unavailable locally;
Compose configuration is validated separately from native application tests.

There is no uncertain-delivery resolution UI, outbox/history pagination, automatic
retention, distributed secret rotation or production hosting setup. Configuration
edits cover common operational fields; full provider/route edits use typed API
namespace updates or private deployment YAML. Source scans remain mock-only.
Phase 4 can build on durable coordination and provider boundaries, but requires
an authorized Upwork authentication/terms feasibility proof first. Upwork and
Codex evaluation remain disabled; there are no marketplace-write endpoints.
