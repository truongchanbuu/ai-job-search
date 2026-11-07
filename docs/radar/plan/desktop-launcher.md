# Desktop launcher

Double-click **Freelance Radar** on the Windows desktop to open the web interface
and a Windows Terminal tab running its backend in WSL. This is a local application; Supabase remains
the configured database and requires an internet connection.

The Rust launcher loads repository `.env` without executing shell commands.
Inherited environment variables take precedence. Runtime configuration and
profile directories retain `RADAR_CONFIG_DIR` / `RADAR_PROFILE_DIR` overrides;
desktop defaults are `private/config` and `private/profiles`. Runtime YAML must
use `radar.yaml`, `scoring.yaml` and `notifications.yaml`, never template names.
Persisted configuration still has its existing precedence.

It reuses an existing local backend or starts `radar-server`, waits up to 60
seconds for readiness, then opens the selected interface. The desktop shortcut
passes `--web`; running the launcher without it still opens the authenticated
TUI. See [browser login and lifecycle](web-ui.md). It never compiles during
normal startup. The local bind defaults to `127.0.0.1:8080`; non-loopback binds
and mismatched API URLs are rejected. A file lock prevents duplicate launcher
sessions. The terminal client receives its API token and terminal settings, not
database/provider credentials from `.env`.

For the web interface, close the backend terminal when done; closing a browser
tab alone does not stop it. In terminal mode use the TUI's quit control. The launcher gracefully stops only
the backend it started. It leaves a pre-existing backend running. Terminal
hangup/termination also requests cleanup; a forced OS/WSL kill cannot guarantee
graceful shutdown. Errors stay visible until Enter is pressed. Backend output
is appended to `private/runtime/desktop/backend.log` with private permissions;
this local log is not uploaded or committed and can be removed while closed.

Normal startup uses the existing server startup/migration behavior and configured
workers. Configured mock schedules and notification outbox processing can run
while the app is open. Upwork searches and AI operations remain manual; opening
the app does not invoke them. Discord interactions still need public HTTPS.

## Build and install

Build once and rebuild after Rust updates:

```sh
cargo build --manifest-path rust/Cargo.toml --workspace --bins
```

From a WSL terminal in this repository, install the Windows shortcut:

```sh
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass \
  -File tools/install_radar_desktop.ps1 \
  -Distro "$WSL_DISTRO_NAME" -LinuxRoot "$PWD"
```

The installer copies the icon to Windows `%LOCALAPPDATA%/FreelanceRadar` and
creates `Freelance Radar.lnk` in the desktop folder reported by Windows, including
OneDrive redirected desktops. Windows Terminal must already be installed. It
updates its own shortcut but refuses to overwrite an unrelated shortcut. After
moving the repository, rebuild and rerun installation. To uninstall, remove that
shortcut and the `FreelanceRadar` icon directory; application data is unaffected.

Direct/headless operation:

```sh
rust/target/debug/radar-launcher
rust/target/debug/radar-launcher --web
rust/target/debug/radar-launcher --check
```

`--check` uses the existing TUI's authenticated synchronization check. It disables
workers for a backend it starts, but cannot disable workers on a reused backend.
It still runs normal server migrations/initialization if starting a backend;
use `radar-server db-check` for a strictly read-only database probe instead.

## Verification

- Four launcher unit tests: environment precedence/literal shell expressions,
  redacted malformed dotenv errors, local endpoint validation and duplicate lock.
- Workspace formatting, Clippy, credential-free tests and repository guards.
  Database tests remain opt-in and were not rerun for this launcher change.
- Real `radar-launcher --check` started the backend, passed readiness and
  authenticated TUI synchronization against the configured Supabase database,
  then stopped its child backend. No live Upwork search or AI call was made.
- The installed Windows shortcut was opened successfully: launcher, backend and
  TUI processes started, and readiness returned HTTP 200. The window was left
  open for the owner. Credential-free workspace results: 75 passed, 33 database
  tests ignored; nine Python tests and both repository guards passed.

## Icon

Assets: `assets/radar/radar.png` and Windows `assets/radar/radar.ico`.
The logo was generated using the built-in image generation tool; the installer
converts the PNG to a 256-pixel Windows icon using System.Drawing.

Design prompt: “A polished desktop application icon for Personal Freelance Radar,
a quiet focused tool for finding relevant freelance programming work. A simple
recognizable radar sweep forming a subtle letter R, one small opportunity dot,
deep midnight rounded-square tile with restrained teal accent. Flat geometric
professional mark, clean bold silhouette, legible at 48px, generous margins,
no text, no mockup, no shadows outside tile.”
