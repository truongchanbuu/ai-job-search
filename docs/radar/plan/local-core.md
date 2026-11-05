# Phase 0–1 — local core

This guide describes the original Phase 1 local mode. Phase 2 is documented in
[cloud sync](cloud-sync.md); its API and terminal client are now implemented.

The local-mode entry point is `radar-server demo`. It does not start a server.
All runtime application logic is Rust. The five workspace packages follow the
system boundaries. `radar-tui` is now the HTTP client described in the Phase 2 guide.

## Repository audit and scope

The inherited application is a Claude Code workflow with Python salary tools,
Bun portal CLIs and LaTeX templates. There was no Cargo workspace. The starter
requirements, config, profiles, prompt and ZIP were already untracked inputs.
Existing README.md, CLAUDE.md, .claude/, .agents/, Python/TS source, templates
outside radar-profile/ and the original CI workflow are preserved. Existing
workflow concepts (separate facts/preferences and truthful evidence) inform the
new core; the runtime does not execute those workflows or reuse their scrapers.

## Run from the repository root

```sh
mkdir -p private/config
for namespace in radar scoring notifications; do
  cp -n "config/$namespace.example.yaml" "private/config/$namespace.yaml"
done
cargo run --manifest-path rust/Cargo.toml -p radar-server -- demo --at 2026-10-30T12:00:00Z
```

This loads private YAML configuration initialized from the public templates and explicitly synthetic Markdown fixtures.
Seven mock records produce six canonical jobs, including ranked candidates,
unknown-data reviews, a capacity veto and a policy veto. No credentials are
needed. No HTTP requests, marketplace writes or external notifications occur.

For an editable private synthetic example and a reproducible export:

```sh
mkdir -p private/demo-profiles private/config private/radar
cp rust/fixtures/profiles/*.md private/demo-profiles/
mkdir -p private/config
for namespace in radar scoring notifications; do
  cp -n "config/$namespace.example.yaml" "private/config/$namespace.yaml"
done
cargo run --manifest-path rust/Cargo.toml -p radar-server -- demo \
  --config-dir private/config --profile-dir private/demo-profiles \
  --at 2026-10-30T12:00:00Z --output private/radar/demo.json
```

Exports refuse to overwrite existing files; choose a fresh filename on another
run. On Unix, files are created with mode 0600. Repository exports must live
under `private/` or `radar-output/`; exports outside the repository are allowed.
Exports include config/profile snapshots, fetched source text, evaluations and
repository history. Treat them as private. There is no automatic retention or
restart recovery: delete exports when no longer needed.

Manual import uses the same application service and scorer:

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- demo \
  --manual rust/fixtures/manual-jobs.json
```

Use that JSON as the import schema; provenance must be `manual`. Keep original
source URLs and only import data you are authorized to use. Nullable fields stay
unknown. Every ingestion receives the injected clock's retrieval time. Limits
bound each import; oversized files are not automatically paginated or resumed.

## Configuration and profiles

`--config-dir` selects a complete set of `radar.yaml`, `scoring.yaml`, and
`notifications.yaml`. Templates ending in `.example.yaml` are never loaded.
Directory precedence is explicit flag > `RADAR_CONFIG_DIR` > repository
`private/config`. Numeric operational defaults exist only
in YAML; required fields do not silently fall back to duplicated code values.
All nested objects reject unknown keys. Validation checks schema versions,
weights, zones, schedules, quotas, references and safety gates. Errors identify
the file/location without echoing YAML values. `${NAME}` and `*_env` fields are
references only; this phase never resolves provider secrets.

Each load produces a content revision plus namespace provenance. The designed
precedence remains approved DB overrides > YAML > compiled defaults. Phase 1
implements the YAML layer only; there is no fake live override store. Immutable
snapshots accompany exports so later config edits cannot reinterpret old scores.
Cloud/provider/notification settings are validated but are not running services.

Seven Markdown modules load independently. `identity.md` and
`job-preferences.md` require typed YAML front matter; see synthetic fixtures.
The public templates deliberately have incomplete metadata and fail validation
until the owner supplies verified skills/proof/context and a confirmed rate.
The other five modules remain versioned prose, never executable instructions or
sources of inferred verified facts. Numeric capacity comes from the selected
YAML mode; availability prose does not silently override it. Automated prose
interpretation, portfolio matching, style adaptation and feedback learning are
future work.

## Deterministic rubric

Eligibility runs first: prohibited work, capacity, overlap while employed/busy,
known impossible deadline, applied/excluded state, blocked category and unsupported
required skill. A veto has no numeric score. Every listed skill is interpreted
as required; optional skill parsing is not implemented.

For eligible jobs, seven weighted signals use the scoring YAML weights:

- Technical/evidence: all required skills must have verified profile proof.
  Evidence context is retained; a personal project is never relabeled client work.
- Scope and client signals use explicit imported facts, with true/false endpoints.
- Budget uses hourly USD or fixed USD divided by explicitly estimated effort,
  relative to the owner's confirmed minimum. No FX conversion or fee estimation.
- Availability tests required weekly hours against selected mode capacity.
- Interest uses the owner's preferred categories; other eligible categories are neutral.

Unknown signals retain null inputs and use a neutral midpoint for aggregation;
unknown flags lower confidence. Missing full description always requires review.
The algorithm's scale/midpoint and confidence categories are fixed Phase 1 rules,
not learned preferences. Rates, weights, thresholds, capacities and limits remain
editable. A candidate verdict is a review aid and never permission to apply.

The deadline veto catches past deadlines and estimates that cannot fit even in
remaining wall-clock time. Working-window feasibility and calendar planning are
not inferred. Required hours must be supplied explicitly to assess capacity.

## Verification and limits

From `rust/`:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

Tests cover config rejection/revisions, profile separation, deterministic scores,
hard vetoes, missing data, ID/URL deduplication with history, fake local-day time,
paused/resumed scans, caps, source failure, manual imports, mock delivery
idempotency, and CLI exports/errors. The separate Radar CI runs without secrets.
Legacy checks: `python3 -m unittest discover`, `python3 tools/lint_skills.py`,
`python3 tools/security_guards.py`. Bun/LaTeX checks require those tools installed;
this change does not alter their source or existing CI checks.

The repository and quotas are process-local. Serialization is tested, but locks,
crash recovery, cloud persistence, approved override editing, routing/quiet hours,
notification caps/outbox and an actual scheduler are deferred. Mock notification
idempotency is an adapter contract test, not proof of durable Discord delivery.
The AI adapter emits a labeled synthetic response, not a real proposal. Provider
failures return errors without retries or fallback scraping. Upwork cannot be
enabled; real AI and marketplace writes fail configuration validation.

Phase 3 now supplies durable scheduling/outbox and Discord adapters; the limits
above describe the original local-only adapter. See [background operation](background.md).
