# Runtime configuration

These `.example.yaml` files are **committable examples**, not live cloud settings. Copy them into `private/config/` (excluded from Git). Edit values there or use the authenticated TUI/API.

| File | Owns |
|---|---|
| `radar.example.yaml` | Work modes, schedule, quotas, providers, AI runtime limits |
| `scoring.example.yaml` | Score weights, hard vetoes, thresholds, optional preference-learning settings |
| `notifications.example.yaml` | Provider instances, destinations, routes, quiet hours |

From the repository root, initialize missing private files without overwriting existing settings:

```sh
mkdir -p private/config
for namespace in radar scoring notifications; do
  cp -n "config/$namespace.example.yaml" "private/config/$namespace.yaml"
done
```

For existing private copies named `*.example.yaml`, rename them to `*.yaml`,
preserving your values; do not replace them with fresh templates. If both names
exist, only the plain filename is loaded. All three runtime files are required.
Missing or invalid files cause a visible startup error.

Directory selection for native server commands (including demo and Upwork):
`--config-dir` > `RADAR_CONFIG_DIR` > repository `private/config`.
Relative explicit paths are resolved from the current working directory.
Docker sets `RADAR_CONFIG_DIR=/app/config`; its build copies public templates
under the three runtime filenames. Mount private files using those names.

**Priority:** approved DB override > deployed YAML default > compiled default. The TUI changes audited DB overrides; it does **not** silently edit the YAML on your laptop. Provide a config export/reconciliation workflow before production use.

Use `${VARIABLE_NAME}` placeholders for environment secrets and identifiers. No real keys, webhook tokens or Discord IDs in tracked files. Validate on startup and reject unknown keys/invalid values rather than ignoring typos.

Phase 1 loads a complete namespace set from the directory passed through
`--config-dir`: `radar.yaml`, `scoring.yaml`, and `notifications.yaml`.
Templates are never loaded or used as a fallback.
It returns revision/provenance and does not implement DB overrides yet. Required
keys are mandatory; YAML is the sole source of numeric runtime defaults.
Cloud and notification config is validated but not executed in this phase.
See the [local core guide](../docs/radar/plan/local-core.md).

Phase 2 exposes effective values and revisions through `/v1/config`. Owner-approved
namespace updates persist in PostgreSQL, override deployed YAML, and are audited.
The TUI never writes deployment YAML. See the [cloud-sync guide](../docs/radar/plan/cloud-sync.md)
for conflict handling and startup reconciliation.

Phase 3 executes the configured mock scan slots and notification routes.
`scheduler` owns worker cadence, recovery/catch-up, per-run/queue bounds and TUI
refresh. `notifications.runtime` owns delivery/ACK deadlines, backoff, signature
windows and the encryption-key environment reference. The second Discord example
is disabled until explicitly configured and routed. No unused provider adapter
is created. New optional scheduler/runtime sections in older approved DB
namespaces are reconciled from deployment YAML; other DB values retain priority.
Operational edits take effect each worker tick. Environment-secret rotation
requires process restart. `.env.example` contains variable names only. See
[background operation](../docs/radar/plan/background.md) and
[Discord setup](../docs/radar/plan/discord-setup.md).

Phase 4 adds `job_sources.upwork_mcp.runtime` with private state/key environment
references, optional public-client identity, callback/time budgets, pagination
bounds and named manual search profiles. Source enablement is explicit; all
scheduled work remains mock-only. Missing optional budget/type/experience
constraints mean no such local filter; configured constraints reject missing
facts. Upstream parameters and result-field bindings come from live discovery,
not example guesses. See [Upwork operation](../docs/radar/plan/upwork-mcp.md).


Phase 5 adds optional `ai.runtime`: missing/disabled means no inference. The
`mock` and tool-free `openai_responses` providers share the manual evaluation /
proposal workflow. Quotas, model, rubric, bounds, pricing metadata and proposal
length live in YAML/approved DB config, and keys remain environment references.
See [Phase 5 setup, evidence and privacy](../docs/radar/plan/phase5-ai.md).


Phase 6 adds optional `scoring.personalization.runtime`, disabled by default.
It owns allowlisted features, signal weights, sample/shrinkage thresholds, bounds,
optional time decay/training window and calibration thresholds. Learning never
changes hard rules or private Markdown. Approved DB settings retain precedence;
new optional sections reconcile only when missing. Enabling requires an explicit
owner action, and every model activation/rollback remains explicit. No new
credentials or AI provider are needed. See the [learning guide](../docs/radar/plan/phase6-learning.md).

Optional `radar.proposals.templates` holds editable drafting briefs and bounds. Its null default preserves the legacy path; explicitly choose a template in Job Details (T), or approve a default through configuration. Existing populated DB catalogs retain precedence. See [research and setup](../docs/radar/plan/proposal-templates.md).
