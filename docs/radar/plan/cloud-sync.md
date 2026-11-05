# Phase 2 — cloud sync and terminal UI

Current scheduler/Discord behavior is documented in [Phase 3](background.md).
This guide retains the Phase 2 setup and shared API contracts.

## Verified scope and Phase 0–1 audit

Before extending the application, the existing demo, formatting, Clippy and all
24 Rust tests were rerun successfully. The demo really imported seven records
into six canonical jobs. Nine Python tests, skill lint and security guards also
passed. The audit confirmed that PostgreSQL, HTTP and a functional TUI were not
yet implemented. Bun and LaTeX tooling remain unavailable locally; their source
and original workflows remain unchanged.

Phase 2 adds an Axum/Tokio API, a SQLx/PostgreSQL adapter and a Ratatui/Crossterm
client. The existing `radar-server demo` command and tests remain supported.
Config types/validation moved into radar-application for shared typed DTOs;
`radar-adapters::config` reexports the same interface for compatibility.
Imports stage the original Phase 1 `scan` in a database transaction with persisted
usage counters. There is one scorer, `radar-domain::evaluate`.

## Start with Docker Compose

Run from the repository root. Generate private environment values once:

```sh
mkdir -p private
umask 077
printf 'RADAR_DB_PASSWORD=%s\nRADAR_API_TOKEN=%s\n' \
  "$(openssl rand -hex 24)" "$(openssl rand -hex 32)" > private/radar.env

docker compose --env-file private/radar.env -f compose.radar.yaml up --build -d
docker compose --env-file private/radar.env -f compose.radar.yaml ps
curl --fail http://127.0.0.1:8080/health/ready
```

Use an unused filename if you already have private credentials; do not replace
credentials used by an existing database volume. `.env.example` lists supported
values with empty credential placeholders. The Compose-generated database URL
expects a hex password; URI-encode reserved characters if using your own URL.
The PostgreSQL service has no published port in the default configuration. Its
named volume persists across `docker compose down` and subsequent starts.
The PostgreSQL 18 volume is mounted at `/var/lib/postgresql`, following the
[official image's data-directory guidance](https://github.com/docker-library/docs/blob/master/postgres/README.md#pgdata).
The backend runs as a non-root user with a read-only filesystem.

The image copies committed templates to `/app/config/radar.yaml`,
`scoring.yaml`, and `notifications.yaml` at build time. Native execution defaults
to repository `private/config`; prepare those same plain filenames using the
[configuration setup](../../../config/README.md). `--config-dir` overrides
`RADAR_CONFIG_DIR`, which overrides the native default. Runtime loading never
falls back to `.example.yaml` files.

The bundled server profile is explicitly synthetic. To use private settings or
profiles, mount private directories read-only at `/app/config` and
`/app/demo-profiles`, or set `RADAR_CONFIG_DIR` / `RADAR_PROFILE_DIR` to your mounted
paths. Never bake completed profiles into a public container image.

Start the terminal in a second shell:

```sh
set -a
. private/radar.env
set +a
export RADAR_API_URL=http://127.0.0.1:8080
cargo run --manifest-path rust/Cargo.toml -p radar-tui -- --allow-local-http
```

`--allow-local-http` works only with loopback hosts. A remote API URL must use
HTTPS. `--check` performs an authenticated read of all main resources without
entering raw terminal mode.

## Native Rust development

Use an existing local PostgreSQL database, or publish the development database
on loopback explicitly:

```sh
docker compose --env-file private/radar.env \
  -f compose.radar.yaml -f compose.radar.dev.yaml up -d db

set -a
. private/radar.env
set +a
export RADAR_DATABASE_URL="postgres://radar:${RADAR_DB_PASSWORD}@127.0.0.1:5432/radar"
export RADAR_API_URL=http://127.0.0.1:8080

cargo run --manifest-path rust/Cargo.toml -p radar-server -- migrate
cargo run --manifest-path rust/Cargo.toml -p radar-server -- serve --bind 127.0.0.1:8080
```

The server reads `RADAR_DATABASE_URL` by default, or the environment reference
selected in `radar.storage.database_url_env`. YAML `${NAME}` references are not
expanded into logs or API config responses. `RADAR_API_TOKEN` is mandatory and
must contain at least 32 non-whitespace ASCII characters. Optional
`RADAR_READ_ONLY_TOKEN` grants reads but rejects mutations. Empty means unset.
Tokens are supplied through the environment, never command-line arguments.

`serve` also runs embedded migrations before accepting requests. Migrations live
under `rust/migrations/`, with SQLx checksums and version history in
`_sqlx_migrations`. Do not edit an applied migration; add a new version. Repeated
startup and a version-1-to-version-2 data-preserving upgrade are tested. Back up a real database before a schema upgrade; this phase
provides forward migrations, not automated downgrade scripts.

## Interactive demonstration

1. Start the backend and TUI. Dashboard initially shows persisted counts or zero.
2. Press `m` to queue a mock scan. Wait for a completed run on Dashboard;
   jobs appear subject to the active per-run/day quotas.
3. Press `2`, use arrows, and press Enter. `3` shows details; `4` shows the
   persisted score breakdown, evidence, unknowns and snapshot revisions.
4. Press `s` to choose Save, type an optional reason, and press Ctrl+S. Wait for
   `Server saved…`; `6` shows the feedback row read back from PostgreSQL.
5. Press `5`, select Work mode, Enter, replace the value with `busy`, Enter,
   then `s` to save. The server validates the config and increments its revision.
   The UI refreshes the returned state; public YAML remains unchanged.
6. Restart the backend. Press `r` in the TUI. Jobs, decisions, feedback, config
   overrides and evaluation history remain. The same import key can be retried
   without producing another ingestion; new imports preserve fetch history.
7. Stop the backend and press `r`. The UI marks its snapshot offline/stale and
   retains cached rows. No mutation is reported saved without an acknowledgment.

Keyboard reference:

| Key | Action |
|---|---|
| `1`–`7`, Tab | Dashboard, Jobs, Details, Evaluation, Config, Feedback, Alerts |
| Up/Down, `j`/`k` | Select rows/fields; scroll details |
| Page Up/Down | Scroll long details and evidence |
| `/` in Jobs | Search title/description; Enter applies, Esc cancels |
| `v`, `g`, `o` in Jobs | Cycle verdict, decision filter and sort |
| `[`, `]` | Jobs/feedback pagination |
| Enter in Jobs | Open selected details |
| `f`, `i`, `s`, `x`, `n` on a job | Feedback form / Interested / Save / Skip / Not available |
| Up/Down in feedback form | Choose preference or application-outcome action |
| Ctrl+S in feedback form | Send action/reason with job revision and idempotency key |
| Ctrl+A in feedback form | Explicitly confirm an already-made submission for Applied |
| Enter, `s`, `d` in Config | Edit field, save draft, discard/reload fetched server config |
| `m`, `p` | Queue mock scan; pause/resume current mode |
| `r`, `t` | Refresh; retry the same failed operation |
| `q`, Ctrl+C | Quit (Esc closes an input form; Ctrl+C also exits forms) |

Config editing covers mode, timezone, current-mode hours and quotas, thresholds
and all seven weights. Invalid drafts remain editable after rejection; weights
must sum correctly before saving. A stale revision returns HTTP 409. Refresh
fetches current state while preserving a dirty config draft; `d` explicitly
replaces it with the fetched server version. Reapply desired edits and save.
There is no automatic overwrite/merge of conflicting edits.

Refreshes run at the configured interval and after successful mutations. Network
requests run separately from terminal input/rendering, with bounded timeouts.
Resizing preserves state, and small terminals show a resize prompt. No meaning
is communicated by color alone. Source text is displayed as text with terminal
control characters removed, never interpreted as instructions.

## API contracts

All `/v1/*` resources require `Authorization: Bearer <token>`. Owner credentials
can mutate; read-only credentials receive 403 for writes. No browser cookies or
CORS credentials are used. Responses are not cacheable. Both clients use shared
Serde DTOs in `radar-application::api`.

| Method/path | Contract |
|---|---|
| GET `/health/live`, `/health/ready` | Public status only; readiness checks PostgreSQL |
| GET `/v1/dashboard`, `/v1/sources` | Counts, mode/revision, last import and source availability |
| GET `/v1/jobs` | `search`, `verdict`, `decision`, `sort`, `page`, `page_size` (1–100) |
| GET `/v1/jobs/{uuid}` | Full job, latest evaluation, decision/revision, application foundation |
| GET `/v1/jobs/{uuid}/evaluations` | Last 100 immutable evaluations with input/config/profile references |
| POST `/v1/imports` | UUID idempotency key, `source: mock/manual`, optional manual `jobs` |
| GET/PUT `/v1/config` | Effective config; update selected typed namespaces with `expected_revision` |
| GET `/v1/config/history` | Last 100 effective config revisions and actors |
| GET `/v1/feedback` | Global or `job_id` history; paged |
| POST `/v1/jobs/{uuid}/feedback` | UUID key, `expected_revision`, action, optional reason/confirmation |

Errors use `{ "error": { "code": "...", "message": "..." } }`. Validation,
authentication, authorization, stale state, missing records and DB availability
use 400, 401, 403, 409, 404 and 503 respectively. Invalid JSON/query/path inputs
use the same envelope and do not echo submitted values. No token or database
connection URL is logged. A failed/timeout mutation is not assumed committed;
retrying the same import/feedback key resolves uncertainty. Config retries may
return 409 after an acknowledged-lost write; refresh before deciding another edit.

Example request bodies (supply fresh UUIDs and fetched revisions):

```json
{"idempotency_key":"11111111-1111-4111-8111-111111111111","source":"mock","jobs":null}
```

```json
{"idempotency_key":"22222222-2222-4222-8222-222222222222","expected_revision":0,"action":"save","reason":"review after work","confirmed_submission":false,"submitted_at":null}
```

## Persistence, source ownership and application state

Imports, jobs, sources, ingestion hashes/history, evaluations, evidence and
profile/config snapshots are persisted. A canonical job uses Phase 1's provider
ID/normalized URL identity, hashed for a bounded database key; uniqueness is
also enforced for provider IDs. Idempotency keys apply to entire import/feedback
requests. Reusing a key with a different request is a conflict. Persisted local-day
usage is shared across restarts and imported jobs are bounded by effective caps.

Approved namespace overrides take precedence over deployed YAML. Startup
reconciles changed deployment defaults underneath approved namespaces, recording
a new revision. Namespace overrides apply to the whole namespace; changing an
unapproved field within that namespace in YAML does not override an approved DB
snapshot. Deployment files are never edited by the API or TUI. Historical scores
remain historical; a later import evaluates using the then-effective policy.

Preference decisions are distinct from application lifecycle. Applied requires
an owner's explicit confirmation that they already submitted elsewhere.
Interview/Hired/Rejected/Completed require valid prior state. No generated file,
high score or Save action implies submission. The report time is stored separately. Actual submission time remains unknown
unless the owner supplies `submitted_at` in the Applied request; future times
are rejected. The TUI reports submission without inventing its date. Connects
remains unknown. Skipped/unavailable and applied
jobs are excluded from new recommendations on subsequent imports. Feedback never
rewrites profile facts or scoring weights.

## Tests and operational limits

From `rust/`:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
RADAR_TEST_DATABASE_URL=postgres://USER:PASSWORD@127.0.0.1:5432/radar_test \
  cargo test --workspace -- --include-ignored
```

Database tests are explicitly ignored in the default credential-free run and
**must** be included to verify Phase 2. They create/drop uniquely named schemas
only in the supplied test database. That database user needs CREATE SCHEMA.
CI provisions disposable PostgreSQL and includes these tests. The real-server
restart test starts/kills two server processes; it does not simulate restart by
merely clearing a Rust collection. Rendering and interaction tests use Ratatui's
TestBackend with real API data; they cover all six screens and terminal sizes.

Local validation uses real native PostgreSQL because the Docker engine is
unavailable in this environment. Compose configuration is validated; a container
image build/start is still an environment-specific deployment check.

This is a personal application with owner/read-only credentials, not a multi-user
tenancy system. The Compose stack binds its API to loopback and is a local
configuration. Remote deployment needs HTTPS ingress, firewalling, proper secret
rotation, backups, a least-privilege runtime DB role and a retention/deletion
policy. Non-loopback API binding requires an HTTPS `RADAR_PUBLIC_URL` declaration
or explicit local-development opt-in. The application does not terminate TLS or
verify a reverse proxy's deployment; enforce HTTPS at ingress and keep direct
backend access private. An HTTPS URL declaration alone does not provide TLS.

Profile transfer/editing, rollback UI, source pagination, automated retention and
lifecycle notes/Connects editing remain future work. There are no marketplace-write
endpoints. Upwork and Codex integrations remain disabled/gated.

## Phase 3 extension

Persistent scheduling, outbox/retries, signed Discord interactions and additional
TUI controls are implemented and tested; see [background operation](background.md)
for migration versions, configuration precedence, tests and known limitations.
See [Discord setup](discord-setup.md) for the unperformed real-service checks.


Native server database connection acquisition defaults to 30 seconds for remote
PostgreSQL/Supabase. Set `RADAR_DATABASE_ACQUIRE_TIMEOUT_SECONDS` to an integer
from 1 through 120 to override it (leave the variable empty or unset to use the default).
This setting applies to `serve` and `migrate`. Startup errors distinguish
connection timeouts, TLS failures and database rejections without printing the
connection URL or credentials. API database errors log only their SQLSTATE code.

The terminal client's total HTTP request timeout defaults to 45 seconds, allowing
for remote database acquisition and query time. Override it with
`RADAR_HTTP_TIMEOUT_SECONDS` (1–120); leave empty or unset for the default. A client timeout
still reports an unknown mutation outcome, never a success. Refresh or retry
using the same idempotency key before submitting another mutation.
