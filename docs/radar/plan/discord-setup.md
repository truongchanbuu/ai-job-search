# Discord HTTP interactions setup

This is a manual deployment checklist. Automated acceptance uses fake providers
and a local Discord HTTP stub. Real Discord registration, public endpoint
validation and delivery have **not** been performed in this environment.

## Application, permissions and private credentials

Create an application in the [Discord Developer Portal](https://discord.com/developers/applications).
In General Information, copy the Application ID and Public Key. In Bot, generate
a bot token and keep it in your private environment/secret manager. Use a private
test guild and channel. Configure Guild Install with `bot` and
`applications.commands`; grant View Channel, Send Messages and Embed Links only.
No Gateway connection or privileged intents are used. See Discord's
[application setup guide](https://docs.discord.com/developers/quick-start/getting-started).

Copy the three public YAML examples into an unused private config directory as
`radar.yaml`, `scoring.yaml`, and `notifications.yaml`. Follow the
[configuration setup](../../../config/README.md); template filenames are not loaded.
Set the following values in an untracked environment file, never tracked YAML:

| Variable | Meaning |
|---|---|
| `DISCORD_BOT_TOKEN` | Bot token |
| `DISCORD_APPLICATION_ID` | Application ID |
| `DISCORD_PUBLIC_KEY` | Hex Ed25519 public key |
| `DISCORD_ALLOWED_USER_IDS` | Comma-separated authorized user IDs |
| `DISCORD_ALLOWED_GUILD_IDS` | Comma-separated authorized guild IDs |
| `DISCORD_ALERT_CHANNEL_ID` | Channel for job/failure alerts |
| `DISCORD_DIGEST_CHANNEL_ID` | Channel for digest |
| `RADAR_INTERACTION_ENCRYPTION_KEY` | 32 random bytes encoded as 64 hex characters |

Generate the encryption key privately with `openssl rand -hex 32`. Pending
interaction tokens are authenticated/encrypted in PostgreSQL and wiped on
completion/expiry. Keep this key stable across restarts; changing it makes pending
tokens unreadable. Allowlists and destinations are required for feedback buttons;
with only a bot token configured, cards are send-only. Provider status reports
configuration readiness, not a live Discord connection probe.

A disabled `discord_secondary` example demonstrates a separate token/application,
public key, allowlists and destination. Enable it and include its destination in
the desired event route to deliver through both applications. Each bot's public
endpoint uses its own provider name and verification key. Environment variable
names are listed without values in `.env.example`; Compose forwards both sets.
Load the private environment before native startup. For Compose, use
`--env-file private/radar.env`; mount private config read-only as described in
[cloud-sync](cloud-sync.md). Never bake private config/profiles into the image.

## Endpoint and command registration

Expose only the necessary backend routes behind a public HTTPS ingress, with
direct backend access private. Set the Interactions Endpoint URL to:

```text
https://YOUR_HOST/discord/discord_primary/interactions
```

Preserve the raw body and Discord signature headers at the proxy. The handler
verifies Ed25519 over timestamp plus exact raw bytes, validates timestamp age,
application ID, and guild/user/channel allowlists. Signed PING returns PONG.
Discord requires an initial response within three seconds; work is durably queued
under a bounded ACK timeout and acknowledged ephemerally before execution.
Deferred replies edit the original response; interaction tokens are usable for
15 minutes, and Radar uses a shorter configured lifetime. See the official
[interaction protocol](https://docs.discord.com/developers/interactions/receiving-and-responding).

Register guild commands explicitly after loading the environment and editing
private YAML. Replace the guild placeholder with one ID from the allowlist:

```sh
cargo run --manifest-path rust/Cargo.toml -p radar-server -- register-discord \
  --provider discord_primary --guild GUILD_ID_FROM_ALLOWLIST \
  --config-dir private/config
```

This performs Discord's guild bulk-command registration and replaces that
application's guild commands with the configured Radar command set. Use a
dedicated Radar application. Registration is never done automatically on startup;
it uses deployed YAML rather than DB overrides. See
[application commands](https://docs.discord.com/developers/interactions/application-commands).

| Command | Behavior |
|---|---|
| `/status` | Mode and persisted scheduler state |
| `/jobs` | Bounded shortlist |
| `/scan` | Queue a bounded mock scan, report acceptance |
| `/pause` | Persist pause, remembering prior mode |
| `/resume` | Restore prior mode |
| `/digest` | Persist a digest request |

Job cards provide Interested, Save and Skip. Skip opens an optional-reason modal;
its submission is deferred and processed through the same feedback service as
TUI/API. Stable delivery identifiers bind actions to the sent job/revision,
provider, channel and source message. Duplicate interaction IDs are replay-safe;
repeated identical actions on a card do not add feedback. Stale conflicting
buttons fail with a refresh diagnostic. No interaction can submit a proposal,
bid, modify a contract, report Applied, or run arbitrary commands.

## Manual real-Discord verification

1. Start PostgreSQL/backend with private config and credentials; confirm readiness
   and Dashboard worker heartbeat. Validate the Portal endpoint's signed PING.
2. Register the commands above. Confirm `/status` works only for allowed users,
   guilds and configured channels; DMs are rejected.
3. In private config, select an active mode, a near-future local scan slot and
   sufficient quotas. Ensure alerts are enabled, thresholds permit the synthetic
   top candidate, and testing occurs outside quiet hours. Observe the scheduled
   run completing and jobs/evaluations appearing in TUI. `/scan` additionally
   proves manual queueing, not scheduled timing.
4. Observe a real Discord card and a `sent` delivery with its remote message ID
   in Alerts. An unconfigured provider or `pending` record is not delivery proof.
5. Click Interested; confirm the decision and feedback in TUI after auto-refresh.
   Exercise Skip with a reason on another card. Repeat a click and check feedback
   count; try an unauthorized account and confirm no state mutation.
6. Exercise `/pause`, `/resume`, `/jobs` and `/digest`. Check scheduler mode and
   digest delivery. Restart the backend and confirm jobs/feedback remain and
   completed runs/cards are not repeated.
7. Record actual evidence privately. Do not mark external acceptance complete
   solely from the automated fake-provider workflow.

Discord delivery honors rate-limit responses; ambiguous channel sends are marked
uncertain for review. Nonces with enforced deduplication protect recent sends,
but Discord does not promise indefinite nonce deduplication. See
[message creation](https://docs.discord.com/developers/resources/message) and
[rate limits](https://docs.discord.com/developers/topics/rate-limits).
