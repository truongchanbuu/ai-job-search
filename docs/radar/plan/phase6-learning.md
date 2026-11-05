# Phase 6 — adaptive ranking and calibration

## Architectural assumptions (before implementation)

- This is a single-owner application. Reuse existing jobs, deterministic evaluations,
  feedback, application transitions and owner/read-only authentication. No new job,
  score or outcome store is introduced. No external provider calls are needed.
- Interested/Save express preference; authenticated, manually reported application
  lifecycle events are stronger outcome evidence, not independent platform proof.
  Won/Lost use Hired/Rejected. Not suitable uses Skip + reason; Withdrawn and a
  neutral Other action fill actual gaps in the existing enum.
- Each feedback record uses its contemporaneous stored deterministic evaluation.
  Missing historical inputs stay excluded/unknown. Never use a later job import
  as historical evidence. Corrections are append-only learning-attribute revisions.
- Learning uses a small allowlisted set of technologies/categories and deterministic
  budget, scope, workload, effort and verified client indicators. No client IDs,
  country, free-text feedback, job prose or AI confidence become learned features.
- Each job contributes its latest usable explicit learning signal, not every click.
  A neutral rejection does not erase earlier interest or become technical dislike.
  Negative reasons target their feature family; a schedule conflict cannot penalize
  technology. No feedback and no client response are not negative labels.
- Feature means shrink toward zero. Unique-job sample safeguards, per-feature limits,
  an overall cap and the existing verdict groups constrain display adjustments.
  Historical base scores/evaluations are never rewritten. Skip/null-score jobs get
  no adjustment. Disabled/insufficient/stale models retain exact baseline ordering.
- Models are immutable, versioned candidates. Rebuild/preview/backtest is manual;
  activation/rollback is transactional and owner controlled. Feedback correction
  invalidates old model generations. Reset requires confirmation and changes
  activation state only; it never deletes jobs, drafts, feedback or applications.
- Backtests fit a separate chronological training prefix, exclude held-out jobs,
  and use correction attributes as of training cutoff. Report explicit-feedback
  coverage and sample sizes, not population accuracy or causal findings.
- Personalization affects display ordering only, not notification thresholds,
  deterministic eligibility, AI approval gates, private Markdown or marketplace
  actions. Existing Discord feedback remains on the shared feedback service.

## Model, signals and privacy

`radar-application::learning` owns pure extraction, fitting, explanations and
backtesting. The existing PostgreSQL adapter owns history, coordination and
activation. Existing imports, scoring, application transitions, feedback and
Discord services remain the only paths for those operations.

Features have version `radar-features-1`. They use configured technology/category
allowlists, known job type, the existing budget/availability scoring signals,
explicit scope, source-supplied effort/workload, recognized experience levels,
and known payment/history indicators. Missing values add no feature. Client
identity/country, prose, AI confidence and private Markdown are excluded. Phase 5
assessments are compared in diagnostics, not used as learning labels/features.

The current action mapping is configured in `scoring.example.yaml`:

| Signal | Interpretation |
|---|---|
| Interested / Save | Weak preference; not conversion success |
| Applied / Interview | Owner-confirmed application progression |
| Hired / Completed | Stronger, manually reported outcomes; not independent Upwork verification |
| Skip / NotAvailable / Withdrawn | Negative only for the explicitly relevant reason family |
| Rejected / Other | Neutral; rejection is not technical incompetence |

`not suitable` uses Skip with a reason; Won/Lost use Hired/Rejected. No reply or
missing feedback generates no label. Other preserves the current job decision
and application status. Actual effort can be attached only to Completed feedback.
Optional free-text explanations remain in the existing feedback history; they
are excluded from model snapshots, learned explanations and Discord notifications.

A budget reason updates budget features; schedule reasons update
availability/workload and deadline reasons update effort/workload; technology mismatch updates technologies and
experience level; difficulty updates effort/experience; unclear scope updates
scope; client risk updates known client indicators; lack of interest updates
category/job type. Competition, another opportunity and Other do not create proxy penalties.
NotAvailable with no code targets availability only. A positive event with a
negative reason does not reward that reason's feature family. Arbitrary prose is
never parsed; exact legacy reason identifiers remain recognized.

Each distinct job contributes its latest usable non-neutral signal. A later
neutral rejection does not erase earlier interest. Excluding a feedback record
removes that event, so an earlier still-valid event for that job may contribute.
To remove all evidence for a job, exclude each contributing event. Outcome
correction here changes learning attributes, not the authoritative application
lifecycle; exclude an incorrectly reported outcome and review the stored history.

For feature f, with optional age decay d:

```text
points(f) = max_feature_points * sum(action_weight * d) / (prior_strength + sum(d))
d = 1                                  # default
  = 2 ^ (-age_days / half_life_days)     # explicit optional configuration
adjustment = clamp(sum(matching unsuppressed points), -max_adjustment, max_adjustment)
final = clamp(base + adjustment, 0, 100)
```

Per-feature sample minima and the global distinct-job minimum gate adjustments.
Per-feature points are also capped. Default adjustment limits are defined only
in YAML. Sparse estimates shrink toward zero. Confidence describes evidence
strength, not a success probability. Explanations show actual raw contributions,
cap/ceiling effects, distinct evidence counts and the model timestamp. Ranking
retains existing verdict groups, then final score descending, then stable job UUID.
An ineligible/null-score job never gains eligibility; owner-excluded and
application-tracked jobs receive no adjustment. Notification thresholds and AI
approval gates continue using the authoritative deterministic assessment.

No profile file, hard rule, scoring weight, availability or AI prompt is modified
by learning. There are no AI calls, background training or external exports.
Diagnostics contain private job IDs/features; export only to a private location.

## Configuration and migration

Apply `rust/migrations/0010_preferences.sql` with the existing migration command.
It extends feedback with structured attributes, actual effort and a contemporaneous
feature/ranking snapshot; appends learning-attribute revisions; adds immutable
`preference_models` and transactional `preference_state`. It adds Withdrawn/Other
to existing checked enums. Jobs, proposals, feedback and applications are retained.
Use a normal database backup before deployment; automated tests use an isolated
local database, never production Supabase.

`scoring.personalization.runtime` is optional and disabled by default. Copy the
section from the committed template into **private/config/scoring.yaml**, retaining
all existing private settings. The existing approval and global minimum settings
still apply. No new environment variables or credentials are needed.

Precedence remains approved DB namespace > deployed YAML > existing optional
compatibility behavior. A missing runtime section in an old approved namespace
may be supplied by deployment YAML, following earlier optional-section migrations;
existing populated DB runtime settings are never overwritten. Inspect `/v1/config`
provenance before editing. Use authenticated Config updates for an approved DB
namespace; editing a template does not override it. Enable/disable is itself an
audited scoring namespace revision. All controls require current revision numbers.

Settings include allowlists, feedback weights, prior/sample thresholds, bounds,
optional half-life and optional `training_window_days` (null means all history).
The event bound counts revision rows, so a heavily corrected history may require
an explicit narrower window. Comparisons/ranking load a bounded catalog; narrow
the job query if it exceeds the configured limit. Results are re-ranked before
pagination; Newest/Title sorts preserve their original behavior.

## Owner workflow and API

Start existing backend/TUI against the chosen PostgreSQL database, with private
configuration and owner credentials already exported:

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- migrate
cargo run --manifest-path rust/Cargo.toml -p radar-server -- serve --bind 127.0.0.1:8080
cargo run --manifest-path rust/Cargo.toml -p radar-tui -- --allow-http-localhost
```

Use `--config-dir private/config` on server commands if needed. All runtime files
are plain `radar.yaml`, `scoring.yaml`, `notifications.yaml`. See
[cloud setup](cloud-sync.md) for credential/environment and Supabase connection
configuration. Upwork and AI may remain disabled for the entire workflow.

| Authenticated API | Behavior |
|---|---|
| GET `/v1/jobs`, `/v1/jobs/{id}` | Base assessment plus separate optional personalization explanation |
| POST `/v1/jobs/{id}/feedback` | Existing feedback, optional typed `reason_code`, existing revision/idempotency rules |
| PUT `/v1/feedback/{id}/learning` | Correct reason, exclude/restore learning, report Completed actual hours |
| GET `/v1/preferences` | State/revision, last 30 model summaries, active model, suppression/rejection/staleness |
| POST `/v1/preferences/rebuild` | Idempotent candidate; optional historical cutoff, no activation |
| GET `/v1/preferences/models/{id}` | Full reproducible model/evidence diagnostic export; no raw prose |
| GET `/v1/preferences/models/{id}/compare` | Current-catalog baseline vs shadow ranking plus historical backtest |
| GET `/v1/preferences/calibration` | Counts and review-only suggestions |
| POST `/v1/preferences/control` | Explicit activation, rollback, enable/disable, suppress/unsuppress, reject, confirmed reset |

Rebuild body: `{"idempotency_key":"UUID","cutoff":null}`. Reuse the same key to
retry an uncertain request; a conflicting payload returns 409. Concurrent rebuilds
use a PostgreSQL advisory lock scoped to the application schema; a busy rebuild
returns 409. Future cutoffs are rejected.

Control body: `{"expected_revision":0,"action":"activate","model_id":"UUID","feature":null,"confirm_reset":false}`.
Use `rollback` with a prior model ID; `enable`/`disable` need no model ID.
`suppress`/`unsuppress` require a feature from the selected model. `reject` marks
that model unusable and clears activation if active. Reset requires
`"action":"reset","confirm_reset":true` and clears activation only; history,
manual preferences and suppression controls survive. A model is never auto-activated,
even when its measured metrics improve.

Correction body: `{"expected_version":1,"reason_code":"budget_too_low","excluded":true,"actual_effort_hours":null}`.
Corrections append a revision and invalidate all old model generations immediately;
rebuild and review before reactivation. Historical-cutoff models cannot be used
to bypass a later correction. Feature/settings changes also require a rebuild.
Rollback restores an old model under **current** jobs, owner decisions, feedback
correction epoch and suppression settings; it does not undo those user changes.
A stale generation cannot be rolled back into service.

Read-only tokens can read diagnostics but cannot submit/correct feedback, rebuild,
activate, reset, suppress, reject or change config. All writes use existing owner
middleware, conflict handling and audit records.

## Terminal controls

Existing screens 1–7 and Job Details AI controls remain. Screen **8 Learning** adds:

| Control in Learning | Action |
|---|---|
| ↑/↓ | Select model |
| R / v | Explicit rebuild / shadow comparison |
| a / b | Activate / roll back to selected model |
| e / d | Enable / disable personalization |
| [ / ] then s / u | Select feature, suppress / restore it |
| z / c | Reject model / view calibration |
| x | Reset dialog; type RESET and Enter to confirm |
| Y | Copy selected diagnostic summary via existing clipboard mechanism |

Use the full model GET endpoint for the training evidence export. Job Explorer
shows Base and Rank separately when enabled; Details/Evaluation show computed
contributions and confidence apart from Phase 5 AI output. Feedback entry uses
Ctrl+R to cycle reason codes and Ctrl+S to save. Feedback History **R** opens a
correction form: Ctrl+R reason, Ctrl+D exclusion, optional actual hours, Ctrl+S save.
Lowercase r refresh and existing retry/conflict behavior remain. Learning-specific
keys take their documented meaning only on that screen. Terminal rendering uses
existing responsive panels, resizing and control-character sanitization.

## Backtesting and calibration limits

A candidate's full fit is **not** evaluated on its own labels. Backtesting fits
a separate earlier chronological prefix, excludes all held-out job IDs, and
uses feedback/revision attributes available at the prefix cutoff. It reports
training/held-out sample counts, explicit-feedback-only P@K, rank-position
agreement and eligible labelled coverage. It does not measure the unlabelled
population, conversion probability, causal impact or guaranteed predictive quality.
Tiny datasets report insufficient evidence. Models preserve cutoff, window,
feature/config versions, evidence epoch and reproducible structured training inputs.

Calibration compares explicit choices to base scores, recorded personalized
rankings, prior completed AI evaluations, existing draft history and manually
reported outcomes. Effort comparisons use estimates completed before the first
reported application; actual hours must be explicitly recorded. Missing effort
estimates remain unknown. Suggestions cite counts and require human review;
they never rewrite a prompt/rubric or private facts.

There is no historical impression/selection-time log for pre-Phase-6 jobs.
Contemporaneous feedback snapshots approximate decision context. Historical
source inputs come from the last stored evaluation **before** the feedback;
later reimports are not substituted. Jobs lacking historical inputs/version
compatibility are counted and excluded. Missing labels, selection bias,
correlated features, broad positive labels and changing preferences limit conclusions.
No diversity optimization, statistical significance claims or learned auto-alerts
are provided. Re-evaluate/reimport jobs through the existing pipeline when hard
policy/profile changes make the stored deterministic evaluation stale; learning
never recalculates or overrides that authoritative assessment.

## Verification and repeatable synthetic demonstration

From `rust/` (use an isolated test database URL, not production Supabase):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RADAR_TEST_DATABASE_URL=postgres://USER@127.0.0.1:PORT/radar_test \
  cargo test --workspace -- --include-ignored
RADAR_TEST_DATABASE_URL=postgres://USER@127.0.0.1:PORT/radar_test \
  cargo test -p radar-server --test cloud preference_shadow -- --include-ignored --nocapture
```

The shadow test imports 13 synthetic manual jobs using real TCP API/SQLx, records
five positive and five budget-negative feedback events through the shared service,
reviews candidates, activates, rolls back and renders the terminal Learning screen.
`rust/fixtures/learning-feedback.json` describes the synthetic feedback scenario.
The outcome/effort test uses a fake Phase 5 provider, never a paid service.
No live AI, Upwork, Discord or production-data calls are needed.

| Demonstration | Observed assertion |
|---|---|
| A: no feedback | Exact baseline order with an empty active model |
| B: five Interested events | Base 85, bounded adjustment +8, final 93 on untouched matching job |
| C: five budget-coded Skips | Fresh matching-budget job demoted; technology weights get no negative evidence |
| D: hard-ineligible job | Verdict stays Skip, adjustment zero |
| E: rollback | Entire same-input ranking and scores restored; jobs/history retained |
| F: disabled | Exact baseline order, no personalization output, feedback retained |

An additional integration test starts/kills/restarts actual backend processes and
checks activated weights, exact ordering, pagination and feedback persistence.
See [acceptance](acceptance.md) for final executed results and test names.

## Phase 6 changed-file inventory

New:
- `rust/crates/radar-application/src/learning.rs`
- `rust/crates/radar-adapters/src/postgres/learning.rs`
- `rust/crates/radar-adapters/tests/learning.rs`
- `rust/apps/radar-server/src/learning.rs`
- `rust/migrations/0010_preferences.sql`
- `rust/fixtures/learning-feedback.json`
- `docs/radar/plan/phase6-learning.md`

Updated:
- `rust/crates/radar-domain/src/decisions.rs`
- `rust/crates/radar-application/src/lib.rs`
- `rust/crates/radar-application/src/api.rs`
- `rust/crates/radar-application/src/config.rs`
- `rust/crates/radar-adapters/src/postgres.rs`
- `rust/crates/radar-adapters/src/postgres/interactions.rs`
- `rust/crates/radar-adapters/tests/ai.rs` (shared DTO fixture)
- `rust/apps/radar-server/src/lib.rs`
- `rust/apps/radar-server/tests/cloud.rs`
- `rust/apps/radar-tui/src/client.rs`
- `rust/apps/radar-tui/src/ui.rs`
- `rust/apps/radar-tui/src/ui/view.rs`
- `config/scoring.example.yaml`
- `config/README.md`
- `README_RADAR.md`
- `docs/radar/README.md`
- `docs/radar/requirements/R04-personalization.md`
- `docs/radar/architecture/02-data.md`
- `docs/radar/plan/roadmap.md`
- `docs/radar/plan/acceptance.md`
- `docs/radar/plan/backlog.md`

This inventory describes Phase 6 only. Earlier phase files were already present
as untracked workspace additions; Git status alone does not distinguish them.
No Cargo dependency/lockfile, `.env`, private config/profile, Claude workflow,
Python utility or Upwork adapter changes belong to this phase.
