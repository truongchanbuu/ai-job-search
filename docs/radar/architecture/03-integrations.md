# Architecture — adapter contracts & external integrations

## Ports

| Port | Must support | First adapter |
|---|---|---|
| `JobSource` | capability check; search/import; normalized jobs; pagination/rate errors | `Mock`, `Manual`; Upwork MCP gated |
| `AiProvider` | structured evaluation/draft, timeout, usage, error reason | `Mock`; Codex gated |
| `NotificationProvider` | send event to destination; result/remote ID; retry-after | Discord |
| `InteractionProvider` | verify sender/request; map command/action to use case | Discord HTTP |
| `Clock` | time zone and fake clock for tests | Tokio/chrono |
| `Repository` | jobs, config, feedback, outbox, run locks | SQLx/PostgreSQL |

## Upwork MCP gate

1. Confirm official API/MCP terms permit the proposed unattended workload, not just the request quota.
2. Verify a supported OAuth flow and token refresh for a **headless cloud client** without copying Codex browser-session credentials.
3. Smoke-test a bounded search, read, pagination and proper `retry_after` handling.
4. Record capability and authorization results. Only then enable scheduled Upwork scans; fail closed if gate fails.
5. Marketplace writes remain **disabled** until separately implemented with the official confirmation workflow and human approval.

## Discord integration

- Prefer HTTP interactions to avoid a permanent Gateway socket for slash commands/components.
- Verify Discord signatures against the **raw request body**, validate sender allowlist, respond to PING.
- Quickly acknowledge/defer, then queue expensive work. Outbound embeds sent through bot REST API.
- If a provider supports only sending, skip the inbound interface for it.

## Codex / AI runtime

- Codex is the default development tool; cloud evaluation is an **adapter**, not a presupposition.
- Never depend on a human Codex session remaining open. Validate noninteractive auth, model quota, token usage, audit and permission boundaries in a POC.
- Rule-based scoring must work with no AI credentials.

## Configuration precedence

`DB-approved overrides > server YAML defaults > compiled defaults`. The API returns effective values **plus provenance and revision**; the TUI edits DB overrides. Local YAML is not silently overwritten. Personal Markdown versions/sync are a separate workflow.

## Phase 4 implemented boundary

The async Upwork gateway prepares a bounded normalized `UpworkJobSource` batch;
that implements the existing synchronous `JobSource` port inside the shared
import/scoring transaction. Manual source selection extends the existing run
ledger and outbox. Tokens/account references stay in Radar's encrypted private
state directory, separate from Codex and PostgreSQL. SDK Streamable HTTP/OAuth
is exercised against local fake servers; authenticated Upwork acceptance remains
pending. Search schemas are live-discovered and operator-bound, then revalidated
for every scan. No scheduled Upwork polling or marketplace writes are supported.
See [setup, evidence and limitations](../plan/upwork-mcp.md).


## Phase 5 implemented boundary

The existing application `AiProvider` port now supports async typed inference.
Adapters implement deterministic mock and tool-free Responses HTTP transport.
Manual owner API requests admit a quota-reserved PostgreSQL operation and launch
one bounded task. There is no automatic AI scheduler or marketplace write path.
Curated profile facts, input/output validation and evidence-labelled assembly
are shared application logic; PostgreSQL stores evidence revisions, AI history
and separate immutable proposal edits. Existing job storage/scoring is reused.
See [Phase 5 operation and limitations](../plan/phase5-ai.md).
