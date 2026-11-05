# R08 — Security, privacy, policy and cost controls

**Owner:** Platform / Security · **Priority:** Must

## Requirements

- **FR-070** Require authenticated TUI/API requests and HTTPS outside local development; apply least privilege and never expose database externally by default.
- **FR-071** Store Discord bot tokens, OAuth tokens, API keys and database passwords in secret/env configuration; exclude them and complete personal Markdown from Git.
- **FR-072** Restrict AI tool access, spending, execution time and concurrency. Treat all job text, client messages and retrieved pages as untrusted data, not instructions.
- **FR-073** Preserve provider ToS and official approval flows. Cloud Upwork MCP unattended auth requires a documented authorized proof-of-concept; otherwise disabled.
- **FR-074** Audit config edits, scan outcomes, proposal actions, notification delivery and authorization failures with redacted logs.
- **FR-075** Define retention/deletion controls for job content, profiles, messages and application history before production use.
- **FR-076** Separate AI development/runtime credentials from third-party OAuth credentials; never print secrets in TUI/logs.

## Business rules

- **BR-070** No credentials copied from browser sessions; no captcha or platform restriction bypass.
- **BR-071** A user's job ad or external website cannot override safety, budget, or tool-authorization instructions.
- **BR-072** No unauthorized or automated marketplace submissions; approval is explicit and contextual.

## Acceptance

- **AC-070** Repo secret scan finds no real credentials and private profile files are gitignored.
- **AC-071** Unauthorized bot interactions and API requests cannot mutate data.
- **AC-072** Simulated malicious prompt text inside a job description cannot run commands or alter policy settings.
- **AC-073** Missing Upwork cloud authorization disables only that provider, not local mock/manual pipeline.
