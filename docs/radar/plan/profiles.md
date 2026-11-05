# Profiles — private CV and verified evidence setup

Profiles adds an authenticated editor on top of the existing seven Markdown
modules and Phase 5 evidence contracts. It manages one owner's main profile;
proposal templates and multiple profile variants are separate follow-up features.
No AI call is made to edit, validate or preview a profile.

## Terminal workflow

Start the existing backend and TUI with your private configuration/environment.
Apply embedded migration `0011_profile_setup.sql` using `radar-server migrate`
(or normal server startup). Production Supabase is not used by automated tests.

Open **9 Profiles**. The dark, two-column view separates profile sections from
records and CV text. Existing screens 1–8 retain their navigation.

| Control | Action |
|---|---|
| [ / ] | Select section: CV/skills, mindset, availability, job preferences, portfolio evidence, writing style, lessons |
| ↑ / ↓ | Select skill/evidence record |
| n / e / Enter | Add / edit a record; preferences use labelled category/rate fields |
| p | Edit the section's CV/prose text while preserving structured metadata |
| Delete | Confirm removal from the local draft; the last verified skill is protected |
| v | Validate draft on the server and preview verified evidence and sharing flags |
| s | Review confirmation, then save a new active version |
| d | Discard local changes and reload the last fetched server profile |
| PgUp / PgDn | Scroll section content |

Forms use Tab / Shift+Tab to move fields, Space to cycle experience context or
toggle verified/shareable/confirmation, Ctrl+U to clear a field, and Ctrl+S to
accept. Text supports bracketed paste, Backspace and multiline CV entry. Escape
cancels the open form. Edits remain local until the separate save confirmation.
A periodic refresh preserves unsaved edits and never replaces an open form.
On a stale revision, refresh and explicitly discard/reconcile the draft before
saving again. Successful writes are acknowledged only after the API confirms them.

Skills require a name, actual proof and experience context. Add employment,
contract, academic, training and personal portfolio facts with separate Verified
and Shareable switches. New facts default to false for both. A public proof URL
is optional and must satisfy the existing HTTPS evidence checks. IDs are unique;
source references identify an identity/portfolio Markdown section. The editor
preserves surrounding Markdown and never generates facts.

CV/prose can describe professional summary, roles/dates, education, languages,
certifications and preferences. Prose alone does not create verified claims.
Add explicit portfolio evidence for facts the AI may cite. Skill proof prose is
retained privately and excluded from AI prompts, following the Phase 5 model.
Only verified evidence enters evaluation; only verified, shareable evidence may
be rendered into proposals. Preview is data preparation, not an AI request.

Availability prose describes commitments and preferred windows. **Effective
numeric capacity/mode remains in Configuration (screen 5)**; editing descriptive
CV text never silently changes scheduler quotas or hard constraints. Proposal
writing-style text is sent through the existing curated Phase 5 evidence path;
keep it free of confidential information.

## Persistence and active-profile precedence

Existing `profile_revisions` stores immutable snapshots. Migration 0011 adds only
`owner_profile`, an active revision pointer plus optimistic concurrency revision.
First save retains the deployment Markdown snapshot for recovery. Explicit saves
are audited with revision identifiers, never full private text.

Approved DB profile > deployment Markdown fallback. Server restarts do not
replace an approved DB profile with filesystem content. Local Markdown files
remain untouched. The deployment profile must still be present/valid at startup;
this feature does not remove the existing startup requirement. Native local demo
commands continue to use their explicitly loaded Markdown files.

Cloud imports, scheduled mock/manual Upwork scans and manual AI operations read
the active DB profile through the shared repository. Historical job scores are
preserved: reimport/re-evaluate with the existing core pipeline to apply new facts
or preferences. Existing AI assessments/drafts whose profile revision differs
become stale under the existing Phase 5 review rules. Feedback, application state,
learning models and proposals are not deleted or rewritten by a profile save.

## API

Existing bearer owner/read-only authorization and HTTPS requirements apply.
Read-only credentials may read private profiles but cannot preview/save them;
issue such credentials only to trusted readers. Responses use the existing
no-cache policy. Profile data belongs in a protected database and backups.

- GET `/v1/profile`: active profile, source, concurrency revision, curated evidence
  and latest 30 immutable version summaries.
- POST `/v1/profile/preview`: validate and return curated evidence without saving.
- PUT `/v1/profile`: validate, confirm and persist/activate the full module set.
- GET `/v1/profile/versions/{hash}`: retrieve a stored snapshot for private export
  or recovery. Restore through a normal confirmed PUT with current revision and
  that snapshot's documents; history remains intact. History restore has no TUI
  shortcut yet.

Preview/save payload:

```json
{
  "expected_revision": 0,
  "documents": {
    "identity": "full private identity.md including structured skills",
    "mindset": "private text",
    "availability": "private text",
    "job-preferences": "full private module including preference metadata",
    "portfolio": "full private module including evidence metadata",
    "proposal-style": "private text",
    "lessons-learned": "private text"
  },
  "confirmed_facts": true
}
```

Use actual valid module contents, not the illustrative strings above. Missing or
unknown module names, invalid metadata, control characters, duplicate verified
evidence IDs, invalid rates/verified-evidence URLs and oversized modules are rejected. Each module
is bounded at 32 KiB; requests retain the existing global HTTP body bound. Values
are redacted from validation errors. No client path is used for filesystem I/O.
Preview ignores confirmation; save requires explicit `confirmed_facts: true`.
A stale write returns 409 and leaves the current profile intact.

## Verification and limitations

Tests exercise add/edit forms, default sharing flags, prose preservation,
confirmation, invalid rates, bounded Unicode paste, authenticated API validation,
read-only denial, stale saves, evidence preview, active-profile imports and AI,
terminal rendering/resizing, immutable recovery and actual process restart.
All fixtures are synthetic, with Upwork and paid AI services unnecessary.

The current editor is keyboard driven and uses basic text entry/paste rather than
a full document editor. PDF/DOCX CV extraction, rich formatting, profile variants,
mouse forms, proposal template management and automatic fact verification are
not included. Owner confirmation attests a fact; it does not establish external
verification. Never put credentials or confidential employer/customer material
into CV text, evidence summaries or writing instructions.

Changed files for this feature:

- `rust/crates/radar-application/src/profiles.rs` (new), `src/lib.rs`
- `rust/crates/radar-adapters/src/lib.rs`, `src/postgres.rs`, `src/postgres/profiles.rs` (new)
- `rust/migrations/0011_profile_setup.sql` (new)
- `rust/apps/radar-server/src/profiles.rs` (new), `src/lib.rs`, `src/ai.rs`, `src/workers.rs`, `tests/cloud.rs`
- `rust/apps/radar-tui/Cargo.toml`, `src/client.rs`, `src/main.rs`, `src/ui.rs`, `src/ui/view.rs`, `src/ui/profile.rs` (new)
- `rust/Cargo.lock` (TUI reuses the workspace's existing serde_yaml dependency)
- `docs/radar/plan/profiles.md` (new), `docs/radar/README.md`, `docs/radar/requirements/R04-personalization.md`, `docs/radar/plan/acceptance.md`, `README_RADAR.md`
