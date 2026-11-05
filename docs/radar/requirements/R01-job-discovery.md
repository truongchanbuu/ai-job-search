# R01 — Job discovery & ingestion

**Owner:** Job Source · **Priority:** Must · **Related:** R02, R03, R08

## Requirements

- **FR-001** Support `JobSource` adapters, beginning with `MockJobSource` and manually imported jobs.
- **FR-002** Integrate the official Upwork MCP only through authorized capabilities after an auth/terms feasibility check; cloud unattended scan is gated and must be disabled when unsupported.
- **FR-003** Normalize source job ID, URL, title, description, budget, type, skills, posted timestamp, retrieval timestamp and provider; allow null/unknown fields.
- **FR-004** Deduplicate by `(provider, provider_job_id)` with canonical URL fallback; never infer unseen details.
- **FR-005** Preserve source provenance and enough original text to audit a recommendation, subject to retention policy.
- **FR-006** Respect pagination, throttling, `retry_after`, daily budget and source errors; no busy-loop retries.
- **FR-007** Manual re-scan from TUI/bot is queued, authorization checked, and rate-limited like scheduled scans.

## Business rules

- **BR-001** No unauthorised Upwork DOM/browser scraping, credential extraction or session reuse.
- **BR-002** Search failure is a degraded state, **not** proof there are no suitable jobs.
- **BR-003** Previously submitted or excluded job IDs must not be recommended as new applications.

## Acceptance

- **AC-001** Reimporting the same job twice yields one canonical job and preserved fetch history.
- **AC-002** With provider disabled or auth invalid, the scheduler records the reason and does not attempt bypass/fallback scraping.
- **AC-003** A mock provider supplies a deterministic, testable job batch without network credentials.
