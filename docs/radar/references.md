# References and integration watchlist

Checked on **2026-10-30**. Re-check before implementing production integrations; remote documentation and terms can change.

1. [Upwork MCP Server](https://www.upwork.com/ai/mcp) — official OAuth connector, supported agent clients and confirmation workflow.
2. [Upwork MCP rate limits](https://support.upwork.com/hc/en-us/articles/56577997707923-Rate-limits-for-Upwork-MCP) — per-category limits, `retry_after`, and guidance **against frequent polling**. Do not hardcode published thresholds as permission to schedule unattended searches.
3. [Use bots and other automation properly](https://support.upwork.com/hc/en-us/articles/43342677368467-Use-bots-and-other-automation-properly) — unauthorized bot/browser collection risk.
4. [Official Upwork Agent Plugin](https://github.com/upwork/upwork-agent-plugin) — Codex-compatible skills, MCP integration and human approval.
5. [Discord interactions overview](https://discord.com/developers/docs/interactions/overview) and [receiving/responding](https://discord.com/developers/docs/interactions/receiving-and-responding) — HTTP endpoint, signature validation, PING and acknowledgments.
6. [Ratatui](https://ratatui.rs/) — native Rust TUI approach.
7. [Rust MCP SDK (rmcp)](https://github.com/modelcontextprotocol/rust-sdk) — candidate MCP transport; verify current SDK support before coding.
8. [Codex documentation](https://developers.openai.com/codex/) — noninteractive execution, auth and deployment model to verify during POC.

**Open integration risk:** a successful local Codex + Upwork MCP login does not establish legal/technical unattended OAuth operation from a separate cloud worker. Treat that as a project gate, with manual/mock sources as the initial fallback.
