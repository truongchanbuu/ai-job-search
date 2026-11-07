//! Shared PostgreSQL transport policy and diagnostics. Never log server error text or SQL.
use sqlx::{
    ConnectOptions, Connection, PgConnection, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{
    str::FromStr,
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PoolSettings {
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}
impl PoolSettings {
    pub fn from_env() -> anyhow::Result<Self> {
        fn number(key: &str, default: u64, min: u64, max: u64) -> anyhow::Result<u64> {
            let value = std::env::var(key)
                .ok()
                .filter(|v| !v.trim().is_empty())
                .map_or(Ok(default), |v| v.parse::<u64>())
                .map_err(|_| anyhow::anyhow!("{key} must be an integer from {min} to {max}"))?;
            anyhow::ensure!(
                (min..=max).contains(&value),
                "{key} must be an integer from {min} to {max}"
            );
            Ok(value)
        }
        Ok(Self {
            max_connections: number("RADAR_DATABASE_MAX_CONNECTIONS", 3, 2, 8)? as u32,
            acquire_timeout: Duration::from_secs(number(
                "RADAR_DATABASE_ACQUIRE_TIMEOUT_SECONDS",
                10,
                1,
                120,
            )?),
            idle_timeout: Duration::from_secs(number(
                "RADAR_DATABASE_IDLE_TIMEOUT_SECONDS",
                120,
                1,
                3600,
            )?),
            max_lifetime: Duration::from_secs(number(
                "RADAR_DATABASE_MAX_LIFETIME_SECONDS",
                1800,
                30,
                86400,
            )?),
        })
    }
    pub fn options(&self) -> PgPoolOptions {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(0)
            .acquire_timeout(self.acquire_timeout)
            .idle_timeout(Some(self.idle_timeout))
            .max_lifetime(Some(self.max_lifetime))
            .test_before_acquire(true)
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Failure {
    pub variant: &'static str,
    pub category: &'static str,
    pub sqlstate: Option<String>,
    pub transient: bool,
    pub unavailable: bool,
}
pub fn classify(error: &sqlx::Error) -> Failure {
    use sqlx::Error::*;
    let state = error
        .as_database_error()
        .and_then(|e| e.code())
        .filter(|s| s.len() == 5 && s.bytes().all(|c| c.is_ascii_alphanumeric()))
        .map(|s| s.into_owned());
    let (variant, category, transient, unavailable) = match error {
        PoolTimedOut => ("PoolTimedOut", "pool_acquire_timeout", true, true),
        PoolClosed => ("PoolClosed", "pool_closed", false, true),
        Io(_) => ("Io", "connection_io", true, true),
        Tls(_) => ("Tls", "tls_configuration", false, true),
        Protocol(message) if message == "unexpected response from SSLRequest: 0x00" => {
            ("Protocol", "handshake_eof", true, true)
        }
        Protocol(_) => ("Protocol", "connection_protocol", false, true),
        Database(_) => match state.as_deref() {
            Some(s) if s.starts_with("08") => ("Database", "connection_lost", true, true),
            Some("57P01" | "57P02" | "57P03") => ("Database", "database_restarting", true, true),
            Some("53300") => ("Database", "connection_capacity", true, true),
            Some(s) if s.starts_with("28") => ("Database", "authentication", false, true),
            Some("40001" | "40P01") => ("Database", "transaction_conflict", true, false),
            Some("57014" | "55P03") => ("Database", "query_or_lock_timeout", true, false),
            _ => ("Database", "statement_failure", false, false),
        },
        RowNotFound => ("RowNotFound", "missing_row", false, false),
        ColumnNotFound(_)
        | ColumnIndexOutOfBounds { .. }
        | ColumnDecode { .. }
        | Decode(_)
        | TypeNotFound { .. } => ("Decode", "stored_data_or_schema", false, false),
        _ => ("Other", "database_client_failure", false, false),
    };
    Failure {
        variant,
        category,
        sqlstate: state,
        transient,
        unavailable,
    }
}
#[derive(Clone)]
pub struct Operation {
    pub name: String,
    pub correlation: Uuid,
    pub pool: PgPool,
    pub started: Instant,
}
tokio::task_local! {pub static OPERATION: Operation;}
pub fn report(
    error: &sqlx::Error,
    operation: &str,
    stage: &str,
    elapsed: Duration,
    pool: Option<&PgPool>,
    correlation: Uuid,
) {
    eprintln!(
        "{}",
        serde_json::json!({"event":"postgres_failure","operation":operation,"stage":stage,"elapsed_ms":elapsed.as_millis(),"pool_size":pool.map(PgPool::size),"pool_idle":pool.map(PgPool::num_idle),"correlation_id":correlation,"retry_count":0,"failure":classify(error)})
    );
}
pub fn report_current(error: &sqlx::Error) {
    let _ = OPERATION.try_with(|ctx| {
        report(
            error,
            &ctx.name,
            if matches!(error, sqlx::Error::PoolTimedOut) {
                "acquire"
            } else {
                "query_or_commit"
            },
            ctx.started.elapsed(),
            Some(&ctx.pool),
            ctx.correlation,
        )
    });
}
pub fn connect_options(url: &str) -> anyhow::Result<PgConnectOptions> {
    PgConnectOptions::from_str(url)
        .map(|o| {
            o.application_name("radar-server")
                .disable_statement_logging()
        })
        .map_err(|_| anyhow::anyhow!("invalid PostgreSQL configuration; values redacted"))
}
pub async fn connect(url: &str, settings: &PoolSettings) -> anyhow::Result<PgPool> {
    let options = connect_options(url)?;
    // Only a stateless handshake may be retried; never replay a query or transaction.
    let start = Instant::now();
    let correlation = Uuid::new_v4();
    let deadline = tokio::time::Instant::now() + settings.acquire_timeout;
    for attempt in 0..3u32 {
        match tokio::time::timeout_at(deadline, PgConnection::connect_with(&options)).await {
            Ok(Ok(connection)) => {
                connection.close().await.map_err(|e| {
                    report(
                        &e,
                        "startup_connect",
                        "close",
                        start.elapsed(),
                        None,
                        correlation,
                    );
                    anyhow::anyhow!("PostgreSQL connection close failed; details redacted")
                })?;
                break;
            }
            Ok(Err(e)) => {
                let failure = classify(&e);
                eprintln!(
                    "{}",
                    serde_json::json!({"event":"postgres_failure","operation":"startup_connect","stage":"connect","elapsed_ms":start.elapsed().as_millis(),"pool_size":0,"pool_idle":0,"correlation_id":correlation,"retry_count":attempt,"failure":failure})
                );
                if attempt == 2 || !failure.transient || !failure.unavailable {
                    anyhow::bail!("PostgreSQL {}; details redacted", failure.category);
                }
                let jitter = (Uuid::new_v4().as_u128() % 100) as u64;
                tokio::select! {
                    _=tokio::time::sleep_until(deadline)=>anyhow::bail!("PostgreSQL connection_timeout; details redacted"),
                    _=tokio::time::sleep(Duration::from_millis((100u64<<attempt)+jitter))=>{},
                }
            }
            Err(_) => {
                eprintln!(
                    "{}",
                    serde_json::json!({"event":"postgres_failure","operation":"startup_connect","stage":"connect","failure":{"variant":"Deadline","category":"connection_timeout"},"elapsed_ms":start.elapsed().as_millis(),"correlation_id":correlation,"retry_count":attempt})
                );
                anyhow::bail!("PostgreSQL connection_timeout; details redacted");
            }
        }
    }
    Ok(settings.options().connect_lazy_with(options))
}
pub async fn acquire(
    pool: &PgPool,
    name: &str,
) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, sqlx::Error> {
    let start = Instant::now();
    let correlation = OPERATION
        .try_with(|o| o.correlation)
        .unwrap_or_else(|_| Uuid::new_v4());
    match pool.acquire().await {
        Ok(connection) => {
            if start.elapsed() > Duration::from_millis(250) {
                eprintln!(
                    "{}",
                    serde_json::json!({"event":"postgres_acquired","operation":name,"acquisition_ms":start.elapsed().as_millis(),"pool_size":pool.size(),"pool_idle":pool.num_idle(),"correlation_id":correlation,"retry_count":0})
                );
            }
            Ok(connection)
        }
        Err(e) => {
            report(
                &e,
                name,
                "acquire",
                start.elapsed(),
                Some(pool),
                correlation,
            );
            Err(e)
        }
    }
}
/// Bounded, cancellable, read-only probe. Replays no user operation or transaction.
pub async fn probe(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut connection = acquire(pool, "readiness").await?;
    let start = Instant::now();
    match tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::query("SELECT 1").execute(&mut *connection),
    )
    .await
    {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => {
            report(
                &e,
                "readiness",
                "query",
                start.elapsed(),
                Some(pool),
                OPERATION
                    .try_with(|o| o.correlation)
                    .unwrap_or_else(|_| Uuid::new_v4()),
            );
            Err(e)
        }
        Err(_) => {
            connection.close_on_drop();
            let e = sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "readiness deadline",
            ));
            report(
                &e,
                "readiness",
                "query",
                start.elapsed(),
                Some(pool),
                Uuid::new_v4(),
            );
            Err(e)
        }
    }
}

pub async fn commit(
    tx: sqlx::Transaction<'_, sqlx::Postgres>,
    pool: &PgPool,
    operation: &str,
) -> Result<(), sqlx::Error> {
    let start = Instant::now();
    tx.commit().await.inspect_err(|e| {
        report(
            e,
            operation,
            "commit",
            start.elapsed(),
            Some(pool),
            OPERATION
                .try_with(|o| o.correlation)
                .unwrap_or_else(|_| Uuid::new_v4()),
        )
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe_classification_does_not_treat_schema_errors_as_outages_or_echo_details() {
        let io = sqlx::Error::Io(std::io::Error::other(
            "SECRET password postgres://credential",
        ));
        let failure = classify(&io);
        assert!(failure.unavailable && failure.transient);
        assert!(!serde_json::to_string(&failure).unwrap().contains("SECRET"));
        assert!(
            classify(&sqlx::Error::Protocol(
                "unexpected response from SSLRequest: 0x00".into()
            ))
            .transient
        );
        let tls = sqlx::Error::Tls("SECRET certificate error".into());
        assert!(!classify(&tls).transient);
        assert!(!classify(&sqlx::Error::RowNotFound).unavailable);
        assert!(!classify(&sqlx::Error::ColumnNotFound("PRIVATE_FIELD".into())).unavailable);
        assert!(classify(&sqlx::Error::PoolTimedOut).unavailable);
        assert!(!classify(&sqlx::Error::PoolClosed).transient);
    }
}
