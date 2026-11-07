# Acceptance test matrix

| Scenario | Expected | Requirement |
|---|---|---|
| Import same mock job twice | One canonical job; no duplicated recommendation | AC-001 / FR-004 |
| Unavailable Upwork MCP auth | Provider disabled with actionable diagnostic, no scraping fallback | AC-002, AC-073 |
| Paused work mode | No new scan, no ordinary job push | AC-020 / FR-025 |
| Demand > weekly availability | Hard veto, even for strong skill match | AC-010 / BR-010 |
| Unavailable budget/client data | Unknown flag and lower confidence, no invented amount | AC-011 / BR-012 |
| Same event delivered twice | One delivery per configured destination | AC-050 / FR-054 |
| Forged Discord interaction | Reject and preserve job state | AC-051 / BR-050 |
| Discord `Skip` | Shared DB changes, same result in TUI | AC-052 / FR-055 |
| TUI config edit | Effective config revision changes, YAML example untouched | AC-022, AC-061 |
| Network disconnect | UI reports failure; no false success | AC-062 |
| Job description contains malicious instructions | No shell/file/config execution | AC-072 / BR-071 |
| Proposal built from academic/demo projects | Accurate provenance, no fake paid client history | AC-040 / BR-040 |
| Proposal drafted in MVP | Saved local/cloud draft; zero marketplace writes | AC-042 / BR-042 |
| Restart cloud service | Persisted jobs, config, outbox and run state survive | FR-023, FR-054 |

## Test layers

The owner-authorized [local web client](web-ui.md) uses the same API and services.
Its browser tests verify jobs, feedback, mock AI/proposal review/edit/copy,
profile/template persistence, stale updates, disconnects, HTML escaping and
mobile layout. Cookie auth tests verify owner/read-only permissions, one-use
codes, HttpOnly/SameSite flags, CSRF rejection and logout revocation. The
2026-11-07 isolated workspace run passed 111 tests; real provider availability
is a separate acceptance gate.

Database reliability verification: **104 Rust tests passed** with an isolated
PostgreSQL instance and ignored tests enabled (2026-11-05). This includes
single-connection transaction regression, interrupted connections, pool
saturation, rollback and automatic recovery. Actual local PostgreSQL restart
and read-only Supabase SQLx/HTTP probes also passed. See
[evidence, limits and deployment decision](database-reliability.md).

- Unit: scoring, hard veto, config merge, routing/quiet hours, identifiers.
- Integration: Axum API, SQLx migrations, optimistic updates, scheduler locks/outbox.
- End-to-end: mock job -> evaluation -> persisted event -> fake/Discord notification -> feedback -> TUI.
- External smoke (manual): Discord signature exchange and Upwork cloud MCP feasibility, **not** required in CI.

CI should pass with synthetic data and **no live Upwork, Codex or Discord credentials**.

## Phase 2 executable coverage

`rust/apps/radar-server/tests/cloud.rs` runs against real PostgreSQL and TCP HTTP.
It covers migrations, unique/foreign-key constraints, duplicate imports and
fetch history, deterministic Phase 1 scoring reuse, owner/read-only auth, strict
request rejection, config validation/revisions/concurrency, feedback persistence,
human-reported application outcomes, manual import privacy boundaries, persisted
quotas, paused mode, namespace precedence, actual server-process restart and
TUI client round trips/disconnects. All six screens render at multiple sizes.
Run these opt-in tests with `RADAR_TEST_DATABASE_URL` and `--include-ignored`; a
default test run that skips them does not establish Phase 2 acceptance.
Production TLS/deployment acceptance remains environment-specific;
see [Phase 2 operation and limitations](cloud-sync.md).

## Phase 3 executable coverage

The same real-database integration suite now includes 18 tests (46 Rust tests
total with `--include-ignored`; default runs pass 28 and skip 18). Added coverage:

| Acceptance | Executable evidence in `apps/radar-server/tests/cloud.rs` |
|---|---|
| Timezone, missed slots, dynamic quiet hours | `missed_slots_coalesce_and_live_quiet_hour_edits_release_pending_work`; application DST/local-day unit test |
| Modes, queue admission, per-run/day quotas | `manual_queue_limits_per_run_daily_caps_and_busy_resume_use_live_configuration` |
| Coordination, expired claim recovery, idempotence | `scheduler_coordinates_workers_recovers_claims_and_preserves_quotas` |
| Multi-instance routing, dedup, scan failures | `outbox_routes_multiple_instances_without_duplicate_deliveries_and_records_failures` |
| Quiet hours, caps, retry-after, ambiguous send recovery | `quiet_hours_rate_limits_daily_caps_and_ambiguous_restart_are_safe` |
| Signed commands/buttons/Skip modal, feedback and terminal sync | `signed_discord_commands_buttons_modals_and_terminal_share_persisted_state` |
| Raw-body verification, replay, expiry, authorization revocation | `discord_raw_body_replays_expiry_and_authorization_changes_fail_closed` |
| Actual startup background processing and persistent restart | `background_service_starts_automatically_and_restart_does_not_repeat_work` |

The Discord adapter's local HTTP stub additionally verifies card components,
nonce, mention suppression, command registration, rate limits and deferred reply
edits. Existing Phase 2 tests remain active; all seven TUI screens render at
four sizes. Migration tests upgrade existing version-1 data through version 7.

Automated end-to-end acceptance uses fake Discord, real PostgreSQL and TCP HTTP.
Real Discord acceptance steps 7–9 require the [manual check](discord-setup.md);
they have **not** been verified against Discord. Docker image build/start and
public TLS ingress likewise remain deployment checks. No Upwork/Codex integration
or marketplace-write acceptance is claimed.

## Phase 4 executable coverage and external gate

Final validation: 38 credential-free Rust tests pass; 19 DB tests are skipped by
default. With isolated native PostgreSQL and `--include-ignored`, **57 tests pass**
with none ignored. The opt-in live Upwork test is additionally feature-gated and
was not compiled/run in the normal test suites.

- Adapter tests cover normalization/missing fields, account selection, full and
  compact discovery, schema changes, read-only gating, pagination, duplicate
  results, empty reasons, ignored filters/local constraints, partial rate limits,
  encrypted state/session locking and UTC request budgets.
- Local fake HTTP servers exercise the actual rmcp Streamable HTTP protocol,
  401/403/429/5xx classification and Retry-After, OAuth PKCE/CSRF, token refresh,
  encrypted persistence and credential reload/clear.
- `upwork_manual_source_uses_existing_queue_scores_feedback_outbox_and_never_schedules_live_work`
  exercises the owner/read-only HTTP boundary, existing queue/import/scorer,
  canonical upserts, duplicate notification prevention, saved feedback, TUI HTTP
  snapshot, paused-mode preflight and interrupted-manual-run handling.
- Existing mock/manual, migration, restart, Discord and terminal tests still pass.
  Migration tests upgrade existing data through version 8.

Real Radar-owned Upwork consent/refresh, live account/search schemas, real search
and resulting Discord delivery remain **unverified**. The public MCP/OAuth
metadata was reachable; that is not authenticated acceptance. Supabase production
was not accessed. Scheduled Upwork polling is disabled and not claimed permitted.
See [manual setup and limitations](upwork-mcp.md).


### 2026-11-04 live verification update

The credentialed official MCP discovery, single-result schema probe, and existing
`official_upwork_live_read_only_smoke` adapter test passed. Verification exposed
and corrected action-enum binding support and per-request trace IDs in schema
hashing; regression tests cover both. The conservative application request budget
stopped one attempt; the retry passed after the reported delay. Supabase migration
metadata shows versions 1–8 applied. No database integration tests were run against
production Supabase. This is not full live ingestion/Discord acceptance, nor
verification of interactive consent, refresh, or unattended operation.


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


## Phase 5 verified manual slice — 2026-11-04

Final validation from `rust/`: fmt and Clippy (workspace/all targets/all features,
warnings denied) passed. Credential-free tests: **56 passed, 23 database tests
ignored**. With an isolated local PostgreSQL 18.6 database: **79 passed, none
ignored**. Neither live Upwork nor live AI test features were enabled. Python's
nine tests, skill lint, security guards and both Compose configurations passed.
Production Supabase data/configuration and private profiles/credentials were not
modified by Phase 5 verification; migration 0009 was tested only locally.

| Acceptance | Executable evidence |
|---|---|
| Grounding, truthful training context, injection, malformed output, unknown facts, screening answers | `radar-adapters/tests/ai.rs` |
| Tool-free request, private proof exclusion, response usage/cost metadata | `responses_protocol_is_tool_free_redacts_proofs_and_accounts_for_usage` |
| Mock job → deterministic score → AI → approved evaluation → draft → edit → regeneration → TUI/copy effect | `manual_ai_mock_to_reviewed_proposal_edit_regeneration_and_terminal_is_persisted` |
| Idempotency/concurrency, quotas, timeout/cancel, expired-run recovery | `manual_ai_coordinates_quota_concurrency_cancel_timeout_and_restart_recovery` |
| Owner/read-only authorization, stale inputs, preserved user edits | Manual slice integration test |
| Provider failures, invalid claims/output, input/cost bounds | `ai_rejects_provider_failures_bad_outputs_and_overbudget_or_oversized_requests` |
| No AI bypass of deterministic skip | `ai_cannot_bypass_a_deterministic_veto_even_with_an_approved_analysis` |

Standalone HTTP demonstration used the synthetic **small fixed-price repair**
job, deterministic score 100 and a mock strong-candidate assessment. Its proposal
used a labelled training fact, was manually edited/approved as revision 2, and
left application status `none`. Two completed AI operations and the approved
edited text survived an actual backend process restart. The terminal client
successfully authenticated after restart with Upwork disabled. Terminal rendering
and clipboard-request effects are automated-test verified; real terminal OSC52
clipboard behavior was not manually confirmed.

**Real AI verification:** none. Responses API transport/decoding is fixture-tested;
a feature-gated synthetic opt-in smoke test is provided. Installed Codex CLI
capabilities were inspected, but no CLI provider was enabled or inference run.
The tool-free API is the selected real adapter. Real model quality, credentials,
model access and billing remain an opt-in deployment check. Reasoning prose and
human edits still require factual/privacy review. See [Phase 5 guide](phase5-ai.md).

## Phase 6 verified manual learning slice — 2026-11-04

Executed from the Rust workspace (with `--manifest-path rust/Cargo.toml` from
repository root): formatting check and workspace/all-target/all-feature Clippy
with warnings denied passed. Credential-free tests: **63 passed, 28 isolated-DB
tests ignored**. Isolated PostgreSQL 18.6: **91 passed, none ignored**. Nine Python
tests, skill lint, security guards and local Markdown link checks passed. No live
AI, Upwork or Discord calls were made. Production Supabase and private
configuration/profiles/credentials were not modified. Migration 0010 was applied
only to disposable local test schemas.

| Acceptance / coverage | Executable evidence |
|---|---|
| AC-031, AC-033: cold start, sparse/conflicting evidence, positive/negative reasons, stronger confirmed outcomes, unknown metadata, caps, hard veto, time weighting | `radar-adapters/tests/learning.rs` (seven deterministic tests) |
| AC-032–034: API → feedback → candidates → shadow → activation → ranking → rollback → disable → confirmed reset, no private prose export, baseline pagination with no usable model | `preference_shadow_activation_negative_scope_rollback_disabled_and_reset_are_end_to_end` |
| AC-034–035: correction/exclusion, stale CAS, old-cutoff admission rejection, immutable historical source rather than future import | `feedback_corrections_invalidate_models_without_losing_history_or_leaking_future_metadata` |
| AC-032, AC-034, AC-036: coordinated rebuild, idempotency, suppression/restoration, rejected/stale feature/config versions, control revisions | `preference_rebuild_coordination_controls_and_versions_are_owner_controlled` |
| FR-037: prior validated fake AI, draft history, owner disagreement, manually reported lifecycle, actual effort comparison; no automatic prompt/rule change | `preference_calibration_uses_recorded_outcomes_and_predecision_ai_without_changing_facts` |
| AC-036: real backend process restart, activated weights and exact ranking persistence, global pagination/ties, feedback retained | `activated_preference_weights_and_exact_ranking_survive_actual_process_restart` |
| AC-035: future labels/revisions excluded, held-out jobs disjoint, production verdict priority retained | `corrections_and_chronological_holdout_do_not_leak_future_labels`, `backtest_preserves_production_verdict_priority_before_numeric_scores` |
| Owner/read-only access, migration integrity, Phases 1–5 regressions | Full workspace suite, including shared API middleware and real PostgreSQL tests |

Demonstrated A–F using 13 synthetic stored jobs through real authenticated HTTP:
empty model reproduces baseline; five positive events promote an untouched job
**85 → 93 (+8 cap)**; budget-coded negatives demote the relevant fresh job without
negative technology evidence; a hard-veto job remains Skip with zero adjustment;
rollback restores the entire same-input ranking/scores; disabling restores exact
baseline and leaves history intact. Reset preserves 13 jobs, ten feedback events
and all three model versions. Learning screen rendering is verified through
Ratatui TestBackend. Interactive terminal use/clipboard delivery and real-owner
predictive usefulness have not been manually validated. The CLI reproduction,
controls, migration/configuration, formulas, sample limitations and exact file
inventory are in the [Phase 6 guide](phase6-learning.md).

## Profiles editor verified — 2026-11-05

Formatting and workspace/all-target/all-feature Clippy passed with warnings denied.
Default tests: **65 passed, 30 database tests ignored**. With an isolated local
PostgreSQL 18.6 instance: **95 passed, none ignored**. Nine Python tests, skill
lint and security guards passed. No production Supabase, private Markdown or
credentials were modified; migration 0011 was exercised only in local schemas.

- `profile_setup_forms_preview_authorization_versions_and_active_pipeline_are_persisted`:
  labelled add-skill form, explicit save confirmation, owner/read-only controls,
  missing/unknown/oversized/invalid metadata rejection, redacted errors, stale
  revisions, unsaved edit preservation, evidence preview, rendered 120×40 and
  72×24 terminal views, active-profile deterministic/AI use and version recovery.
- `owner_profile_survives_actual_backend_restart_without_rewriting_deployment_markdown`:
  real backend processes use the approved DB profile after restart; source files
  remain unchanged.
- `radar-tui::ui::profile::tests`: add/edit data mapping, prose preservation,
  new evidence sharing defaults, confirmation, invalid rates, bounded Unicode
  paste and field clearing.

Manual terminal appearance, OS clipboard/paste behavior and real-owner CV content
still require operator review. No paid inference was needed. See
[Profiles setup](profiles.md) for keyboard controls, API, source precedence,
limits and changed files.

## Researched proposal templates — verified 2026-11-05

Five optional briefs and manual Markdown patterns are implemented from reviewed
Upwork guidance and Reddit discussions, with source/retrieval limitations recorded
in [research and operation](proposal-templates.md). They are editorial defaults,
not verified conversion optimization. Public profile writing guidance was updated;
private profiles/configuration and production Supabase were not modified.

Validation: fmt check and workspace/all-target/all-feature Clippy passed. Default
suite: **68 passed, 31 database tests ignored**. Isolated local PostgreSQL:
**99 passed, none ignored**. Nine Python tests, skill lint, security guards and
local documentation link checks passed.

- `proposal_templates_personalize_verified_relevant_evidence_and_keep_one_closing_action`:
  all five synthetic brief shapes, actual job opening, context-labelled profile
  evidence, bounded length and one closing action.
- `proposal_template_guards_reject_filler_irrelevant_claims_placeholders_and_excess_questions`:
  invalid openings/slots, duplicate/unrelated facts, question/length bounds and
  no matching evidence.
- `proposal_template_catalog_validation_versions_and_evidence_order_are_deterministic`:
  optional defaults, missing/duplicate IDs, content revisions and stable evidence
  ordering.
- `proposal_template_selection_is_explicit_versioned_grounded_and_preserves_edits`:
  real HTTP/API plus TUI selection, reader denial, missing-template rejection,
  persisted definition/hash, idempotency, regeneration preserving edits and
  application status unchanged.

No live AI inference, real proposal submission, client messages, Upwork polling,
or private-profile publication occurred. No new migration/dependency is required.
Real provider wording and effectiveness remain unverified; manual review is required.
