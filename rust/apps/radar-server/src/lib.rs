mod ai;
mod discord;
mod learning;
mod profiles;
pub mod workers;
use axum::{
    Json, Router,
    extract::{
        DefaultBodyLimit, Extension, Path, Query, Request, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use radar_adapters::{
    bots::BotProviderRegistry,
    postgres::{PgRepository, StoreError},
};
use radar_application::api::*;
use radar_domain::Profile;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub repository: PgRepository,
    profile: Arc<Profile>,
    tokens: Arc<Tokens>,
    pub bots: Arc<BotProviderRegistry>,
    pub upwork: Arc<dyn radar_adapters::upwork::UpworkGateway>,
    pub ai: Option<Arc<dyn radar_application::AiProvider>>,
    worker_id: Uuid,
    config_cache: Arc<tokio::sync::RwLock<Option<ConfigView>>>,
}
struct Tokens {
    owner: [u8; 32],
    reader: Option<[u8; 32]>,
}
#[derive(Clone, Copy)]
struct Principal {
    actor: &'static str,
    can_write: bool,
}

impl AppState {
    pub fn new(
        repository: PgRepository,
        profile: Profile,
        owner: &str,
        reader: Option<&str>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            owner.len() >= 32 && owner.is_ascii() && !owner.chars().any(char::is_whitespace),
            "RADAR_API_TOKEN must contain at least 32 non-whitespace ASCII characters"
        );
        if let Some(reader) = reader {
            anyhow::ensure!(
                reader.len() >= 32
                    && reader != owner
                    && reader.is_ascii()
                    && !reader.chars().any(char::is_whitespace),
                "read-only token must be distinct and contain at least 32 non-whitespace ASCII characters"
            );
        }
        Ok(Self {
            repository,
            profile: Arc::new(profile),
            bots: Arc::new(BotProviderRegistry::default()),
            upwork: Arc::new(radar_adapters::upwork::OfficialUpwork),
            ai: None,
            worker_id: Uuid::new_v4(),
            config_cache: Arc::new(tokio::sync::RwLock::new(None)),
            tokens: Arc::new(Tokens {
                owner: Sha256::digest(owner.as_bytes()).into(),
                reader: reader.map(|v| Sha256::digest(v.as_bytes()).into()),
            }),
        })
    }
    pub fn with_bots(mut self, bots: Arc<BotProviderRegistry>) -> Self {
        self.bots = bots;
        self
    }
    pub fn with_upwork(mut self, gateway: Arc<dyn radar_adapters::upwork::UpworkGateway>) -> Self {
        self.upwork = gateway;
        self
    }
    async fn current_profile(&self) -> Result<Profile, StoreError> {
        self.repository.current_profile(&self.profile).await
    }
    async fn remember_config(&self, config: &ConfigView) {
        let mut cached = self.config_cache.write().await;
        if cached
            .as_ref()
            .is_none_or(|c| c.revision <= config.revision)
        {
            *cached = Some(config.clone());
        }
    }
    pub async fn refresh_config(&self) -> Result<ConfigView, StoreError> {
        let config = self.repository.config().await?;
        self.remember_config(&config).await;
        Ok(config)
    }
}

pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}
impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    fn json(_: JsonRejection) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON body: check required fields, unknown keys and value types",
        )
    }
    fn query(_: QueryRejection) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "invalid_query",
            "invalid query parameters",
        )
    }
    fn path(_: PathRejection) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "invalid_path",
            "job ID must be a UUID",
        )
    }
}
impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Upwork(ref e) => Self::new(
                if e.category == "rate_limited" {
                    StatusCode::TOO_MANY_REQUESTS
                } else {
                    StatusCode::SERVICE_UNAVAILABLE
                },
                "upwork_unavailable",
                e.to_string(),
            ),
            StoreError::Invalid(message) => {
                Self::new(StatusCode::BAD_REQUEST, "validation", message)
            }
            StoreError::Conflict => Self::new(StatusCode::CONFLICT, "conflict", error.to_string()),
            StoreError::NotFound => {
                Self::new(StatusCode::NOT_FOUND, "not_found", error.to_string())
            }
            StoreError::Database(ref database) => {
                let code = database
                    .as_database_error()
                    .and_then(|error| error.code())
                    .filter(|code| {
                        code.len() == 5 && code.chars().all(|c| c.is_ascii_alphanumeric())
                    });
                eprintln!(
                    "radar: database request failed (SQLSTATE={}; details redacted)",
                    code.as_deref().unwrap_or("unavailable")
                );
                Self::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "database_unavailable",
                    "database unavailable; retry after refreshing state",
                )
            }
            StoreError::Data(_) => {
                eprintln!("radar: stored data decoding failed");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "stored data could not be decoded",
                )
            }
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: ErrorDetail {
                    code: self.code.into(),
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
}
async fn authorize(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let candidate: [u8; 32] = Sha256::digest(token.unwrap_or("").as_bytes()).into();
    let owner = bool::from(candidate.ct_eq(&state.tokens.owner));
    let reader = state
        .tokens
        .reader
        .as_ref()
        .is_some_and(|r| bool::from(candidate.ct_eq(r)));
    if token.is_none() || (!owner && !reader) {
        eprintln!("radar: authorization rejected (missing or invalid credential)");
        return ApiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "a valid bearer token is required",
        )
        .into_response();
    }
    if !owner && !matches!(*request.method(), Method::GET | Method::HEAD) {
        eprintln!("radar: authorization rejected (read-only principal)");
        return ApiError::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "this credential is read-only",
        )
        .into_response();
    }
    request.extensions_mut().insert(Principal {
        actor: if owner { "owner" } else { "reader" },
        can_write: owner,
    });
    next.run(request).await
}
async fn no_cache(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/dashboard", get(dashboard))
        .route("/v1/sources", get(sources))
        .route("/v1/jobs", get(jobs))
        .route("/v1/jobs/{id}", get(job))
        .route("/v1/jobs/{id}/evaluations", get(evaluations))
        .route("/v1/jobs/{id}/feedback", post(feedback))
        .route("/v1/feedback", get(feedback_history))
        .route("/v1/imports", post(import))
        .route("/v1/config", get(config).put(update_config))
        .route("/v1/config/history", get(config_history))
        .route("/v1/jobs/{id}/ai", get(ai::history))
        .route("/v1/jobs/{id}/ai/evaluate", post(ai::evaluate))
        .route("/v1/jobs/{id}/ai/proposals", post(ai::proposal))
        .route("/v1/jobs/{id}/ai/proposal", axum::routing::put(ai::edit))
        .route("/v1/jobs/{id}/ai/{operation}/approve", post(ai::approve))
        .route("/v1/jobs/{id}/ai/{operation}/cancel", post(ai::cancel))
        .route("/v1/profile", get(profiles::view).put(profiles::save))
        .route("/v1/profile/preview", post(profiles::preview))
        .route("/v1/profile/versions/{id}", get(profiles::version))
        .route("/v1/preferences", get(learning::view))
        .route("/v1/preferences/rebuild", post(learning::rebuild))
        .route("/v1/preferences/control", post(learning::control))
        .route("/v1/preferences/calibration", get(learning::calibration))
        .route("/v1/preferences/models/{id}", get(learning::model))
        .route(
            "/v1/preferences/models/{id}/compare",
            get(learning::compare),
        )
        .route(
            "/v1/feedback/{id}/learning",
            axum::routing::put(learning::correct),
        )
        .route("/v1/upwork", get(upwork_status))
        .route("/v1/upwork/scan", post(queue_upwork))
        .route("/v1/scheduler", get(scheduler_status))
        .route("/v1/scheduler/scan", post(queue_scan))
        .route("/v1/scheduler/mode", post(set_paused))
        .route("/v1/notifications", get(notification_history))
        .route("/v1/notifications/digest", post(queue_digest))
        .route_layer(middleware::from_fn_with_state(state.clone(), authorize));
    Router::new()
        .merge(protected)
        .route(
            "/health/live",
            get(|| async {
                Json(Health {
                    status: "ok".into(),
                })
            }),
        )
        .route("/health/ready", get(ready))
        .route(
            "/discord/{provider}/interactions",
            post(discord::interaction).layer(DefaultBodyLimit::max(65536)),
        )
        .fallback(|| async {
            ApiError::new(StatusCode::NOT_FOUND, "not_found", "endpoint not found")
        })
        .method_not_allowed_fallback(|| async {
            ApiError::new(
                StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                "method not allowed",
            )
        })
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn(no_cache))
        .with_state(state)
}
async fn ready(State(state): State<AppState>) -> Result<Json<Health>, ApiError> {
    sqlx::query("SELECT 1")
        .execute(&state.repository.pool)
        .await
        .map_err(StoreError::from)?;
    Ok(Json(Health {
        status: "ready".into(),
    }))
}
async fn dashboard(State(state): State<AppState>) -> Result<Json<Dashboard>, ApiError> {
    Ok(Json(state.repository.dashboard().await?))
}
async fn sources(State(state): State<AppState>) -> Result<Json<Vec<SourceView>>, ApiError> {
    Ok(Json(state.repository.sources().await?))
}
async fn jobs(
    State(state): State<AppState>,
    query: Result<Query<JobQuery>, QueryRejection>,
) -> Result<Json<JobPage>, ApiError> {
    Ok(Json(
        state
            .repository
            .jobs(query.map_err(ApiError::query)?.0)
            .await?,
    ))
}
async fn job(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<JobView>, ApiError> {
    Ok(Json(
        state.repository.job(id.map_err(ApiError::path)?.0).await?,
    ))
}
async fn evaluations(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<Vec<EvaluationView>>, ApiError> {
    Ok(Json(
        state
            .repository
            .evaluations(id.map_err(ApiError::path)?.0)
            .await?,
    ))
}
async fn config(State(state): State<AppState>) -> Result<Json<ConfigView>, ApiError> {
    Ok(Json(state.refresh_config().await?))
}
async fn config_history(State(state): State<AppState>) -> Result<Json<Vec<ConfigView>>, ApiError> {
    Ok(Json(state.repository.config_history().await?))
}
async fn update_config(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    body: Result<Json<ConfigUpdate>, JsonRejection>,
) -> Result<Json<ConfigView>, ApiError> {
    debug_assert!(principal.can_write);
    let result = state
        .repository
        .update_config(body.map_err(ApiError::json)?.0, principal.actor)
        .await?;
    state.remember_config(&result).await;
    Ok(Json(result))
}

async fn import(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    body: Result<Json<ImportRequest>, JsonRejection>,
) -> Result<Json<ImportResult>, ApiError> {
    Ok(Json(
        state
            .repository
            .import(
                body.map_err(ApiError::json)?.0,
                &state.current_profile().await?,
                principal.actor,
                Utc::now(),
            )
            .await?,
    ))
}
async fn feedback(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    id: Result<Path<Uuid>, PathRejection>,
    body: Result<Json<FeedbackRequest>, JsonRejection>,
) -> Result<Json<FeedbackView>, ApiError> {
    Ok(Json(
        state
            .repository
            .feedback(
                id.map_err(ApiError::path)?.0,
                body.map_err(ApiError::json)?.0,
                principal.actor,
            )
            .await?,
    ))
}
async fn feedback_history(
    State(state): State<AppState>,
    query: Result<Query<FeedbackQuery>, QueryRejection>,
) -> Result<Json<FeedbackPage>, ApiError> {
    Ok(Json(
        state
            .repository
            .feedback_history(query.map_err(ApiError::query)?.0)
            .await?,
    ))
}

async fn scheduler_status(State(state): State<AppState>) -> Result<Json<SchedulerView>, ApiError> {
    Ok(Json(state.repository.scheduler_status(Utc::now()).await?))
}
async fn queue_scan(
    State(state): State<AppState>,
    body: Result<Json<ScanRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ScanRun>), ApiError> {
    let request = body.map_err(ApiError::json)?.0;
    Ok((
        StatusCode::ACCEPTED,
        Json(
            state
                .repository
                .queue_scan(request.idempotency_key, "manual", Utc::now())
                .await?,
        ),
    ))
}
async fn set_paused(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    body: Result<Json<ModeRequest>, JsonRejection>,
) -> Result<Json<ConfigView>, ApiError> {
    let result = state
        .repository
        .set_paused(body.map_err(ApiError::json)?.0, principal.actor)
        .await?;
    state.refresh_config().await?;
    Ok(Json(result))
}
async fn notification_history(
    State(state): State<AppState>,
) -> Result<Json<NotificationHistory>, ApiError> {
    Ok(Json(
        state.repository.notification_history(&state.bots).await?,
    ))
}
async fn queue_digest(
    State(state): State<AppState>,
    body: Result<Json<ScanRequest>, JsonRejection>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = state
        .repository
        .queue_digest(body.map_err(ApiError::json)?.0.idempotency_key, Utc::now())
        .await?;
    Ok(Json(serde_json::json!({"event_id":id,"status":"queued"})))
}

async fn upwork_status(State(state): State<AppState>) -> Result<Json<UpworkStatus>, ApiError> {
    Ok(Json(state.repository.upwork_status().await?))
}
async fn queue_upwork(
    State(state): State<AppState>,
    body: Result<Json<UpworkScanRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ScanRun>), ApiError> {
    Ok((
        StatusCode::ACCEPTED,
        Json(
            state
                .repository
                .queue_upwork_scan(body.map_err(ApiError::json)?.0, "manual", Utc::now())
                .await?,
        ),
    ))
}
