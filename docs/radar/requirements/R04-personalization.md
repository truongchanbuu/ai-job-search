# R04 — Personal profile, portfolio and learning

**Owner:** Profile / Feedback · **Priority:** Must

## Requirements

- **FR-030** Read modular private Markdown: `identity`, `mindset`, `availability`, `job-preferences`, `portfolio`, `proposal-style`, `lessons-learned`.
- **FR-031** Separate `verified facts` (experience, dates, skills, client work) from `preferences` (what to target/avoid), `availability`, and `inferences` (model suggestions).
- **FR-032** Track per-job actions `interested`, `skip`, `save`, `not_available`, `applied`, `interview`, `hired`, `rejected`, `completed`, `withdrawn`, `other` with optional structured reason and explanation.
- **FR-033** Suggest changes in weights/preferences based on feedback; show evidence and require approval before promoting changes to effective config.
- **FR-034** Keep history/versioning of changes; allow rollback and distinguish outcome feedback from job-preference feedback.
- **FR-035** Use only explicit owner feedback/valid manually reported outcomes for bounded, explainable ranking above unchanged deterministic eligibility. Preserve cold-start/disabled ordering.
- **FR-036** Support corrected/excluded learning evidence, immutable feature/model versions, manual rebuild, shadow comparison, chronological holdout and explicit activation/reset/rollback.
- **FR-037** Compare base/AI/learned assessments with explicit choices and reported outcomes; produce counted review suggestions without changing facts, hard rules or prompts.

## Business rules

- **BR-030** AI may propose but never fabricate or auto-approve profile facts, certificates, metrics or paid client work.
- **BR-031** Feedback on one job is a weak signal; never silently rewrite hard filters after one rating.
- **BR-032** Private profile content is never automatically committed to the public GitHub fork.

## Acceptance

- **AC-030** A new private `mindset.md` changes explanation style/preferences without altering verified years of experience.
- **AC-031** Skip feedback with reason `schedule_conflict` does not permanently lower interest in the technology.
- **AC-032** A suggested weight update is visible, reversible and inactive until approved.

- **AC-033** No feedback gives baseline order; reason-specific negatives cannot penalize unrelated technologies; hard-ineligible jobs remain ineligible.
- **AC-034** Corrections invalidate affected model generations; historical cutoffs cannot bypass them. Rebuild, activation, rollback and confirmed reset preserve jobs/drafts/history.
- **AC-035** Backtests exclude future labels/revisions and held-out job IDs from fitting; diagnostics report sample limits rather than predictive accuracy.
- **AC-036** Read-only credentials cannot learn or mutate preferences. Activated weights/order survive backend process restart.

Implementation and executable evidence: [Phase 6 guide](../plan/phase6-learning.md)
and [acceptance](../plan/acceptance.md). Learning attributes can be corrected or
excluded; authoritative application outcomes retain the existing audited lifecycle.

## Profiles setup extension

- **FR-038** Provide labelled private profile forms for CV text, skills, experience context, portfolio evidence, preferences and writing style, with preview and explicit owner confirmation before saving.
- **FR-039** Reuse immutable profile snapshots; an approved DB profile overrides deployment Markdown for cloud operations without rewriting private source files. Preserve historical scores and mark incompatible AI assessments stale.
- **AC-037** Owner can add/edit skills/evidence, preview sharing, save and recover a profile version. Invalid/stale/read-only writes cannot change the profile; saved content survives actual backend restart.

See [Profiles operation and verification](../plan/profiles.md).
