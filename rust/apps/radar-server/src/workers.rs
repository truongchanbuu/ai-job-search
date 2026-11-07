use crate::AppState;
use chrono::{DateTime, Utc};
use radar_adapters::postgres::StoreError;
use tokio::sync::watch;

/// Injectable time and real ports make the entire worker pipeline testable without credentials.
pub async fn tick(state: &AppState, now: DateTime<Utc>) -> Result<(), StoreError> {
    let context = radar_adapters::database::Operation {
        name: "background_tick".into(),
        correlation: uuid::Uuid::new_v4(),
        pool: state.repository.pool.clone(),
        started: std::time::Instant::now(),
    };
    radar_adapters::database::OPERATION
        .scope(context, tick_inner(state, now))
        .await
}
async fn tick_inner(state: &AppState, now: DateTime<Utc>) -> Result<(), StoreError> {
    state.refresh_config().await?;
    sqlx::query("INSERT INTO worker_heartbeats(id,last_seen) VALUES ($1,$2) ON CONFLICT(id) DO UPDATE SET last_seen=excluded.last_seen").bind(state.worker_id).bind(now).execute(&state.repository.pool).await?;
    state
        .repository
        .plan_scans_with_startup(now, state.worker_id)
        .await?;
    if let Some((id, owner)) = state.repository.claim_scan(now).await? {
        state
            .repository
            .execute_scan_with_gateway(
                id,
                owner,
                &state.current_profile().await?,
                now,
                state.upwork.as_ref(),
            )
            .await?;
    }
    state.repository.plan_digest(now).await?;
    state.repository.deliver_one(&state.bots, now).await?;
    state
        .repository
        .process_interaction(&state.bots, now)
        .await?;
    state.repository.reply_interaction(&state.bots, now).await?;
    state.refresh_config().await?;
    Ok(())
}
pub async fn run(state: AppState, mut shutdown: watch::Receiver<bool>) {
    loop {
        if *shutdown.borrow() {
            break;
        }
        tokio::select! {
            _=shutdown.changed()=>break,
            result=tick(&state,Utc::now())=>if result.is_err(){eprintln!("radar: worker tick failed; durable state retained for recovery");},
        }
        let delay = state
            .config_cache
            .read()
            .await
            .as_ref()
            .and_then(|c| c.effective.radar.scheduler.as_ref())
            .map_or(2, |s| s.poll_interval_seconds);
        tokio::select! {_=shutdown.changed()=>{},_=tokio::time::sleep(std::time::Duration::from_secs(delay))=>{}}
    }
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        sqlx::query("DELETE FROM worker_heartbeats WHERE id=$1")
            .bind(state.worker_id)
            .execute(&state.repository.pool),
    )
    .await;
}
