# R05 — Proposal drafting & application tracking

**Owner:** Proposal workflow · **Priority:** Should; no marketplace writes in MVP

## Requirements

- **FR-040** Generate a copy-ready short proposal from verified profile evidence and the client's actual job description.
- **FR-041** Pick at most a few relevant portfolio projects, separating real employment from training, academic and personal projects.
- **FR-042** Provide a separate set of screening-question answers, assumptions, gaps and recommended client clarification questions.
- **FR-043** Allow review/edit/version history, status and follow-up notes; do not mutate Upwork via browser automation.
- **FR-044** In a later authorized integration, respect official Upwork MCP preview and explicit confirmation before any submission, messages, bids or profile updates.
- **FR-045** Track Connects spent only from verified input/source; no guessed bid or submitted status.

## Business rules

- **BR-040** Every claimed skill/achievement maps to a verifiable profile/portfolio source.
- **BR-041** Never promise price or delivery time without user-confirmed constraints.
- **BR-042** `Drafted` is not `Submitted`; only human confirmation or a verified platform result changes status to `Submitted`.

## Acceptance

- **AC-040** A proposal never describes the SmartReturn training project as a paid client engagement.
- **AC-041** An unverified metric is omitted or explicitly marked for user confirmation.
- **AC-042** Calling the MVP proposal workflow creates a local draft without platform writes.
