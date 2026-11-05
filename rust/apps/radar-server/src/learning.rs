use super::*;
use radar_application::learning::*;
pub async fn view(State(state): State<AppState>) -> Result<Json<PreferenceView>, ApiError> {
    Ok(Json(state.repository.preference_view().await?))
}
pub async fn model(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<PreferenceModel>, ApiError> {
    Ok(Json(
        state
            .repository
            .preference_model(id.map_err(ApiError::path)?.0)
            .await?,
    ))
}
pub async fn compare(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<ModelComparison>, ApiError> {
    Ok(Json(
        state
            .repository
            .compare_preferences(id.map_err(ApiError::path)?.0)
            .await?,
    ))
}
pub async fn calibration(State(state): State<AppState>) -> Result<Json<Calibration>, ApiError> {
    Ok(Json(state.repository.preference_calibration().await?))
}
pub async fn rebuild(
    State(state): State<AppState>,
    body: Result<Json<RebuildRequest>, JsonRejection>,
) -> Result<Json<PreferenceModel>, ApiError> {
    Ok(Json(
        state
            .repository
            .rebuild_preferences(body.map_err(ApiError::json)?.0)
            .await?,
    ))
}
pub async fn control(
    State(state): State<AppState>,
    body: Result<Json<PreferenceControl>, JsonRejection>,
) -> Result<Json<PreferenceView>, ApiError> {
    Ok(Json(
        state
            .repository
            .control_preferences(body.map_err(ApiError::json)?.0)
            .await?,
    ))
}
pub async fn correct(
    State(state): State<AppState>,
    id: Result<Path<Uuid>, PathRejection>,
    body: Result<Json<FeedbackCorrection>, JsonRejection>,
) -> Result<Json<FeedbackView>, ApiError> {
    Ok(Json(
        state
            .repository
            .correct_learning_feedback(
                id.map_err(ApiError::path)?.0,
                body.map_err(ApiError::json)?.0,
            )
            .await?,
    ))
}
