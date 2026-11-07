use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use std::{
    collections::HashMap,
    fs::File,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::process::{Child, Command};

#[derive(Parser)]
#[command(about = "Open the local Radar backend and terminal app together")]
struct Cli {
    #[arg(long)]
    root: Option<PathBuf>,
    /// Authenticate and check synchronization without entering the TUI. Workers are disabled.
    #[arg(long)]
    check: bool,
    /// Open the web interface instead of the terminal UI.
    #[arg(long, conflicts_with = "check")]
    web: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli.check, cli.web, cli.root).await {
        eprintln!("Freelance Radar: {error}");
        // A desktop-launched window must remain visible when startup fails.
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            eprintln!("Nhan Enter de dong cua so / Press Enter to close.");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
        std::process::exit(1);
    }
}

fn environment(
    path: &Path,
    mut inherited: HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    if path.exists() {
        let entries = dotenvy::from_path_iter(path)
            .map_err(|_| anyhow::anyhow!("Cannot read .env; values redacted"))?;
        for entry in entries {
            let (key, value) =
                entry.map_err(|_| anyhow::anyhow!("Invalid .env syntax; values redacted"))?;
            inherited.entry(key).or_insert(value);
        }
    }
    Ok(inherited)
}

fn endpoint(env: &HashMap<String, String>) -> Result<SocketAddr> {
    let address: SocketAddr = env
        .get("RADAR_BIND")
        .map(String::as_str)
        .unwrap_or("127.0.0.1:8080")
        .parse()
        .context("RADAR_BIND must be a local IP address and port")?;
    ensure!(
        address.ip().is_loopback() && address.port() != 0,
        "Desktop launcher requires a loopback RADAR_BIND with a fixed port"
    );
    if let Some(value) = env.get("RADAR_API_URL").filter(|v| !v.is_empty()) {
        let parsed = url::Url::parse(value)
            .map_err(|_| anyhow::anyhow!("Invalid RADAR_API_URL; value redacted"))?;
        let expected = url::Url::parse(&format!("http://{address}"))?;
        ensure!(
            parsed == expected,
            "RADAR_API_URL must match the local RADAR_BIND for desktop use"
        );
    }
    Ok(address)
}

fn private_file(path: &Path) -> Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    Ok(std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .append(true)
        .mode(0o600)
        .open(path)?)
}

async fn stop(child: &mut Child) {
    if let Some(id) = child.id() {
        // Signal only the child this launcher owns, never an existing server.
        let _ = Command::new("/bin/kill")
            .args(["-TERM", &id.to_string()])
            .status()
            .await;
        if tokio::time::timeout(Duration::from_secs(8), child.wait())
            .await
            .is_err()
        {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }
}

async fn closed() -> Result<()> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut terminate = signal(SignalKind::terminate())?;
    let mut hangup = signal(SignalKind::hangup())?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => result?,
        _ = terminate.recv() => {},
        _ = hangup.recv() => {},
    }
    Ok(())
}

async fn run(check: bool, web: bool, root: Option<PathBuf>) -> Result<()> {
    let root = root
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .canonicalize()
        .context("Radar repository has moved; reinstall the shortcut")?;
    let env = environment(&root.join(".env"), std::env::vars().collect())?;
    let address = endpoint(&env)?;
    let token = env
        .get("RADAR_API_TOKEN")
        .filter(|v| !v.is_empty())
        .context("RADAR_API_TOKEN is missing from environment/.env")?;
    let config = env
        .get("RADAR_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or(root.join("private/config"));
    for name in ["radar.yaml", "scoring.yaml", "notifications.yaml"] {
        ensure!(
            root.join(&config).join(name).is_file(),
            "Missing runtime configuration: {name}; set up private/config first"
        );
    }
    let profile = env
        .get("RADAR_PROFILE_DIR")
        .map(PathBuf::from)
        .unwrap_or(root.join("private/profiles"));
    ensure!(
        root.join(&profile).is_dir(),
        "Private profile directory is missing"
    );
    let bin = std::env::current_exe()?
        .parent()
        .context("Missing binary directory")?
        .to_path_buf();
    for name in ["radar-server", "radar-tui"] {
        ensure!(
            bin.join(name).is_file(),
            "Missing {name}; run cargo build --manifest-path rust/Cargo.toml --workspace --bins"
        );
    }
    let runtime = root.join("private/runtime/desktop");
    std::fs::create_dir_all(&runtime)?;
    let lock = private_file(&runtime.join("launcher.lock"))?;
    let locked = fs2::FileExt::try_lock_exclusive(&lock).is_ok();
    ensure!(
        locked || web,
        "Radar is already open; use its existing window"
    );
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()?;
    let base = format!("http://{address}");
    if !locked {
        if !open_browser(&client, &base, token).await? {
            println!("Click the connection link above, then press Enter to close this window.");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
        return Ok(());
    }
    let live = client.get(format!("{base}/health/live")).send().await;
    let mut server = None;
    if let Ok(response) = live {
        ensure!(
            response.status().is_success(),
            "Another service is using the Radar port"
        );
        println!("Using the existing local backend.");
    } else {
        // A listening but unresponsive service must not trigger a second backend.
        ensure!(
            tokio::net::TcpStream::connect(address).await.is_err(),
            "Radar port is occupied but unresponsive; check the existing backend"
        );
        println!("Starting Freelance Radar...");
        let log = private_file(&runtime.join("backend.log"))?;
        let mut command = Command::new(bin.join("radar-server"));
        command
            .current_dir(&root)
            .env_clear()
            .envs(&env)
            .args(["serve", "--bind", &address.to_string(), "--config-dir"])
            .arg(&config)
            .arg("--profile-dir")
            .arg(&profile)
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log)
            .kill_on_drop(true);
        if check {
            command.arg("--no-workers");
        }
        server = Some(command.spawn().context("Cannot start backend")?);
    }
    let result = session(
        &client,
        &base,
        &bin,
        &root,
        &env,
        token,
        check,
        web,
        &mut server,
    )
    .await;
    if let Some(child) = server.as_mut() {
        stop(child).await;
    }
    result
}

#[allow(clippy::too_many_arguments)]
async fn session(
    client: &reqwest::Client,
    base: &str,
    bin: &Path,
    root: &Path,
    env: &HashMap<String, String>,
    token: &str,
    check: bool,
    web: bool,
    server: &mut Option<Child>,
) -> Result<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(child) = server.as_mut() {
            ensure!(
                child.try_wait()?.is_none(),
                "Backend stopped during startup; inspect private/runtime/desktop/backend.log"
            );
        }
        if let Ok(response) = client.get(format!("{base}/health/ready")).send().await
            && response.status().is_success()
        {
            break;
        }
        ensure!(
            tokio::time::Instant::now() < deadline,
            "Database readiness failed; inspect private/runtime/desktop/backend.log"
        );
        tokio::select! {
            result = closed() => { result?; bail!("Startup cancelled"); },
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
        }
    }
    // Reject an unexpected health response before starting the authenticated client.
    let health = client
        .get(format!("{base}/health/live"))
        .send()
        .await
        .context("Cannot read backend health")?
        .text()
        .await?;
    ensure!(
        health.trim() == r#"{"status":"ok"}"#,
        "The local listener is not Radar"
    );
    if web {
        let opened = open_browser(client, base, token).await?;
        if server.is_none() {
            if !opened {
                println!("Click the connection link above, then press Enter to close this window.");
                let _ = std::io::stdin().read_line(&mut String::new());
            }
            return Ok(());
        }
        if opened {
            println!("Radar is open in your browser.");
        }
        println!("Local backend is running. Close this terminal window to stop it.");
        loop {
            tokio::select! {
                result = closed() => return result,
                _ = tokio::time::sleep(Duration::from_secs(2)) => {
                    if let Some(child) = server.as_mut() {
                        ensure!(child.try_wait()?.is_none(), "Backend stopped; inspect private/runtime/desktop/backend.log");
                    }
                }
            }
        }
    }
    let mut command = Command::new(bin.join("radar-tui"));
    // The terminal client receives only its token and terminal environment, not DB/provider secrets.
    command
        .current_dir(root)
        .env_clear()
        .args(["--api-url", base, "--allow-local-http"])
        .env("RADAR_API_TOKEN", token)
        .kill_on_drop(true);
    for key in [
        "HOME",
        "USER",
        "PATH",
        "TERM",
        "COLORTERM",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TZ",
        "RADAR_HTTP_TIMEOUT_SECONDS",
    ] {
        if let Some(value) = env.get(key) {
            command.env(key, value);
        }
    }
    if check {
        command.arg("--check");
    }
    let mut tui = command.spawn().context("Cannot open terminal client")?;
    let status = tokio::select! {
        result = tui.wait() => result?,
        result = closed() => { stop(&mut tui).await; return result; }
    };
    ensure!(
        status.success(),
        "Terminal client failed; check backend readiness and API token"
    );
    Ok(())
}

async fn open_browser(client: &reqwest::Client, base: &str, token: &str) -> Result<bool> {
    let response = client
        .get(format!("{base}/v1/web-access"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Cannot connect to local backend"))?;
    ensure!(
        response.status().is_success(),
        "Backend does not support browser login or token is invalid; restart the updated backend"
    );
    let value: serde_json::Value = response
        .json()
        .await
        .context("Invalid browser connection response")?;
    let code = value["code"]
        .as_str()
        .context("Missing browser connection code")?;
    ensure!(
        code.len() == 64 && code.bytes().all(|c| c.is_ascii_hexdigit()),
        "Invalid browser connection code"
    );
    let url = format!("{base}/#connect={code}");
    let opened = if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        // WSL --exec launched by Explorer/Terminal may omit Windows directories from PATH.
        let mut native = Command::new("/mnt/c/Windows/System32/rundll32.exe");
        native.args(["url.dll,FileProtocolHandler", &url]);
        if browser_process(native).await {
            true
        } else {
            let mut fallback =
                Command::new("/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe");
            // URL contains only the validated numeric loopback endpoint and a hex nonce.
            fallback.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Start-Process -FilePath '{url}'"),
            ]);
            browser_process(fallback).await
        }
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(&url);
        browser_process(command).await
    };
    if !opened {
        println!("Could not open the default browser. The backend stays available.");
        println!(
            "Ctrl+click this one-use connection link within 60 seconds (no API token needed):"
        );
        println!("{url}");
    }
    Ok(opened)
}

async fn browser_process(mut command: Command) -> bool {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    matches!(tokio::time::timeout(Duration::from_secs(8), command.status()).await, Ok(Ok(status)) if status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dotenv_preserves_environment_and_does_not_execute_shell() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(
            &path,
            "RADAR_API_TOKEN=file-token\nOTHER='$(touch /tmp/never-run-radar)'\n",
        )
        .unwrap();
        let values = environment(
            &path,
            HashMap::from([("RADAR_API_TOKEN".into(), "existing".into())]),
        )
        .unwrap();
        assert_eq!(values["RADAR_API_TOKEN"], "existing");
        assert_eq!(values["OTHER"], "$(touch /tmp/never-run-radar)");
        assert!(
            environment(&dir.path().join("absent"), HashMap::new())
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn malformed_dotenv_does_not_disclose_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(&path, "PRIVATE_SECRET='unterminated").unwrap();
        let error = environment(&path, HashMap::new()).unwrap_err().to_string();
        assert!(!error.contains("PRIVATE_SECRET"));
        assert!(!error.contains("unterminated"));
    }
    #[test]
    fn desktop_endpoint_requires_matching_loopback() {
        assert!(endpoint(&HashMap::new()).unwrap().ip().is_loopback());
        assert!(
            endpoint(&HashMap::from([(
                "RADAR_BIND".into(),
                "0.0.0.0:8080".into()
            )]))
            .is_err()
        );
        assert!(
            endpoint(&HashMap::from([(
                "RADAR_API_URL".into(),
                "https://elsewhere.example".into()
            )]))
            .is_err()
        );
    }
    #[test]
    fn second_launcher_cannot_take_the_lock() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lock");
        let first = private_file(&path).unwrap();
        fs2::FileExt::try_lock_exclusive(&first).unwrap();
        let second = private_file(&path).unwrap();
        assert!(fs2::FileExt::try_lock_exclusive(&second).is_err());
        drop(first);
        assert!(fs2::FileExt::try_lock_exclusive(&second).is_ok());
    }
}
