# PostgreSQL reliability verification

Verified locally on 2026-11-05. No cloud deployment was performed.

## Finding and scope

The repository held a transaction connection while acquiring another connection
from the same pool. Feedback, scheduler claims, AI request admission, learning
controls and outbox preparation contained this pattern. Concurrent lock waiters
could occupy the remaining slots and prevent the transaction holding the lock
from progressing. A larger pool postpones this failure rather than fixing it.

The one-connection regression fails on baseline `1a21d30`: feedback times out
without committing a row. The patched version completes feedback, scheduler
claim and AI admission with one connection. These operations now reuse their
transaction connection. Digest preparation, enqueue and cursor advancement also
share a transaction. Business rules, provider behavior and idempotency keys are
unchanged. No migration is needed.

Another defect mapped every SQLx error to HTTP 503. Transport/acquisition failures
now produce sanitized 503 responses; schema/data/programming errors produce 500.
Constraint conflicts keep their existing handling.

This establishes an application defect, not attribution of every historical
Supabase failure. Old logs did not identify the failure stage. SQLx acquisition
timeouts include connection establishment and checking, so a timeout alone does
not prove pool exhaustion. No evidence establishes a TLS mismatch, Supabase
capacity limit or a pool per request. The backend already shared one pool.

## Pool and recovery policy

| Setting | Default | Environment override |
|---|---|---|
| Maximum connections | 3 | `RADAR_DATABASE_MAX_CONNECTIONS` (2–8) |
| Minimum connections | 0 | Fixed |
| Acquire timeout | 10 seconds | `RADAR_DATABASE_ACQUIRE_TIMEOUT_SECONDS` (1–120) |
| Idle timeout | 120 seconds | `RADAR_DATABASE_IDLE_TIMEOUT_SECONDS` (1–3600) |
| Maximum lifetime | 1800 seconds | `RADAR_DATABASE_MAX_LIFETIME_SECONDS` (30–86400) |
| Test before acquire | true | Fixed |

Empty/unset variables use defaults. Existing database URL selection and runtime
YAML/database precedence are preserved. Compose supplies the same defaults.
Credentials remain in environment variables; `.env.example` lists names only.
TLS options in the selected URL are preserved. Supabase shared session pooling
on port 5432 remains supported; there is no switch to transaction pooling.

Only startup's stateless connection handshake is retried: at most three attempts,
100/200 ms exponential delay plus bounded jitter, within one acquisition deadline.
Authentication and permanent TLS/configuration errors stop immediately. Queries,
writes and commits are never replayed. SQLx discards broken connections and
subsequent requests can acquire replacements. Worker cancellation drops pending
transactions and shutdown cleanup is bounded. SIGTERM and SIGINT are handled.

Diagnostics contain the SQLx variant, safe category, SQLSTATE when available,
operation, stage, elapsed time, pool size/idle count, correlation ID and retry
count. Acquisition helpers measure acquisition duration; generic error conversion
records operation elapsed time, not a fabricated acquisition duration. Requests
receive `x-request-id`; worker ticks receive a correlation ID. Logs exclude SQL,
parameters, connection URLs, credentials and raw server error messages.

Liveness remains independent of PostgreSQL. Readiness acquires a connection and
executes a bounded `SELECT 1`, returning 503 on genuine unavailability. A failed
readiness check does not terminate the process. Configure external orchestration
to avoid restarting the service solely because PostgreSQL is temporarily offline.

## Verification

- Formatting and Clippy with all targets/features: passed.
- Workspace suite with isolated PostgreSQL and ignored tests enabled: **104 passed,
  0 failed, 0 ignored**. Credential-free suite: **71 passed, 33 ignored**.
- Python unit tests (9), skill lint and security guards: passed.
- One-connection regression proves the original defect and patched behavior.
- TCP fault injection verifies offline startup, connection interruption, idle
  expiry, saturated pool/slow query, cancellation, concurrent workers/readiness,
  rollback of interrupted writes, restoration and bounded worker shutdown.
- Actual local PostgreSQL stop/start: readiness 200 → 503 → 200 in the same
  backend process; liveness stayed 200. Eight subsequent readiness requests passed.
  Six mock jobs and one feedback record survived; replaying the import key did
  not duplicate jobs. SIGTERM exited successfully.
- Existing fake-provider/outbox tests passed. Fault tests do not establish
  exactly-once delivery across an ambiguous external provider acknowledgment.
- Supabase read-only smoke: five low-rate SQLx `SELECT 1` probes and five actual
  HTTP readiness requests succeeded with a stable one-connection pool, no 503s.
  No migrations, initialization, workers, provider calls or data writes ran there.

Reproduce automated checks from the workspace root:

```sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings
cargo test --manifest-path rust/Cargo.toml --workspace
# Set this only to an isolated, disposable PostgreSQL instance:
RADAR_TEST_DATABASE_URL=postgres://radar_test@127.0.0.1:55432/postgres \
  cargo test --manifest-path rust/Cargo.toml --workspace -- --include-ignored
```

Read-only checks against the configured database, after exporting credentials:

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- db-check --reads 5
cargo run --manifest-path rust/Cargo.toml -p radar-server -- db-check --reads 5 --http
```

`db-check` accepts 1–10 reads, spaced two seconds apart. The HTTP variant starts
a temporary loopback server without workers or startup mutations. Never point
the integration/fault suite at production Supabase.

## Deployment decision and remaining limits

The code passes the database reliability gates above. **NO-GO for production
deployment yet**: the already-running user backend was not replaced, and a
short Supabase smoke test is not a long-running production soak. Restart the
local service with the patched binary and review correlated diagnostics during
normal use before approving cloud deployment. No deployment has been authorized.

Existing source operations may still hold a scan transaction during an external
provider call. That behavior was preserved to avoid changing Upwork execution
semantics; a slow provider can temporarily occupy a slot. Pool sizing must also
account for other applications sharing the database. No production pooler
settings were changed.

References: [Supabase connection methods](https://supabase.com/docs/guides/database/connecting-to-postgres)
and [SQLx 0.8.6 PoolOptions](https://docs.rs/sqlx/0.8.6/sqlx/pool/struct.PoolOptions.html).

## Changed-file inventory

- `.env.example`, `compose.radar.yaml`, `compose.radar.supabase.yaml`
- `rust/Cargo.toml`, `rust/apps/radar-server/Cargo.toml`
- `rust/apps/radar-server/src/{lib,main,workers}.rs`
- `rust/apps/radar-server/tests/{cli,cloud}.rs`
- `rust/crates/radar-adapters/src/{lib,database,postgres}.rs`
- `rust/crates/radar-adapters/src/postgres/{ai,interactions,learning,outbox,profiles,scheduler,upwork}.rs`
- `docs/radar/README.md`, `docs/radar/plan/{acceptance,cloud-sync,database-reliability}.md`

Changes to AI, learning, profiles and Upwork repository files concern connection
reuse and commit diagnostics only. No private settings or profiles were changed.
