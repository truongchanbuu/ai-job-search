# BRD — Personal Freelance Radar

**ID:** RADAR-BRD · **Version:** 0.1.0 · **Status:** Draft for implementation · **As of:** 2026-10-30

## 1. Business problem

A software engineer with full-time employment wants to discover occasional freelance work without repeatedly searching manually or spending Connects on poor-fit jobs. They need evidence-based job ranking, concise proposal drafts, configurable operating modes, and useful mobile notifications. The local computer may be off while approved cloud tasks continue running.

## 2. Business objectives

- Reduce time spent reviewing irrelevant jobs; **quality over number of applications**.
- Respect real weekly capacity and current work commitments.
- Keep job decisions explainable; record why the user liked, skipped or applied.
- Prepare truthful Upwork proposals from verified portfolio evidence.
- Access the same jobs/settings/feedback from a Rust terminal app and Discord/mobile.
- Avoid committing personal data or bot credentials to a public fork.

## 3. Stakeholders

| Actor | Responsibility |
|---|---|
| Owner/freelancer | Configures constraints, reviews job matches, approves proposals and marketplace writes |
| Cloud service | Runs permitted scheduled jobs, persists decisions, routes events |
| Terminal app | Authenticated desktop management interface |
| Bot adapter | Sends notifications; accepts authorized feedback/commands |
| Upwork / other authorized source | Supplies jobs via approved APIs / MCP where supported |
| Codex | Development tool and optional bounded AI runtime, **not** a mandatory always-on process |

## 4. Scope

**In scope:** profile/mindset Markdown; configurable work modes; permitted job acquisition; normalization/deduplication; explainable ranking; feedback and application tracking; proposal drafts; cloud scheduler; persistent storage; Ratatui TUI; Discord-first bot and future adapters; auditing and rate-limit handling.

**Out of scope (initial):** automatic proposal submission, purchasing Connects, unattended marketplace writes, browser-based Upwork scraping, impersonation, multiple Upwork accounts, a web dashboard, a public multi-user SaaS, advanced self-training/fine-tuning, trading any platform-compliance risk for coverage.

## 5. Success measures (initial targets, not promised outcomes)

- Every job shown has a source/time, normalized ID, score explanation and unknowns.
- No duplicate notifications for the same event and destination.
- TUI and bot feedback converge to the same database record.
- Paused mode produces no new job scans or noncritical job alerts.
- No Upwork submission without human confirmation; MVP sends **zero** marketplace writes.
- MVP works end-to-end with mock jobs and no external credentials.

## 6. Functional requirements

| Domain | Document | Priority |
|---|---|---|
| Discovery | [R01](requirements/R01-job-discovery.md) | Must |
| Evaluation | [R02](requirements/R02-job-evaluation.md) | Must |
| Schedule & work modes | [R03](requirements/R03-work-modes.md) | Must |
| Personalization | [R04](requirements/R04-personalization.md) | Must |
| Proposal drafting | [R05](requirements/R05-proposals.md) | Should (after core MVP) |
| Multi-bot | [R06](requirements/R06-multi-bot.md) | Must (Discord vertical slice) |
| TUI + sync | [R07](requirements/R07-tui-and-sync.md) | Must |
| Security | [R08](requirements/R08-security.md) | Must |

## 7. Non-functional requirements

- **Security:** authenticated HTTPS API, bot user allowlist, least-privilege permissions, secrets outside version control.
- **Reliability:** restart-safe schedules, idempotent jobs, persisted outbox, bounded retries, visible failures.
- **Performance:** UI remains responsive when network/AI tasks are running; long jobs are asynchronous.
- **Cost:** configurable daily LLM, request and notification caps; do not require always-on Codex locally.
- **Maintainability:** Rust Cargo workspace, small modules, docs by concern, config by concern, mock-first testing.
- **Privacy:** private personal Markdown and client/job data; explain remote AI processing and retention.

## 8. Dependencies & constraints

1. Actual unattended Upwork MCP authentication/authorization must be **proved** in an isolated test. Until then, cloud job acquisition uses manual/authorized/mock sources. Rate limits do not, by themselves, grant permission for unattended automation.
2. Any later marketplace write follows official preview/confirmation flows and explicit human approval.
3. Free/cheap hosting is a target, not a guarantee; database, hosting, Codex/API and notification costs must be tracked.
4. Existing upstream `.claude`, `.agents`, tests and templates remain functional; extension instead of rewrite.

## 9. Delivery milestones

See [roadmap](plan/roadmap.md). Acceptance is defined in [acceptance cases](plan/acceptance.md), not by a demo-only claim.

## 10. Open decisions

- Cloud host and secret-management option: **TBD**.
- Upwork MCP unattended auth: **Unknown; gated POC**.
- AI adapter (Codex CLI vs API) and cost model: **Unknown; prove before deployment**.
- Storage retention/deletion of job and proposal content: **Set defaults in implementation**.
