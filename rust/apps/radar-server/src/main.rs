mod demo;
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use radar_adapters::{
    config::Config,
    load_profile,
    postgres::{MIGRATOR, PgRepository},
};
use radar_server::{AppState, router};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Parser)]
#[command(about = "Personal Freelance Radar — local core and authenticated cloud API")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Radar-owned Upwork authentication and schema discovery. No marketplace writes.
    Upwork {
        #[arg(long)]
        config_dir: Option<PathBuf>,
        #[command(subcommand)]
        action: UpworkCommand,
    },
    Demo {
        #[arg(long)]
        config_dir: Option<PathBuf>,
        #[arg(long)]
        profile_dir: Option<PathBuf>,
        #[arg(long)]
        manual: Option<PathBuf>,
        #[arg(long)]
        at: Option<DateTime<Utc>>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Apply embedded versioned migrations and exit. Requires RADAR_DATABASE_URL.
    Migrate,
    /// Read-only connection/pool smoke test. Never migrates or initializes application state.
    DbCheck {
        #[arg(long,default_value_t=5,value_parser=clap::value_parser!(u32).range(1..=10))]
        reads: u32,
        /// Exercise the real health route on a temporary loopback server; no startup writes/workers.
        #[arg(long)]
        http: bool,
    },
    /// Explicitly register Radar commands in one allowed Discord guild (never runs on startup).
    RegisterDiscord {
        #[arg(long)]
        provider: String,
        #[arg(long)]
        guild: String,
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// Start the API and configured background workers. Only mock scans are enabled.
    Serve {
        /// Disable background processing for isolated API development/tests.
        #[arg(long)]
        no_workers: bool,
        #[arg(long)]
        bind: Option<SocketAddr>,
        #[arg(long)]
        config_dir: Option<PathBuf>,
        #[arg(long)]
        profile_dir: Option<PathBuf>,
    },
}
#[derive(Subcommand)]
enum UpworkCommand {
    Login,
    Discover {
        #[arg(long)]
        account_index: Option<usize>,
        #[arg(long)]
        output: PathBuf,
    },
    /// Install an operator-reviewed mapping of the discovered read-only search schema.
    Bind {
        file: PathBuf,
    },
}
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("radar: {error:#}");
        std::process::exit(1);
    }
}
async fn pool(variable: &str) -> Result<sqlx::PgPool> {
    let url = std::env::var(variable).with_context(|| format!("{variable} must be set"))?;
    let settings = radar_adapters::database::PoolSettings::from_env()?;
    radar_adapters::database::connect(&url, &settings).await
}
async fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    match Cli::parse().command {
        Command::Upwork { config_dir, action } => {
            use radar_adapters::upwork::{self, state::PrivateState};
            let dir = config_dir
                .or_else(|| std::env::var_os("RADAR_CONFIG_DIR").map(PathBuf::from))
                .unwrap_or_else(|| root.join("private/config"));
            let config = Config::load(&dir)?;
            let settings = config
                .radar
                .job_sources
                .upwork_mcp
                .runtime
                .as_ref()
                .context("Upwork runtime configuration missing")?;
            let state = std::sync::Arc::new(PrivateState::from_settings(settings)?);
            if matches!(action, UpworkCommand::Login) {
                return upwork::mcp::login(settings, state)
                    .await
                    .map_err(Into::into);
            }
            let _guard = state.lock()?;
            match action {
                UpworkCommand::Discover {
                    account_index,
                    output,
                } => {
                    state.reserve(settings, chrono::Utc::now().timestamp())?;
                    let mut session =
                        upwork::mcp::OfficialSession::connect(settings, state.clone()).await?;
                    let result =
                        upwork::discover(&mut session, &state, settings, account_index).await;
                    session.close().await;
                    let result = result?;
                    state.write("discovery.enc", &result)?;
                    let mut options = std::fs::OpenOptions::new();
                    options.write(true).create_new(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        options.mode(0o600);
                    }
                    use std::io::Write;
                    options
                        .open(output)?
                        .write_all(&serde_json::to_vec_pretty(&result)?)?;
                    println!(
                        "Saved live discovery privately. Select a read-only action and configure a search binding; no job search was run."
                    );
                }
                UpworkCommand::Bind { file } => {
                    let data = std::fs::read(file)?;
                    ensure!(data.len() <= 128 * 1024, "binding file too large");
                    let binding: upwork::SearchBinding = serde_json::from_slice(&data)
                        .map_err(|_| anyhow::anyhow!("invalid binding JSON; values redacted"))?;
                    let discovery: upwork::Discovery = state
                        .read("discovery.enc")?
                        .context("Run Radar discovery first")?;
                    upwork::verify_binding(&discovery.tools, &discovery.help, &binding)?;
                    state.write("binding.enc", &binding)?;
                    println!(
                        "Read-only search binding installed. Every scan revalidates live schema before execution."
                    );
                }
                UpworkCommand::Login => unreachable!(),
            }
            Ok(())
        }
        Command::Demo {
            config_dir,
            profile_dir,
            manual,
            at,
            output,
        } => demo::run(
            root.canonicalize()?,
            config_dir,
            profile_dir,
            manual,
            at,
            output,
        ),
        Command::DbCheck { reads, http } => {
            let pool = pool("RADAR_DATABASE_URL").await?;
            let mut server = None;
            let mut endpoint = None;
            if http {
                let profile_dir = std::env::var_os("RADAR_PROFILE_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| root.join("rust/fixtures/profiles"));
                let profile = load_profile(&profile_dir)?;
                let state = AppState::new(
                    PgRepository::new(pool.clone()),
                    profile,
                    &uuid::Uuid::new_v4().simple().to_string(),
                    None,
                )?;
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
                endpoint = Some(format!("http://{}/health/ready", listener.local_addr()?));
                server = Some(tokio::spawn(async move {
                    let _ = axum::serve(listener, router(state)).await;
                }));
            }
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(16))
                .no_proxy()
                .build()?;
            for sample in 0..reads {
                if let Some(url) = &endpoint {
                    let response =
                        client.get(url).send().await.map_err(|_| {
                            anyhow::anyhow!("read-only HTTP probe transport failure")
                        })?;
                    anyhow::ensure!(
                        response.status().is_success(),
                        "read-only HTTP readiness failed with status {}; details redacted",
                        response.status().as_u16()
                    );
                    println!(
                        "HTTP readiness read {}: 200 | pool_size={} idle={}",
                        sample + 1,
                        pool.size(),
                        pool.num_idle()
                    );
                } else {
                    radar_adapters::database::probe(&pool).await.map_err(|_| {
                        anyhow::anyhow!("PostgreSQL probe failed; see sanitized diagnostics")
                    })?;
                    println!(
                        "PostgreSQL read {} OK | pool_size={} idle={}",
                        sample + 1,
                        pool.size(),
                        pool.num_idle()
                    );
                }
                if sample + 1 < reads {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
            if let Some(server) = server {
                server.abort();
                let _ = server.await;
            }
            pool.close().await;
            Ok(())
        }
        Command::Migrate => {
            let pool = pool("RADAR_DATABASE_URL").await?;
            MIGRATOR
                .run(&pool)
                .await
                .map_err(|error|{
                    match &error {
                        sqlx::migrate::MigrateError::Execute(e)|sqlx::migrate::MigrateError::ExecuteMigration(e,_)=>radar_adapters::database::report(e,"startup_migrations","migration",std::time::Duration::ZERO,Some(&pool),uuid::Uuid::new_v4()),
                        _=>eprintln!("radar: startup_migrations failed; migration metadata mismatch; values redacted"),
                    }
                    anyhow::anyhow!("database migration failed; details redacted")
                })?;
            pool.close().await;
            println!("Database migrations applied");
            Ok(())
        }
        Command::RegisterDiscord {
            provider,
            guild,
            config_dir,
        } => {
            use radar_adapters::bots::{EnvironmentSecrets, SecretSource};
            let directory = config_dir
                .or_else(|| std::env::var_os("RADAR_CONFIG_DIR").map(PathBuf::from))
                .unwrap_or_else(|| root.join("private/config"));
            let config = Config::load(&directory)?;
            let Some(radar_application::config::Provider::Discord {
                enabled: true,
                token_env,
                application_id_env,
                allowed_guild_ids_env,
                ..
            }) = config.notifications.providers.get(&provider)
            else {
                anyhow::bail!("selected Discord provider is disabled or unknown")
            };
            let secrets = EnvironmentSecrets;
            ensure!(
                secrets
                    .get(allowed_guild_ids_env)
                    .is_some_and(|g| g.split(',').any(|v| v.trim() == guild)),
                "guild is not in the selected bot's configured allowlist"
            );
            let token = secrets
                .get(token_env)
                .context("Discord bot token missing")?;
            let application = secrets
                .get(application_id_env)
                .context("Discord application ID missing")?;
            let timeout = config
                .notifications
                .runtime
                .as_ref()
                .context("bot runtime settings missing")?
                .request_timeout_seconds;
            radar_adapters::discord::DiscordProvider::new(token, timeout, false)
                .map_err(anyhow::Error::msg)?
                .register(&application, &guild, &config)
                .await
                .map_err(anyhow::Error::msg)?;
            println!("Radar commands registered for selected provider and allowed guild");
            Ok(())
        }
        Command::Serve {
            no_workers,
            bind,
            config_dir,
            profile_dir,
        } => {
            let config_dir = config_dir
                .or_else(|| std::env::var_os("RADAR_CONFIG_DIR").map(PathBuf::from))
                .unwrap_or_else(|| root.join("private/config"));
            let profile_dir = profile_dir
                .or_else(|| std::env::var_os("RADAR_PROFILE_DIR").map(PathBuf::from))
                .unwrap_or_else(|| root.join("rust/fixtures/profiles"));
            let config = Config::load(&config_dir)?;
            let profile = load_profile(&profile_dir)?;
            let owner = std::env::var("RADAR_API_TOKEN").context("RADAR_API_TOKEN must be set")?;
            let reader = std::env::var("RADAR_READ_ONLY_TOKEN")
                .ok()
                .filter(|v| !v.is_empty());
            let bind = bind.unwrap_or(
                std::env::var("RADAR_BIND")
                    .unwrap_or_else(|_| "127.0.0.1:8080".into())
                    .parse()
                    .context("invalid RADAR_BIND")?,
            );
            // Non-loopback serving requires an explicit local-dev opt-in or HTTPS ingress declaration.
            if !bind.ip().is_loopback()
                && std::env::var("RADAR_ALLOW_INSECURE_HTTP").as_deref() != Ok("true")
            {
                let public = std::env::var("RADAR_PUBLIC_URL").unwrap_or_default();
                ensure!(
                    public.starts_with("https://") && public.len() > 8,
                    "non-loopback deployment requires HTTPS ingress via RADAR_PUBLIC_URL; RADAR_ALLOW_INSECURE_HTTP=true is for isolated local development only"
                );
            }
            let variable = config
                .radar
                .storage
                .database_url_env
                .strip_prefix("${")
                .and_then(|v| v.strip_suffix('}'))
                .unwrap_or(&config.radar.storage.database_url_env);
            let pool = pool(variable).await?;
            MIGRATOR
                .run(&pool)
                .await
                .map_err(|error|{
                    match &error {
                        sqlx::migrate::MigrateError::Execute(e)|sqlx::migrate::MigrateError::ExecuteMigration(e,_)=>radar_adapters::database::report(e,"startup_migrations","migration",std::time::Duration::ZERO,Some(&pool),uuid::Uuid::new_v4()),
                        _=>eprintln!("radar: startup_migrations failed; migration metadata mismatch; values redacted"),
                    }
                    anyhow::anyhow!("database migration failed; details redacted")
                })?;
            let repository = PgRepository::new(pool.clone());
            repository
                .recover_ai()
                .await
                .map_err(|_| anyhow::anyhow!("AI operation recovery failed; details redacted"))?;
            let state = AppState::new(repository, profile, &owner, reader.as_deref())?;
            state.repository.initialize(&config).await.map_err(|_| {
                anyhow::anyhow!("database configuration initialization failed (details redacted)")
            })?;
            state.refresh_config().await?;
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .context("binding HTTP listener")?;
            println!(
                "Radar API listening on {} | mock scheduler enabled when configured | Discord readiness in /v1/notifications | Upwork manual only (/v1/upwork) | AI manual only (disabled until configured)",
                listener.local_addr()?
            );
            let (shutdown, shutdown_rx) = tokio::sync::watch::channel(false);
            let worker = if no_workers {
                None
            } else {
                Some(tokio::spawn(radar_server::workers::run(
                    state.clone(),
                    shutdown_rx,
                )))
            };
            axum::serve(listener, router(state))
                .with_graceful_shutdown(shutdown_signal())
                .await?;
            let _ = shutdown.send(true);
            if let Some(worker) = worker {
                let _ = worker.await;
            }
            pool.close().await;
            Ok(())
        }
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        if let Ok(mut terminate) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            tokio::select! {_=tokio::signal::ctrl_c()=>{},_=terminate.recv()=>{}}
        } else {
            let _ = tokio::signal::ctrl_c().await;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
