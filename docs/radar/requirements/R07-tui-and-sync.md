# R07 — Ratatui terminal application and cloud sync

**Owner:** TUI / API · **Priority:** Must

## Requirements

- **FR-060** Rust Ratatui desktop app can connect to a cloud Axum API over authenticated HTTPS.
- **FR-061** Main screens: Dashboard, Jobs, Job Detail, Feedback, Profile, Config, Runs/Logs; proposal list follows in next phase.
- **FR-062** Jobs support search/filter/sort/pagination and show score breakdown, risk, source URL, updated time and unknowns.
- **FR-063** Support keyboard-first interaction and an accessible non-colour-only state indication; remain responsive during network requests.
- **FR-064** User actions update server state, then refresh UI; show offline/network errors explicitly rather than pretending a change succeeded.
- **FR-065** Effective config read/write is versioned (optimistic concurrency) and audited; local files are not silently overwritten by cloud state.

## Acceptance

- **AC-060** With mock cloud data, TUI can list jobs and display evidence for a selected score.
- **AC-061** Changing an allowed setting via TUI is reflected by subsequent API reads, without changing a public example YAML.
- **AC-062** Disconnecting the backend produces a visible recoverable error and no false success message.
