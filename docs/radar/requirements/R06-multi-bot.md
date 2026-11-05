# R06 — Multi-bot notifications and interactions

**Owner:** Notification router · **Priority:** Must (Discord first)

## Requirements

- **FR-050** Use `NotificationProvider` and `InteractionProvider` ports; one event can be routed to multiple destinations/providers.
- **FR-051** Implement Discord first: useful embeds, `Interested`, `Save`, `Skip`, feedback modal and `/status`, `/jobs`, `/pause`, `/resume`, `/scan` commands.
- **FR-052** Support multiple Discord destinations and future Telegram, email or generic webhook adapters **without changing domain logic**.
- **FR-053** Route by event type, severity, threshold, owner, quiet hours and cap; log delivery outcome per destination.
- **FR-054** Persist an outbox and retry only safe failures with backoff and platform rate-limit instructions; idempotency key `(event_id, destination)`.
- **FR-055** Send job feedback and commands through shared cloud services and persistence; bot must not contain independent application state.

## Business rules

- **BR-050** Discord HTTP interactions must verify Ed25519 signature on raw request body + timestamp, accept valid PING, and acknowledge/defer promptly.
- **BR-051** Require configured Discord account/guild/channel allowlists before accepting sensitive commands; never trust only a button payload.
- **BR-052** Bot commands can trigger permitted scans or draft preparation, but never submit marketplace proposals automatically.
- **BR-053** A plain webhook can be send-only; it does not have to implement inbound interaction features.

## Acceptance

- **AC-050** Same `JobMatched` event delivered twice creates only one Discord delivery to each destination.
- **AC-051** Invalid Discord signatures return unauthorized and do not change job feedback.
- **AC-052** Clicking `Skip` results in the same state as skipping from TUI.
