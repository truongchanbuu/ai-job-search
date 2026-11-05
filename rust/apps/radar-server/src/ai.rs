use super::*;
use radar_application::ai::*;

pub async fn history(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<AiHistory>, ApiError> {
    Ok(Json(
        state
            .repository
            .ai_history(
                id.map_err(ApiError::path)?.0,
                &state.current_profile().await?,
            )
            .await?,
    ))
}
async fn start(
    state: AppState,
    id: Uuid,
    kind: TaskKind,
    request: AiRequest,
) -> Result<(StatusCode, Json<AiOperation>), ApiError> {
    let (operation, work) = state
        .repository
        .begin_ai(id, kind, request, &state.current_profile().await?)
        .await?;
    if let Some((input, config)) = work {
        let provider = match &state.ai {
            Some(p) => p.clone(),
            None => radar_adapters::ai::provider(&config.radar.ai.provider).map_err(|_| {
                ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "ai_provider",
                    "unsupported AI provider",
                )
            })?,
        };
        let repository = state.repository.clone();
        let saved = operation.clone();
        tokio::spawn(async move {
            if repository
                .execute_ai(saved, input, config, provider)
                .await
                .is_err()
            {
                eprintln!(
                    "radar: AI operation persistence failed; inspect operation status; details redacted"
                );
            }
        });
    }
    Ok((StatusCode::ACCEPTED, Json(operation)))
}
pub async fn evaluate(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
    body: Result<Json<AiRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<AiOperation>), ApiError> {
    start(
        state,
        id.map_err(ApiError::path)?.0,
        TaskKind::Evaluate,
        body.map_err(ApiError::json)?.0,
    )
    .await
}
pub async fn proposal(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
    body: Result<Json<AiRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<AiOperation>), ApiError> {
    start(
        state,
        id.map_err(ApiError::path)?.0,
        TaskKind::Proposal,
        body.map_err(ApiError::json)?.0,
    )
    .await
}
pub async fn approve(
    State(state): State<AppState>,
    ids: Result<Path<(Uuid, Uuid)>, PathRejection>,
    body: Result<Json<ReviewRequest>, JsonRejection>,
) -> Result<Json<AiHistory>, ApiError> {
    let (job, id) = ids.map_err(ApiError::path)?.0;
    state
        .repository
        .approve_ai(
            job,
            id,
            body.map_err(ApiError::json)?.0.expected_revision,
            &state.current_profile().await?,
        )
        .await?;
    Ok(Json(
        state
            .repository
            .ai_history(job, &state.current_profile().await?)
            .await?,
    ))
}
pub async fn cancel(
    State(state): State<AppState>,
    ids: Result<Path<(Uuid, Uuid)>, PathRejection>,
) -> Result<Json<AiHistory>, ApiError> {
    let (job, id) = ids.map_err(ApiError::path)?.0;
    state.repository.cancel_ai(job, id).await?;
    Ok(Json(
        state
            .repository
            .ai_history(job, &state.current_profile().await?)
            .await?,
    ))
}
pub async fn edit(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
    body: Result<Json<DraftEdit>, JsonRejection>,
) -> Result<Json<ProposalRevision>, ApiError> {
    Ok(Json(
        state
            .repository
            .edit_proposal(
                id.map_err(ApiError::path)?.0,
                body.map_err(ApiError::json)?.0,
                &state.current_profile().await?,
            )
            .await?,
    ))
}
