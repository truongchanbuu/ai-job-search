use crate::{ApiError, AppState};
use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use radar_application::config::digest;
use serde_json::{Value, json};

pub async fn interaction(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let config = state.config_cache.read().await.clone().ok_or_else(|| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "not_ready",
            "interaction configuration is not ready",
        )
    })?;
    let config = config.effective;
    let verifier = state.bots.verifier(&config, &provider).map_err(|_| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Discord verifier unavailable or instance disabled",
        )
    })?;
    let signature = headers
        .get("x-signature-ed25519")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let timestamp = headers
        .get("x-signature-timestamp")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let now = Utc::now();
    let verified = verifier
        .verify(&body, signature, timestamp, now)
        .map_err(|_| {
            eprintln!("radar: Discord interaction signature or sender rejected");
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Discord signature, timestamp or sender rejected",
            )
        })?;
    if verified.ping {
        return Ok(Json(json!({"type":1})));
    }
    let deadline = config
        .notifications
        .runtime
        .as_ref()
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "not_ready",
                "bot runtime settings missing",
            )
        })?
        .acknowledgment_timeout_ms;
    let response = tokio::time::timeout(
        std::time::Duration::from_millis(deadline),
        state.repository.accept_interaction(
            &provider,
            verified,
            &digest(&body),
            &config,
            &state.bots,
            now,
        ),
    )
    .await
    .map_err(|_| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "busy",
            "could not durably acknowledge interaction before deadline; retry safely",
        )
    })??;
    Ok(Json(response))
}
