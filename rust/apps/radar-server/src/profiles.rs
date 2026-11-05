use super::*;
use radar_application::profiles::*;
pub async fn view(State(state): State<AppState>) -> Result<Json<ProfileView>, ApiError> {
    Ok(Json(state.repository.profile_view(&state.profile).await?))
}
pub async fn save(
    State(state): State<AppState>,
    body: Result<Json<ProfileUpdate>, JsonRejection>,
) -> Result<Json<ProfileView>, ApiError> {
    Ok(Json(
        state
            .repository
            .save_profile(&state.profile, body.map_err(ApiError::json)?.0)
            .await?,
    ))
}
pub async fn preview(
    State(state): State<AppState>,
    body: Result<Json<ProfileUpdate>, JsonRejection>,
) -> Result<Json<radar_application::ai::EvidenceSnapshot>, ApiError> {
    let request = body.map_err(ApiError::json)?.0;
    let profile = radar_adapters::profile_from_documents(request.documents).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_profile",
            "invalid profile modules or evidence; values redacted",
        )
    })?;
    let config = state.repository.config().await?;
    let evidence =
        radar_application::ai::evidence(&profile, config.effective.settings().weekly_hours)
            .map_err(|_| {
                ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "invalid_profile",
                    "invalid evidence; values redacted",
                )
            })?;
    Ok(Json(evidence))
}
pub async fn version(
    State(state): State<AppState>,
    id: Result<Path<String>, PathRejection>,
) -> Result<Json<Profile>, ApiError> {
    Ok(Json(
        state
            .repository
            .profile_version(&id.map_err(ApiError::path)?.0)
            .await?,
    ))
}
