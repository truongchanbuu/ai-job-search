# Minimal, profile-grounded proposal templates

## Research, reviewed 2026-11-05

The goal is to help a client assess a specific solution quickly. “Optimized” here
means edited for relevance, brevity and truthful evidence. It does not mean a
verified increase in interviews, hires or earnings.

| Source | Observation and limitation | Design decision |
|---|---|---|
| [Upwork: proposal examples](https://www.upwork.com/resources/how-to-create-a-proposal-that-wins-jobs) | The indexed guidance emphasizes addressing the client's needs early, relevant examples and short paragraphs. Direct page access returned 403; only the search excerpt was available. | Open on the problem; keep a concrete approach and relevant proof. |
| [Upwork: cover-letter tips](https://www.upwork.com/resources/cover-letter-tips) | The indexed excerpt says opening lines appear before a client opens the full proposal. Direct retrieval returned 403. The visible amount can vary with interface. | Make the opening useful without relying on an exact character/line-count trick. |
| [Upwork Help: submitting a proposal](https://support.upwork.com/hc/en-us/articles/211062998-How-to-submit-a-proposal-on-Upwork) | Describes the cover letter, client questions and a suggested next step. This is operational guidance, not a controlled test of writing styles. | Keep screening answers separate and check all requested information before manual submission. |
| [Reddit: Does short (2–5 lines) work?](https://www.reddit.com/r/Upwork/comments/1so6bgw/does_short_25_lines_proposal_work_on_upwork/) | Replies disagree on a universal length; several emphasize understanding the problem, a first step and a reason to reply. Individual wins do not establish causation. | No padded minimum length or universal “winning” word count. |
| [Reddit: First 50 proposals](https://www.reddit.com/r/Upwork/comments/1jic50a/) | Search excerpts criticize generic self-focused templates and suggest focusing on the client's project; the full page was unavailable. | Use a reusable drafting brief, never a reusable candidate biography. |
| [Reddit: generic AI proposals](https://www.reddit.com/r/Upwork/comments/1q9wr64/you_are_not_getting_work_because_of_your_proposal/) | Some self-described clients dislike repetitive AI wording and prefer specific technical relevance; other comments dispute proposal quality as the sole cause of success. Identities/statistics are unverified. | Remove filler, retain job specificity, and avoid promises that wording alone fixes poor fit or competition. |
| [Reddit: Proposal thoughts as a client](https://www.reddit.com/r/Upwork/comments/1sxwqa6/proposal_thoughts_as_a_client/) | Search excerpt favors concise proposals and points to client-review guidance. An anecdotal perspective, not representative hiring data. | Prefer readable, compact paragraphs and one related example; review individual job requirements. |

No commenter claims, income, credentials or example wording are copied into the
owner's profile/proposals. Promotional posts claiming analysis of “winning
proposals,” guaranteed formulas, compulsory unpaid trials and boosting statistics
were not adopted. These findings inform editorial choices, not ranking weights
or inferred application outcomes. No Reddit ingestion/background search was added.

## Templates delivered

The editable [pattern library](../../../templates/radar-proposals/README.md)
contains five English patterns with explicit placeholders and usage notes:

- **problem_first:** problem → first action/check → relevant proof → easy reply.
- **focused_repair:** reported symptom → reproduction/focused fix/regression check.
- **integration:** actual flow → contract/mapping → relevant success/failure checks.
- **salesforce_change:** user workflow → smallest justified sandbox change.
- **clarify_scope:** known outcome → bounded first deliverable → consequential question.

The runtime briefs are in `radar.proposals.templates` in the effective config,
with owner-editable wording, word/evidence/question limits and a default ID.
Public Markdown patterns are for manual editing/review; the executable YAML brief
is the runtime source. Keep them aligned when intentionally changing a pattern.
Default ceilings of 120–140 words, one evidence reference and one question are
**our editorial starting points**, not externally proven optimal settings. Never
omit required technical details or screening answers merely to make a draft short.

## Personalization and safety

Generation uses the selected stored job, approved/current Phase 5 evaluation and
the active private profile from Profiles. Experience facts must be verified and
shareable. A deterministic relevance ordering compares word tokens from actual
job skills/title/category to evidence IDs/statements; ties use the evidence ID.
The server permits only the top configured evidence references. Missing matches
stop the templated request **before** reserving a paid provider call. This is a
conservative lexical check, not proof of delivery ability or semantic equivalence.
Transferable experience may require human review and a more accurate, truthful
profile summary; never add keywords that misrepresent the actual work.

Prose is customized against the actual job and writing-style instructions. The
server assembles candidate claims verbatim with their real context and approved
optional portfolio link. A training/academic project never becomes paid client
work. Generic openings, unresolved `{{...}}` slots in plan prose, duplicate or
irrelevant evidence, excess questions and oversized copy are rejected. Existing
claim/URL/numeric-promise checks still apply. The final body uses a clarification
question **or** the supplied next step; it does not append a duplicate closing
CTA. Assumptions/warnings and screening answers stay outside copy-ready text.

The title/job description remains untrusted data. Neither a template nor profile
text authorizes tools, secret access, marketplace actions or manufactured facts.
Editing/review and approval remain manual. A successful mock draft proves data
flow/validation, not actual model quality or proposal effectiveness.

## Configuration, API and TUI

Copy/merge `proposals.templates` from `config/radar.example.yaml` into private
`radar.yaml`, preserving your settings. Inspect `/v1/config` first: approved DB
namespaces retain precedence. Missing optional template catalogs may reconcile
from deployed YAML following existing optional-section behavior; an existing
populated catalog is preserved. `default_id: null` keeps the existing drafting
behavior until a template is selected. Set it to an actual item ID only when you
want that default. No secret variables or database migrations are added.

Job Details/Evaluation **T** cycles the configured templates and returns to the
configured default/legacy choice after the last item. The proposal view displays
the selected name, purpose and word ceiling. **G** explicitly generates using
that choice after a current evaluation is approved. Existing **E** edit and **Y**
copy-reviewed-proposal controls remain. Switching a template makes no AI call.

The existing generation API accepts optional `template_id`:

```json
{
  "idempotency_key": "NEW-UUID",
  "expected_revision": 0,
  "evaluation_id": "APPROVED-EVALUATION-UUID",
  "template_id": "focused_repair"
}
```

Use POST `/v1/jobs/{id}/ai/proposals`; GET `/v1/config` exposes the catalog.
Templates are rejected on Evaluate requests. Missing IDs/catalogs return a
visible validation error, never a different template silently. Read-only tokens
cannot generate or change settings. No template still means no new constraints
unless an owner configured a default.

Each AI operation stores the complete selected template definition and its
content hash in its existing snapshot, plus existing profile/config/prompt
revisions. GET `/v1/jobs/{id}/ai` exposes that frozen selection. Later edits to a
catalog cannot rewrite old operations. Prompt contract version is `radar-ai-2`;
old snapshots remain readable. Regeneration appends a normal proposal revision,
retains user edits and does not mark an application submitted or approve new text.

## Verification and limitations

Tests cover all five brief shapes with synthetic profile/job data, relevant
fact selection and stable tie-breaking, unknown/duplicate IDs, revision hashes,
filler/placeholder/evidence/question/word constraints, owner/read-only admission,
TUI selection, operation persistence/idempotency and preserved edits across
regeneration. Existing Phase 1–6/Profile behavior runs in the full suite.

No live AI, real proposal submission or private-profile publication is needed.
The mock provider uses a job-title-based opening and the selected approach hint;
it is a transport/assembly fixture, not an example of high-quality live reasoning.
Real provider phrasing, multilingual copy and conversion benefit remain unverified.
The existing conservative Phase 5 plan validator can reject valid first-person
or numeric wording; review/edit manually rather than weaken truthfulness checks.

The five patterns are **not** automatically selected by topic, nor learned from
views/no replies. Use actual explicit feedback and manually reported outcomes
when comparing drafts; small samples and job/client differences prevent causal
claims. Template CRUD has no dedicated TUI screen; private YAML or existing owner
configuration API edits are available. No changes to private settings/profile
facts or production Supabase are made by this implementation.

Changed files: new `radar-application/src/proposal_templates.rs`, the existing
application AI/config exports, adapter AI/SQLx configuration and AI admission,
TUI Job Details/view, API integration tests and AI tests; `config/radar.example.yaml`,
`templates/radar-proposals/*.md`, public `proposal-style.md`, proposal standard,
this guide, README/config/docs indexes and acceptance records. No new dependency,
SQL migration, Upwork adapter, marketplace write or existing Claude workflow change.


Verified results (2026-11-05): **99 Rust tests passed**, including **31 isolated
PostgreSQL integration tests**; default credential-free run passed 68 and skipped
31. Formatting, all-target/all-feature Clippy, nine Python tests, repository guards
and local Markdown link checks passed. See [acceptance](acceptance.md).

Exact implementation inventory (relative to repository root):

- `rust/crates/radar-application/src/proposal_templates.rs` (new)
- `rust/crates/radar-application/src/lib.rs`
- `rust/crates/radar-application/src/config.rs`
- `rust/crates/radar-application/src/ai.rs`
- `rust/crates/radar-adapters/src/ai.rs`
- `rust/crates/radar-adapters/src/postgres.rs`
- `rust/crates/radar-adapters/src/postgres/ai.rs`
- `rust/crates/radar-adapters/tests/ai.rs`
- `rust/apps/radar-server/tests/cloud.rs`
- `rust/apps/radar-tui/src/ui.rs`
- `rust/apps/radar-tui/src/ui/view.rs`
- `config/radar.example.yaml`
- `config/README.md`
- `templates/radar-proposals/README.md` (new)
- `templates/radar-proposals/problem-first.md` (new)
- `templates/radar-proposals/focused-repair.md` (new)
- `templates/radar-proposals/integration.md` (new)
- `templates/radar-proposals/salesforce-change.md` (new)
- `templates/radar-proposals/clarify-scope.md` (new)
- `templates/radar-profile/proposal-style.md`
- `docs/radar/standards/proposals.md`
- `docs/radar/plan/proposal-templates.md` (new)
- `docs/radar/plan/acceptance.md`
- `docs/radar/README.md`
- `README_RADAR.md`
