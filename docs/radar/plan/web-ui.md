# Local web interface

The owner-authorized HTML/CSS/JavaScript interface is served by the existing Axum
process at `/`. No Node service, frontend framework, CDN, animation library or
second business pipeline is introduced. TUI and existing API clients continue
to work unchanged. This extends the original terminal-only scope at the owner's
explicit request; it does not authorize cloud deployment or marketplace writes.

## Screens

- **Overview:** current job counts, mode, sources, worker status and next mock scan.
- **Jobs:** search, verdict filter, paging and existing ranking order. Details show
  base score, personalization separately, hard vetoes, original description,
  explicit feedback/reasons and manually reported application outcomes.
- **AI and proposals inside job details:** explicit evaluation, manual review and
  approval, template selection, draft generation, editing, final approval, copy
  and revision history. The UI distinguishes accepted/running requests from
  completed work. Refresh explicitly to inspect async operation results.
- **Profile:** seven labelled private Markdown modules, evidence preview, facts
  confirmation and versioned save. These are Markdown editors, not a rich-text
  CV builder; existing YAML front matter must remain valid. Private files are
  not rewritten by the browser.
- **Templates:** create/edit proposal briefs, bounds and default selection through
  the existing versioned runtime configuration service. No facts are invented.
- **History:** latest feedback, notification delivery statuses and scan executions.
- **Settings:** manual mock/Upwork queueing, pause/resume, work mode, personalization
  diagnostics and validated namespace JSON editing with revision checks.
  Learning model rebuild/activation/rollback remain in the existing TUI/API.

The layout uses a fixed desktop sidebar, responsive cards/tables and compact
horizontal navigation on small screens. Forms have labels, visible focus, status
messages, loading/empty states and confirmation for AI costs, profile facts and
final proposal approval. Failed mutations retain editors. Concurrent/stale saves
return actionable conflict messages. Navigation warns about unsaved changes.

## Launch

```sh
cargo build --manifest-path rust/Cargo.toml --workspace --bins
rust/target/debug/radar-launcher --web
```

Reinstall the desktop shortcut as described in [desktop setup](desktop-launcher.md).
It now passes `--web`. Without that flag, the launcher still opens the TUI.
Alternatively open `http://127.0.0.1:8080/` to resume an existing browser session.
If none exists, open the desktop icon to authenticate automatically. There is no
API-token login form. The launcher reads `.env` on the local machine; the browser
never reads that file.

Browser mode keeps the backend in its terminal window. Closing a browser tab
does not stop the server; closing that terminal requests graceful shutdown of
the backend it owns. Clicking the shortcut again while a launcher is open opens
another browser tab without starting another backend. A backend built before
web support must be restarted first. Supabase still requires internet access.

On WSL the browser opener uses an absolute Windows executable path rather than
depending on the terminal's PATH, with a PowerShell fallback. If the OS still
cannot open a browser, the backend stays running and displays a one-use,
60-second connection link in its local terminal. Ctrl+click authenticates through
the same HttpOnly-cookie flow; no API credential needs to be copied. That link
is sensitive until consumed/expired and should not be shared.

## Browser security

The launcher authenticates to `GET /v1/web-access` and receives a random one-use
connection code valid for 60 seconds. Only that code goes into the browser URL
fragment, which is not sent in HTTP requests. The page immediately removes it
from history and exchanges it at `POST /web/connect`. The long-lived API token
is never placed in a URL, HTML, JavaScript bundle or browser storage.

The resulting browser credential lasts two hours, is stored in a host-only
`HttpOnly; SameSite=Strict` cookie, and retains owner/read-only permissions.
JavaScript never receives the credential. Remote HTTPS cookies also use `Secure`;
loopback HTTP is supported for local desktop use. Ordinary reloads resume the
session through authenticated `GET /v1/session`. The server keeps only
credential hashes in a bounded in-memory registry. Restart invalidates browser
sessions. Logout revokes the browser credential and clears local private state;
if the network is unavailable, the UI reports that server revocation could not
be confirmed. The original API credential remains unchanged.

Existing authorization protects every data read/write. Cookie-authenticated
mutations require a custom same-origin request header and reject mismatched
Origin headers. No permissive CORS is added. Bearer authentication for the TUI
and existing API clients is preserved. Assets use no-store, a strict CSP,
no-referrer, nosniff and frame-ancestors none. Source descriptions, profile text,
feedback and drafts are escaped, never interpreted as HTML/Markdown scripts.
External job links allow only credential-free HTTP(S) with noopener/noreferrer.
No external analytics, fonts or scripts load. Use a trusted browser/device; do not use an untrusted browser extension environment.

## Verification

- Rust tests cover static delivery/security headers, protected APIs, one-use and
  expired codes, browser credential permissions, cookie flags, CSRF rejection and logout revocation without
  needing PostgreSQL.
- `tools/test_radar_web.mjs` is an opt-in Playwright test against an isolated local
  PostgreSQL-backed server. It uses synthetic credentials/jobs and mock AI only.
  It verifies the full jobs → feedback → evaluation → approval → draft/edit/copy
  flow, profile/template persistence, invalid JSON, actual stale-save conflicts,
  connection failures, script injection escaping, read-only authorization and
  mobile layout. Screenshots are synthetic and written under `/tmp`.
- Existing Phase 1–6 PostgreSQL tests remain part of the Rust workspace suite.

Verified on 2026-11-07: 111 Rust tests passed with isolated PostgreSQL and ignored
tests enabled; formatting, Clippy, nine Python tests and both repository guards
passed. Browser tests passed with mock AI and synthetic evidence, including
cookie reload persistence, CSRF rejection and confirmed logout revocation.
An earlier parallel run failed the existing scheduler import-count assertion;
the focused test and subsequent parallel/serial suite runs passed. No scheduler
behavior was changed for the web interface; retain this as a diagnostic caveat.
The WSL opener was also exercised successfully with Windows directories removed
from PATH, reproducing the environment that the old opener could not handle.

Example browser invocation after starting a disposable server on port 18084 with
the synthetic tokens declared in the test, fixture profiles, and mock AI enabled:

```sh
npm install --prefix /tmp/radar-web-browser playwright
/tmp/radar-web-browser/node_modules/.bin/playwright install chromium
RADAR_BROWSER_TEST_URL=http://127.0.0.1:18084 \
RADAR_BROWSER_TEST_ALLOW_WRITES=yes-isolated \
RADAR_PLAYWRIGHT_MODULE=/tmp/radar-web-browser/node_modules/playwright/index.mjs \
  node tools/test_radar_web.mjs
```

Never run browser mutation tests against the normal application or Supabase.
The test rejects the standard application port and requires explicit write opt-in.
Real Upwork/AI availability is unchanged and not established by mock UI tests.

## Files

`rust/apps/radar-server/web/{index.html,app.css,app.js}` contains the frontend;
`src/web.rs` serves embedded assets and browser access grants. Server `lib.rs`
registers routes and recognizes short-lived credentials. `tests/web.rs` verifies
the transport/auth boundary. `radar-launcher`, its manifest/lockfile and
`tools/install_radar_desktop.ps1` add browser launch. Dockerfile/build allowlist
include only the public logo required by embedded assets. No database migration
or private configuration change is required.
