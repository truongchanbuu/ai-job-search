# Phase 4 — read-only Upwork MCP

Radar uses the official `https://mcp.upwork.com/mcp` endpoint through the Rust
MCP SDK (`rmcp` 3.8, Streamable HTTP). There is no scraping, private API,
browser-cookie import, Codex token reuse or marketplace-write adapter.

**Live status (2026-11-04):** Radar-owned stored credentials authenticated,
account/tool/help discovery succeeded, and both a one-result schema probe and the
actual adapter's read-only live smoke test passed. Real consent UI and token
refresh were not exercised in this verification. One live job and Save feedback persisted in Supabase and remained visible to
the terminal client after backend restart. Real Discord delivery remains unverified. Synthetic
schemas in ordinary CI remain test fixtures, not claims about the live API.

Tool discovery accepts plain names such as `list_accounts` and the Upwork
namespace form `upwork__list_accounts`. Calls use the exact name advertised by
the server; live input schemas are still validated. This applies to account
lookup, help, compact discovery/execution and full-list job search. Search
bindings may name `find_jobs` or `upwork__find_jobs`; other tool namespaces and
marketplace-write tools remain rejected. If discovery reports
`required_tool_unavailable`, confirm that the connected catalog contains the
required plain or Upwork-prefixed tool. No tool-mode switch or OAuth reset is
needed solely for this prefix difference.

## Separate Codex setup

For an interactive Codex connection, the official Upwork plugin repository
[documents its installation](https://github.com/upwork/upwork-agent-plugin).
Alternatively, configure the remote MCP server in your user-level Codex settings:

```sh
codex mcp add upwork --url https://mcp.upwork.com/mcp
codex mcp login upwork
```

See the [official MCP client documentation](https://learn.chatgpt.com/docs/extend/mcp).
This authenticates **Codex only**. Radar does not read its credential store,
configuration, browser session or cookies. No Codex/Claude configuration was
modified by this implementation.

## Optional scheduled searches

The default remains `execution_mode: manual`. After Radar-owned OAuth,
account selection, discovery and the read-only binding have all been verified,
the owner may explicitly opt in to bounded searches while the server is
running:

```yaml
job_sources:
  upwork_mcp:
    enabled: true
    runtime:
      execution_mode: opted_in
      scan_on_start: true
      scan_interval_minutes: 60
```

`scan_on_start` queues one run for each server process start, and the interval
is aligned to stable UTC slots so scheduler ticks and restarts cannot duplicate
a run for the same process/start token.
The existing work mode, daily quotas, request budgets, pause/resume controls,
rate-limit backoff and idempotent import ledger still apply. Missing or expired
credentials cause a visible failed run with `reauthorization_required`; the
server never falls back to another credential or silently enables polling.
Set `execution_mode: manual` (or disable the source) to return to on-demand
searches. This is an explicit local opt-in; it is not a claim that Upwork
permits unattended cloud automation.

## Radar-owned OAuth and private state

Initialize the three real configuration files from the repository root (existing
private values are preserved):

```sh
mkdir -p private/config
for namespace in radar scoring notifications; do
  cp -n "config/$namespace.example.yaml" "private/config/$namespace.yaml"
done
```

The runtime requires `radar.yaml`, `scoring.yaml`, and `notifications.yaml`.
If your private files still end in `.example.yaml`, rename them while preserving
their values. There is no template fallback. See [configuration](../../../config/README.md)
for directory and database precedence.
Keep all private values outside Git and the Docker build context. In an untracked
environment file set `RADAR_UPWORK_STATE_DIR` to a dedicated private directory
(mode 0700 on Unix), and `RADAR_UPWORK_ENCRYPTION_KEY` to 32 random bytes encoded
as 64 hex characters. Generate the key with `openssl rand -hex 32`. Load that
private environment before running the CLI/server.

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- \
  upwork --config-dir private/config login
```

Open the printed authorization URL and approve Radar's consent in the browser.
The application binds the configured callback port on **127.0.0.1 only**, validates
PKCE/state through the SDK, and saves the grant encrypted. On a remote server,
use SSH loopback forwarding for that port; do not expose it publicly.

The SDK discovers the authorization server and supports a pre-registered public
client (`RADAR_UPWORK_CLIENT_ID`), a hosted HTTPS Client ID Metadata Document
(`RADAR_UPWORK_CLIENT_METADATA_URL`), or advertised dynamic registration. A hosted
metadata document must describe your own application and the exact configured
loopback redirect URI, with `token_endpoint_auth_method: none` and native-client
metadata. Do not reuse another client's identity or credentials. Dynamic
registration is attempted only by the explicit login command. Upwork's acceptance
of registration/redirects for this standalone client is still a live deployment
check; `oauth_client_registration_failed` is a real blocker, not a successful login.

A session lock prevents simultaneous Radar CLI/server refreshes using the same
state directory. Credentials, selected account, discovery, binding and request
budgets are application-owned, encrypted with authenticated encryption, and
written atomically with private file permissions. SDK refresh persists rotated
tokens. Missing/rejected grants require the same explicit login command again.
Changing the key makes old state unreadable. Store a protected backup of key and
state together, or reauthorize and rediscover. This is for one freelancer; it is
not a multi-tenant authentication system or a distributed secret manager.

## Discover before searching

The [official workflow](https://github.com/upwork/upwork-agent-plugin/blob/master/skills/upwork-workflows/SKILL.md)
requires `list_accounts`, account selection, then live tool/action help. Radar
lists tools, calls `list_accounts`, requires an explicit account index when more
than one account exists, and accepts only an unsuspended freelancer account.
Internal organization IDs/role codes are stored encrypted and never displayed.
Discovery calls `search_tools` when advertised and `get_tool_help` for `find_jobs`.
Every call is checked against the advertised input schema.

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- \
  upwork --config-dir private/config discover --output private/upwork-discovery.json
# If prompted by the diagnostic/list, repeat with --account-index INDEX.
```

The output is created privately and will not overwrite an existing file. It
contains live tools/help, their help hash and a human-readable account label.
Select a read-only search action from that response. Because no authenticated
live schema was available during implementation, Radar requires an operator-
reviewed **binding**, rather than guessing parameter names or result fields.
This is a current setup requirement, not automatic schema adaptation.

Create `private/upwork-binding.json` with these fields, using the actual discovery:

| Binding field | Value to supply |
|---|---|
| `tool` | `find_jobs` or `upwork__find_jobs`; no other marketplace tool can execute |
| `action` | Exact discovered read-only search action |
| `help_hash` | Hash printed in the discovery file |
| `action_pointer` | JSON Pointer into `help` selecting that action's descriptor |
| `parameters_schema_pointer` | JSON Pointer into `help` selecting its JSON Schema |
| `profiles` | Map each configured profile name to actual action parameters matching that schema |
| `records_pointer` | JSON Pointer into a result page selecting its job array |
| `next_cursor_pointer` | Returned cursor pointer, or null when unsupported |
| `cursor_parameter_pointer` | Pointer into profile parameters for the cursor, or null |
| `fields` | Map the normalized field names below to actual per-job JSON Pointers |

The action pointer may select a named action descriptor or the exact action
string from the live input schema enum. The help hash excludes the per-request
`trace_id`; all other help/schema content remains revision checked. Rediscover
and review bindings made before this trace handling change.

The action descriptor must identify the selected action, and live help/tool
annotations must affirm read-only behavior. Unknown/missing evidence fails closed.
Full-list and compact `execute_tool` modes are supported; the enclosing request
is validated against the live tool schema before invocation. Every search
rediscovers the account and help, checks the help hash and validates parameters.
A schema change requires rediscovery/review, not speculative retries.

Supported mappings: `id`, `url`, `title` (required); `description`, `category`,
`job_type`, `skills`, `skill_item_name` (relative to each skill object),
`budget_amount`, `budget_currency`, `weekly_hours`, `posted_at`,
`experience_level`, `payment_verified`, `client_country`, `client_history_good`,
`connects_cost`. Map only fields actually returned. Never map payment verification
to good client history. Budget amount/currency must both be returned; for ranges,
map the actual conservative lower bound. Full range representation is not yet
supported. Job type recognizes hourly/fixed/fixed_price; other/missing values
remain unknown. Missing category uses the existing model's explicit `unknown`
marker. Optional facts remain null; no effort or verified client history is
invented. Missing ID/title/valid returned Upwork HTTPS URL rejects the record.
Missing descriptions or unknown job type cannot produce job-match alerts.

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- \
  upwork --config-dir private/config bind private/upwork-binding.json
```

## Search profiles and the first real search

`job_sources.upwork_mcp` is disabled by default. Enable it in private deployment
YAML or the approved `radar` DB namespace only after login/discovery/binding.
If the namespace already has an approved DB override, update it through
`PUT /v1/config`; editing deployment YAML does not override that snapshot.
Each profile has enabled status, keywords, skills, optional hourly/fixed type,
minimum USD budget, experience levels and result limit. The binding contains that
profile's live-schema upstream parameters. All configured local constraints are
also enforced on returned facts; missing facts fail an explicit constraint.
Keywords require every term in title/description; skills require every skill.
Hourly and fixed budget floors have different units: use separate profiles.

Start/migrate the backend normally, then press **u** in TUI for the default manual
Upwork profile. **m** remains mock. API callers can select a profile explicitly:

```http
POST /v1/upwork/scan
Authorization: Bearer YOUR_RADAR_OWNER_TOKEN
Content-Type: application/json

{"idempotency_key":"FRESH_UUID","profile":"general"}
```

A 202 response means queued, not searched/imported/delivered. Dashboard run status
and `GET /v1/upwork` expose enabled state, credential/connection diagnostics,
selected account label, last successful search, error category, fetched/new/
duplicate/accepted/invalid/filtered counts, partial state and upstream filter
metadata. TUI auto-refresh reads the same state. No permanent MCP connection is
claimed: credentials stored does not prove they remain valid.

Jobs enter the existing canonical upsert/scorer/import transaction, preserving
feedback and application state. Existing event/destination uniqueness prevents
repeated alerts on later scans. Empty pages are successful scans with metadata;
they do not establish that the marketplace is empty. Partial results remain
explicit. Reusing a completed request key does not search again. Queue claims
coordinate through PostgreSQL; completed work is atomic. Interrupted **running**
Upwork searches fail and require a new manual trigger, while mock recovery remains
unchanged. A config revision change during search rejects ingestion and requires
review/retry under current policy.

**Scheduler-triggered Upwork polling is disabled, with a database constraint.**
Scheduled scans and Discord `/scan` remain mock-only. No supported/permitted
unattended Upwork cloud usage model has been established. A worker executing an
explicitly queued manual request does not grant permission to add periodic
polling. Individual calls are bounded and the SDK session is cancelled/closed
after each workflow; there is no user-facing cancel-run control yet.
There is no proposal/application/message/contract action.

## Supabase and deployment

Set `RADAR_DATABASE_URL` to your private Supabase PostgreSQL connection string
with TLS. Prefer direct connectivity; session pooling is the IPv4 alternative.
Transaction pooling has not been validated for this SQLx/migration setup. See
[Supabase connection guidance](https://supabase.com/docs/guides/database/connecting-to-postgres).
Use a dedicated project/database and keep Radar's tables outside exposed Data API
schemas, or otherwise restrict that API's access before storing personal data.
Radar's bearer authorization does not configure Supabase RLS/Data API permissions.

```sh
# Standalone backend; this file does not create a local PostgreSQL service:
docker compose --env-file private/radar.env -f compose.radar.supabase.yaml up --build -d
```

The Compose state volume persists Radar OAuth/budgets across backend restarts.
CLI login/discovery must run on the same host/state, using the same key and user;
for a containerized service, use the same mounted state volume for setup. On
Linux, a temporary host-networked login container can bind the callback to the
host loopback; ordinary Docker port publishing cannot reach a listener bound
only to the container loopback. Alternatively use a protected shared bind-mounted
state directory with matching ownership. Container OAuth setup has not been
verified here. The callback port is intentionally not published by default. Private config can be mounted read-only at `/app/config`.
Credentials/profile files stay outside the build context. Native execution is
also supported. The backend still needs a host even with Supabase managing DB.

Migration **0008** adds source/profile to the existing run ledger, forbids
scheduled Upwork rows, registers canonical `upwork` provenance, and stores only
safe operational search statistics. OAuth/account secrets stay out of Supabase.
Startup applies embedded SQLx migrations; back up a real DB before upgrading.
No production Supabase operations were performed during validation.

## Budgets, troubleshooting and tests

[Official MCP limits](https://support.upwork.com/hc/en-us/articles/56577997707923-Rate-limits-for-Upwork-MCP)
currently permit search/discovery at 15/minute, 150/hour and 750/day. Windows are
fixed, with daily reset at midnight UTC; limits apply to the Upwork user across
clients. Radar's smaller YAML budgets count discovery/tool calls and persist
locally; they cannot account for other clients. Every page costs another call.
Calls are sequential and bounded by time/pages/results. Rate-limit responses
persist a cooldown, honoring Retry-After; transient failures defer later manual
attempts with exponential backoff. There are no automatic tool-call retries or
SSE reconnection loops. Rate-limit compliance does not authorize unattended use.
Upwork explicitly cautions against polling and unauthorized automation; see
[automation policy](https://support.upwork.com/hc/en-us/articles/43342677368467-Use-bots-and-other-automation-properly).

| Diagnostic | Action |
|---|---|
| `reauthorization_required` | Run Radar login; Codex login is separate |
| `oauth_client_registration_failed` | Resolve a supported Radar public-client registration or hosted metadata document |
| `account_selection_required` / select-index diagnostic | Discover and explicitly select the freelancer account |
| `search_binding_required` / `search_schema_changed` | Rediscover and review the private binding |
| `parameters_do_not_match_live_schema` | Correct parameters/mapping against live help |
| `rate_limited` / `request_budget_reached` | Wait; do not repeatedly trigger scans |
| `account_restricted` | Resolve account permissions/restrictions with Upwork |
| `state_permissions_require_*` / decryption error | Fix private directory/file ownership/mode or restore the matching key |
| Partial/ignored filters/empty reason | Inspect diagnostics, enforce local constraints; do not infer marketplace scarcity |

From `rust/`, use the usual fmt/clippy/workspace tests and isolated PostgreSQL
`--include-ignored` suite. CI uses synthetic MCP/OAuth responses. A separate live
smoke test is feature-gated, ignored, requires explicit opt-in and only reads:

```sh
RADAR_TEST_UPWORK_LIVE=1 cargo test -p radar-adapters --features live-upwork-tests \
  --test upwork_live -- --ignored
```

This requires private `RADAR_CONFIG_DIR`, enabled source, Radar OAuth, selected
account and reviewed binding. It never writes to PostgreSQL. Do not enable live
tests in ordinary CI. Remaining live acceptance: consent/registration/refresh
with a real Upwork account, live schema binding, first search and resulting
notification. Unattended cloud scanning remains unsupported and disabled.

## Local verification results

`cargo fmt --all -- --check` and workspace/all-target/all-feature Clippy with
`-D warnings` pass. Default workspace tests: 38 passed, 19 database tests ignored.
With isolated PostgreSQL: 57 passed, none ignored. The feature-gated live test was
not run. Nine Python tests, skill lint and security guards passed; base, merged
development and standalone Supabase Compose configurations validated. Docker
engine build/start remains unavailable in this environment. The BRD and existing
Claude workflows were not rewritten. See the [acceptance matrix](acceptance.md).

## Files changed for Phase 4

- `.env.example`
- `.github/workflows/radar.yml`
- `Dockerfile.radar`
- `README_RADAR.md`
- `compose.radar.supabase.yaml`
- `compose.radar.yaml`
- `config/README.md`
- `config/radar.example.yaml`
- `docs/radar/README.md`
- `docs/radar/architecture/03-integrations.md`
- `docs/radar/plan/acceptance.md`
- `docs/radar/plan/backlog.md`
- `docs/radar/plan/roadmap.md`
- `docs/radar/plan/upwork-mcp.md`
- `rust/Cargo.lock`
- `rust/apps/radar-server/src/lib.rs`
- `rust/apps/radar-server/src/main.rs`
- `rust/apps/radar-server/src/workers.rs`
- `rust/apps/radar-server/tests/cloud.rs`
- `rust/apps/radar-tui/src/client.rs`
- `rust/apps/radar-tui/src/main.rs`
- `rust/apps/radar-tui/src/ui.rs`
- `rust/apps/radar-tui/src/ui/view.rs`
- `rust/crates/radar-adapters/Cargo.toml`
- `rust/crates/radar-adapters/src/lib.rs`
- `rust/crates/radar-adapters/src/postgres.rs`
- `rust/crates/radar-adapters/src/postgres/outbox.rs`
- `rust/crates/radar-adapters/src/postgres/scheduler.rs`
- `rust/crates/radar-adapters/src/postgres/upwork.rs`
- `rust/crates/radar-adapters/src/upwork/mcp.rs`
- `rust/crates/radar-adapters/src/upwork/mod.rs`
- `rust/crates/radar-adapters/src/upwork/normalize.rs`
- `rust/crates/radar-adapters/src/upwork/state.rs`
- `rust/crates/radar-adapters/tests/core.rs`
- `rust/crates/radar-adapters/tests/upwork.rs`
- `rust/crates/radar-adapters/tests/upwork_live.rs`
- `rust/crates/radar-application/src/api.rs`
- `rust/crates/radar-application/src/bots.rs`
- `rust/crates/radar-application/src/config.rs`
- `rust/crates/radar-domain/src/lib.rs`
- `rust/migrations/0008_upwork_manual_scans.sql`


## Local verification on 2026-11-04

Radar-owned credentials successfully authenticated against the official endpoint.
Live tool/account/help discovery and a schema-validated read-only `search` request
returned one real job. The private sample contains source content and is not
committed. Live help uses a JSON Schema action enum; request-specific `trace_id`
is excluded from schema fingerprints. Numeric budgets remain unknown when the
source returns only a display string. This does not establish unattended OAuth
operation or permission for background polling.

To inspect one result before creating a binding, the feature-gated
`official_upwork_live_read_only_sample` test requires explicit
`RADAR_TEST_UPWORK_SAMPLE=1`, `RADAR_CONFIG_DIR`, Radar-owned credentials,
`RADAR_TEST_UPWORK_SAMPLE_PARAMETERS` (an absolute path to reviewed JSON params
with `limit: 1` and no cursor), and `RADAR_TEST_UPWORK_SAMPLE_OUTPUT` (a fresh
absolute private path). It validates the request against the advertised schema,
uses the selected encrypted freelancer account and performs one search. Run it
by name; do not run every ignored live test together. It does not write to
PostgreSQL or send notifications.


The initial database/API failures were resolved by replacing the server's
three-second acquisition timeout with a configurable 30-second default and the
terminal's eight-second total request timeout with a configurable 45-second
default. Private environment values were sourced without displaying them.
The authenticated API queued one manual live scan. Existing repository claim /
execute services processed it without running scheduling or delivery workers:
one real Upwork job, one evaluation, one new canonical job and zero duplicates.
Reusing its request key returned the completed run without another search.
A verification `Save` action persisted through the shared feedback API. After
restarting the backend, the terminal client read one job, one feedback entry,
configuration revision 4 and the successful Upwork counters. Manual source
execution is enabled in the approved DB namespace; deployment YAML was not
changed. The original profile result limit was restored after the one-result
check. Scheduled Upwork execution remains disabled.

The bundled synthetic profile was used for scoring; this does not validate the
freelancer's personal fit. The returned job scored `skip` with `unsupported_skill_requirement`, so no job alert was
queued or delivered. Real Discord delivery/interactions remain unverified;
provider readiness reports incomplete interaction configuration. No marketplace
writes or destructive production-database tests ran. No new migrations were
required. Final validation: 49 regular Rust tests passed, 19 isolated-database
tests skipped; fmt/clippy, nine Python tests and repository guards passed.
Both Docker Compose configurations validated.
