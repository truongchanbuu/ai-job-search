# Phase 5 — manual AI review and grounded proposals

Phase 5 extends the existing Rust application port, PostgreSQL repository,
owner-authenticated Axum API and Job Details workflow. Deterministic evaluation
and application history remain separate. Mock/manual jobs work with Upwork off.
There is no automatic AI processing on import, scheduler tick or notification.

## Providers and configuration

`ai.runtime.enabled` is false in committed templates. An older private/DB
configuration without `ai.runtime` also disables calls. Copy the runtime section
from `config/radar.example.yaml` into your **private** `radar.yaml` before enabling.
The normal approved DB namespace > deployment YAML precedence applies. If a DB
`radar` override already exists, update its effective namespace through Config
in the TUI or owner `PUT /v1/config`; editing YAML does not replace approved DB
values. New missing optional runtime sections reconcile from deployment YAML.

Providers:

- `mock`: deterministic, offline `MockAiProvider`; zero inference cost. Use this
  for the demo and CI. It is a fixture provider, not a real model assessment.
- `openai_responses`: one HTTPS Responses request, no tools, no MCP, no files,
  `store: false`, strict JSON Schema and bounded output tokens. Configure a model
  that supports structured output and `RADAR_OPENAI_API_KEY` (or the environment
  reference named in `api_key_env`). Credentials never enter prompts or DB rows.

`evaluation_rubric`, model, size bounds, proposal length, timeout, concurrency,
per-kind daily limits and reservation amount are configuration values. The
active mode's existing deep-evaluation quota also caps evaluations. Daily quotas
use the configured IANA timezone. Failed/cancelled requests consume quota and
cost reservation; reusing a key does not make another provider call. Provider
failures are not automatically retried. Set token-price metadata explicitly if
estimated USD accounting is wanted; no marketplace/model price is guessed.
Reservations and estimates are **not** a provider-side spending guarantee.
Use the provider's billing controls separately. Token usage is saved when
returned, including when model output is rejected after a successful response.

The implementation follows [OpenAI structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs).

### Local Codex CLI decision

Installed CLI `0.166.0` was inspected with `codex --version` / `codex exec --help`.
It advertises non-interactive exec, output schemas, ephemeral operation, read-only
sandboxing and ignoring user configuration. See [official non-interactive guidance](https://learn.chatgpt.com/docs/non-interactive-mode).
A read-only sandbox is not a sufficient guarantee that tools cannot read unrelated
files. Phase 5 therefore selects the tool-free API as its first real provider;
**no `codex_cli` provider is registered**. Setting that name is rejected, rather
than spawning an unrestricted agent. There is no child process, repository
working directory, copied CLI OAuth credential or inherited CLI/MCP configuration
in the implemented inference path. Codex login does not authenticate the API.
A future CLI adapter needs independently verified filesystem, tool and process
cancellation isolation before registration. The async `AiProvider` port supports
that extension without a second workflow.

Cloud operation requires its own API key, configured model, outbound HTTPS and
billing access. No local Codex login/session is assumed in Docker or cloud.

## Verified profile evidence

Continue using the private Markdown modules. Identity skill names and context
are read from existing typed front matter. Their raw proof prose is **excluded**
from AI input, including proof strings embedded in deterministic evaluation
explanations. Unrelated Markdown modules and documents are not forwarded.

Optional `portfolio.md` front matter adds evidence:

```yaml
---
evidence:
  - id: project:my-reviewed-project
    source: portfolio.md#my-reviewed-project
    context: training
    statement: A factual, reviewed summary safe to share externally.
    verified: true
    shareable: true
    portfolio_url: null
---
```

Use `employment`, `contract`, `academic`, `training` or `personal`; certifications
may be recorded as reviewed training evidence. Never relabel a training project
as employment. An entry with `verified: false` is excluded. Entries without
explicit sharing approval cannot be used as proposal claims. The `source`
reference names a profile file/section; the complete evidence/profile revisions
are retained with each operation. The application never edits these Markdown
facts. Verify them yourself before setting the flags; software cannot establish
whether a claimed real-world achievement actually happened.

Only the following go to the configured external provider: selected job and
redacted deterministic assessment, verified skill labels/context, reviewed
portfolio evidence, configured availability/minimum rate, the private
`proposal-style.md`, rubric, and up to ten relevant feedback entries. Keep that
writing-style file and all shareable summaries free of confidential information.
Credentials, config secrets, Upwork OAuth state, raw identity proofs and unrelated
profile prose are excluded. Prompts/results are not logged by default. Outputs
and curated evidence are persisted privately in PostgreSQL for review/history.

## Evaluation and factual gates

Typed version-1 output covers technical fit, literal supporting strengths,
missing requirements, risks, assumptions, unknowns, effort range/dependencies,
pricing considerations, confidence and next action. Every claimed strength must
quote a supplied verified statement and cite its evidence ID. Unknown IDs,
unexpected enums, malformed schemas and invalid/negative/reversed effort ranges
fail validation. Reasoning/risk prose remains a model assessment requiring human
review; schema validation does not prove semantic correctness.

Effort may remain null when scope is insufficient. A non-null best/conservative
range is advisory, with assumptions and confidence. Hourly comparisons use only
returned budget facts and the configured minimum. No marketplace averages,
guaranteed payment reliability, fixed quote or guaranteed deadline is inferred.
Optional fixed-price suggestions remain advisory reasoning; there is no automatic
quote calculation or commitment.

A deterministic `skip` blocks proposal generation even if an AI recommendation
is favorable. The independent AI output never changes the score, vetoes, hard
filters or application status. A high recommendation is not permission to apply.
The user must review/approve a current completed evaluation before drafting.

Proposal plan prose cannot carry candidate experience, credentials, numeric
promises, prices, deadlines or invented links. Candidate claims are assembled
verbatim from explicitly shareable evidence, prefixed with their actual context.
The provider leaves `proposal_text` empty for server assembly, or supplies the
exact same canonical text; arbitrary additional paragraphs are rejected. Plan
validation is deliberately conservative and may reject legitimate wording.
Manual edits remain possible. Proposals include at most two clarification
questions; assumptions/warnings stay separate from copy-ready text. Screening
answers are a separate structured list: each question must occur verbatim in the
job description and each answer must quote context-labelled shareable facts.
They are shown separately in proposal review, rather than merged into the main
copy-ready proposal. Questions requiring additional personal facts need manual
answers; missing evidence is never fabricated.

## API and operation lifecycle

All routes reuse the existing bearer authentication. GET permits read-only tokens;
mutations require the owner token. JSON request fields reject unknown keys.

| Operation | Endpoint/body |
|---|---|
| Evaluate | POST `/v1/jobs/{id}/ai/evaluate` with `idempotency_key`, `expected_revision`, `evaluation_id: null` |
| History/status | GET `/v1/jobs/{id}/ai` |
| Approve evaluation | POST `/v1/jobs/{id}/ai/{operation}/approve` with `expected_revision` of job |
| Generate/regenerate | POST `/v1/jobs/{id}/ai/proposals` with new `idempotency_key`, job `expected_revision`, approved `evaluation_id` |
| Cancel | POST `/v1/jobs/{id}/ai/{operation}/cancel` |
| Save/edit/approve text | PUT `/v1/jobs/{id}/ai/proposal` with draft `expected_revision`, `text`, `approve` |

202 acknowledges a recorded operation, not a completed model call. A request
launches exactly one bounded task; no automatic AI worker loop is created.
PostgreSQL serializes quota admission and rejects overlapping operations.
History reports running/completed/failed/cancelled and redacted error categories.
Cancellation is checked during the request task; dropping inference cancels its
HTTP future, but the external provider may already have charged for work.
Expired leases are shown as failed and reconciled on startup/new owner requests.
Interrupted requests never automatically restart paid work; use a fresh key.

History currently returns the last 50 operations and 100 proposal revisions.
Job payload, deterministic evaluation, feedback revision, profile revision and
config revision determine staleness. Any config revision conservatively makes
old assessments stale. Changed profile files take effect on server restart,
matching the existing profile-loading lifecycle. Re-evaluate/reapprove stale
work. User edits remain stored when AI drafts regenerate, and regeneration
clears final approval for the new revision. Generated text, edited text and final
approved text are separate fields/versions. None changes application submission
status or calls a marketplace write API.

## Terminal workflow

Select a job in Explorer and press Enter. Existing navigation/feedback keys stay.

| Key in Details/Evaluation | Action |
|---|---|
| `e` | Explicit evaluation |
| `a` | View AI analysis, evidence, effort, uncertainty and operation status |
| `b` | Approve current completed evaluation for drafting |
| `G` | Explicit generate/regenerate |
| `P` | View proposal and its AI original |
| `E` | Edit latest proposal; Enter adds a newline |
| Ctrl+A in editor | Toggle final user approval |
| Ctrl+S in editor | Save a new revision to server |
| `Y` | Request copying approved text via OSC52 |
| `C` | Cancel running operation |
| `r` | Refresh status; does not invoke AI |
| `3` / `4` | Return to deterministic job/score view |

Clipboard support/permission depends on the terminal. Radar says it sent a
clipboard request, not that clipboard contents were confirmed. No external
clipboard subprocess reads files. Review user edits for truth/confidentiality
before final approval; the server cannot certify arbitrary human-authored text.

## Reproducible synthetic demo and tests

Use a disposable local PostgreSQL database, never production Supabase for tests.
Make private runtime config copies with plain filenames, enable AI with provider
`mock`, leave Upwork off, and use `rust/fixtures/profiles`. Start the backend:

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- serve \
  --config-dir /absolute/path/to/demo-config \
  --profile-dir rust/fixtures/profiles --no-workers
```

Set `RADAR_DATABASE_URL` to that disposable database and `RADAR_API_TOKEN` to a
synthetic owner token. Start `radar-tui -- --allow-local-http`, import mock jobs
through owner POST `/v1/imports` (`source: mock`, fresh idempotency key, `jobs:
null`), select **Synthetic API debugging**, then `e`, `r`, `a`, `b`, `G`, `r`, `P`,
`E`, Ctrl+A, Ctrl+S, `Y`. No source scheduler/Discord delivery worker is needed.
`m` queues scans and requires workers; the demo uses direct mock import instead.

Automated complete demo (fresh schema per test):

```sh
cd rust
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RADAR_TEST_DATABASE_URL=postgres://USER@127.0.0.1:PORT/DISPOSABLE_DB \
  cargo test --workspace -- --include-ignored
```

Tests cover grounding, training/paid distinctions, injection text, malformed
outputs, weak/strong matches, unclear scope, budget advice, quotas, concurrent
idempotency, cancellation/timeout/failure, restart recovery, stale inputs,
read-only authorization, revisions, terminal rendering/copy effect and the
unchanged deterministic veto. Existing Phase 1–4 tests run in the same suite.

Opt-in real API smoke (synthetic job only; no proposal submission):

```sh
# Load private RADAR_OPENAI_API_KEY without displaying it; explicitly choose MODEL.
cd rust
RADAR_TEST_AI_LIVE=1 RADAR_AI_LIVE_MODEL=MODEL \
  cargo test -p radar-adapters --features live-ai-tests --test ai \
  live_tool_free_provider_evaluates_only_a_synthetic_job -- --ignored --exact
```

Ordinary CI, including ignored database tests, does not enable this feature.
The real provider requires separate credentials/billing; no live inference was
performed during implementation. Mock-provider success does not establish real
model quality, API account/model compatibility, billing accuracy or clipboard
behavior on a user's actual terminal.

## Deployment and troubleshooting

Migration `0009_manual_ai.sql` is additive: AI operations/evidence/output metadata
and append-only proposal revisions referencing existing jobs. No job/scoring/
feedback/application table is replaced. `radar-server migrate` or normal startup
applies it once. Production Supabase was not used for Phase 5 tests or migrated
by this implementation. Back up deployment data through your normal process.
Docker Compose passes the default AI key environment reference; custom secret
names require corresponding container environment wiring. Private profiles stay
outside the image/build context.

- Disabled: add/configure runtime and explicitly enable the effective DB namespace.
- 403: the read-only token cannot spend inference or edit/approve drafts.
- Concurrency/quota: inspect existing operations; cancellation/failure still counts.
- Invalid evidence: review source IDs, verified/shareable flags and context.
- Stale: refresh job/profile/config, evaluate again, then review that assessment.
- Provider failure: check credentials, selected model, access/billing and network;
  no automatic retry or secret-bearing upstream body is exposed.
- Proposal rejected: inspect evidence, word bounds and constrained plan rules;
  do not relax safety gates to force an unsupported claim through.


## Exact Phase 5 changed-file inventory

Paths are relative to the repository root. Existing uncommitted Phase 0–4 files
and unrelated user changes are preserved; this list is scoped to Phase 5.

```text
rust/crates/radar-application/src/ai.rs
rust/crates/radar-application/src/lib.rs
rust/crates/radar-application/src/config.rs
rust/crates/radar-adapters/src/ai.rs
rust/crates/radar-adapters/src/lib.rs
rust/crates/radar-adapters/src/postgres.rs
rust/crates/radar-adapters/src/postgres/ai.rs
rust/crates/radar-adapters/Cargo.toml
rust/crates/radar-adapters/tests/ai.rs
rust/apps/radar-server/src/ai.rs
rust/apps/radar-server/src/lib.rs
rust/apps/radar-server/src/main.rs
rust/apps/radar-server/tests/cloud.rs
rust/apps/radar-tui/src/client.rs
rust/apps/radar-tui/src/ui.rs
rust/apps/radar-tui/src/ui/view.rs
rust/apps/radar-tui/src/main.rs
rust/apps/radar-tui/Cargo.toml
rust/Cargo.lock
rust/migrations/0009_manual_ai.sql
rust/fixtures/profiles/portfolio.md
config/radar.example.yaml
templates/radar-profile/portfolio.md
.env.example
compose.radar.yaml
compose.radar.supabase.yaml
docs/radar/plan/phase5-ai.md
docs/radar/plan/acceptance.md
docs/radar/plan/roadmap.md
config/README.md
docs/radar/README.md
README_RADAR.md
docs/radar/standards/proposals.md
docs/radar/architecture/03-integrations.md
```


Retention limitation: operation/draft history is retained until an operator's
explicit data-management policy is applied. There is no automatic purge or
self-service delete endpoint in Phase 5. Define access, backup and retention
procedures before production use; this implementation does not silently delete
existing jobs, profiles or proposal history.


## Verification result

2026-11-04: fmt, all-feature/all-target Clippy and repository guards passed.
Default Rust suite: 56 passed, 23 ignored; isolated PostgreSQL suite: 79 passed,
none ignored. Standalone synthetic HTTP evaluation/review/draft/edit/approval
and actual backend restart were exercised; terminal-client reads passed with
Upwork disabled. No live AI call or real clipboard acknowledgement was verified.
No production Supabase migration or destructive test was run. The migration,
config and guide are ready for review before enabling external inference.
