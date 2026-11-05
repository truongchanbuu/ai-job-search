use radar_application::ai::*;
use radar_application::api::*;
use radar_application::learning::*;
use radar_application::profiles::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt, time::Duration};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ClientError {
    pub status: Option<u16>,
    pub code: String,
    pub message: String,
}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for ClientError {}
impl ClientError {
    fn transport() -> Self {
        Self { status:None,code:"connection_error".into(),message:"Connection failed or timed out. Mutation outcome may be unknown; refresh or retry the same operation.".into() }
    }
}
#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base: url::Url,
}
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub profile: ProfileView,
    pub preferences: PreferenceView,
    pub dashboard: Dashboard,
    pub jobs: JobPage,
    pub config: ConfigView,
    pub feedback: FeedbackPage,
    pub scheduler: SchedulerView,
    pub notifications: NotificationHistory,
    pub upwork: UpworkStatus,
}
impl ApiClient {
    pub async fn profile(&self) -> Result<ProfileView, ClientError> {
        self.get("v1/profile").await
    }
    pub async fn save_profile(&self, request: &ProfileUpdate) -> Result<ProfileView, ClientError> {
        Self::decode(
            self.http
                .put(self.url("v1/profile"))
                .json(request)
                .send()
                .await,
        )
        .await
    }
    pub async fn preview_profile(
        &self,
        request: &ProfileUpdate,
    ) -> Result<EvidenceSnapshot, ClientError> {
        self.post("v1/profile/preview", request).await
    }

    pub fn new(base: &str, token: &str, allow_local_http: bool) -> anyhow::Result<Self> {
        let mut base =
            url::Url::parse(base).map_err(|_| anyhow::anyhow!("invalid RADAR_API_URL"))?;
        anyhow::ensure!(
            base.username().is_empty()
                && base.password().is_none()
                && base.query().is_none()
                && base.fragment().is_none()
                && base.path() == "/",
            "API URL must have no credentials, query, fragment or path prefix"
        );
        let local = matches!(
            base.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
        );
        anyhow::ensure!(
            base.scheme() == "https" || (base.scheme() == "http" && local && allow_local_http),
            "HTTPS is required; local HTTP requires --allow-local-http and a loopback host"
        );
        base.set_path("/");
        anyhow::ensure!(!token.is_empty(), "RADAR_API_TOKEN must be set");
        let mut authorization = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| anyhow::anyhow!("invalid API token format"))?;
        authorization.set_sensitive(true);
        let headers = reqwest::header::HeaderMap::from_iter([(
            reqwest::header::AUTHORIZATION,
            authorization,
        )]);
        let timeout = std::env::var("RADAR_HTTP_TIMEOUT_SECONDS")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "45".into())
            .parse::<u64>()
            .map_err(|_| {
                anyhow::anyhow!("RADAR_HTTP_TIMEOUT_SECONDS must be an integer from 1 to 120")
            })?;
        anyhow::ensure!(
            (1..=120).contains(&timeout),
            "RADAR_HTTP_TIMEOUT_SECONDS must be an integer from 1 to 120"
        );
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(timeout))
            .build()?;
        Ok(Self { http, base })
    }
    fn url(&self, path: &str) -> url::Url {
        self.base.join(path).expect("static API path")
    }
    async fn decode<T: DeserializeOwned>(
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<T, ClientError> {
        let response = response.map_err(|_| ClientError::transport())?;
        let status = response.status();
        if !status.is_success() {
            let body = response.json::<ApiErrorBody>().await.ok();
            return Err(ClientError {
                status: Some(status.as_u16()),
                code: body
                    .as_ref()
                    .map(|b| b.error.code.clone())
                    .unwrap_or_else(|| "http_error".into()),
                message: body.map(|b| b.error.message).unwrap_or_else(|| {
                    format!(
                        "HTTP {}: server returned an invalid error response",
                        status.as_u16()
                    )
                }),
            });
        }
        response.json().await.map_err(|_| ClientError {
            status: Some(status.as_u16()),
            code: "invalid_response".into(),
            message: "Server response could not be decoded; refresh before making changes.".into(),
        })
    }
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        Self::decode(self.http.get(self.url(path)).send().await).await
    }
    pub async fn ai_history(&self, id: Uuid) -> Result<AiHistory, ClientError> {
        self.get(&format!("v1/jobs/{id}/ai")).await
    }
    pub async fn ai_start(
        &self,
        id: Uuid,
        kind: TaskKind,
        request: &AiRequest,
    ) -> Result<AiOperation, ClientError> {
        self.post(
            &format!(
                "v1/jobs/{id}/ai/{}",
                if kind == TaskKind::Evaluate {
                    "evaluate"
                } else {
                    "proposals"
                }
            ),
            request,
        )
        .await
    }
    pub async fn ai_approve(
        &self,
        id: Uuid,
        operation: Uuid,
        request: &ReviewRequest,
    ) -> Result<AiHistory, ClientError> {
        self.post(&format!("v1/jobs/{id}/ai/{operation}/approve"), request)
            .await
    }
    pub async fn ai_cancel(&self, id: Uuid, operation: Uuid) -> Result<AiHistory, ClientError> {
        self.post(
            &format!("v1/jobs/{id}/ai/{operation}/cancel"),
            &serde_json::json!({}),
        )
        .await
    }
    pub async fn proposal_edit(
        &self,
        id: Uuid,
        request: &DraftEdit,
    ) -> Result<ProposalRevision, ClientError> {
        Self::decode(
            self.http
                .put(self.url(&format!("v1/jobs/{id}/ai/proposal")))
                .json(request)
                .send()
                .await,
        )
        .await
    }
    pub async fn preferences(&self) -> Result<PreferenceView, ClientError> {
        self.get("v1/preferences").await
    }
    pub async fn rebuild_preferences(
        &self,
        request: &RebuildRequest,
    ) -> Result<PreferenceModel, ClientError> {
        self.post("v1/preferences/rebuild", request).await
    }
    pub async fn control_preferences(
        &self,
        request: &PreferenceControl,
    ) -> Result<PreferenceView, ClientError> {
        self.post("v1/preferences/control", request).await
    }
    pub async fn preference_model(&self, id: Uuid) -> Result<PreferenceModel, ClientError> {
        self.get(&format!("v1/preferences/models/{id}")).await
    }
    pub async fn compare_preferences(&self, id: Uuid) -> Result<ModelComparison, ClientError> {
        self.get(&format!("v1/preferences/models/{id}/compare"))
            .await
    }
    pub async fn calibration(&self) -> Result<Calibration, ClientError> {
        self.get("v1/preferences/calibration").await
    }
    pub async fn correct_feedback(
        &self,
        id: Uuid,
        request: &FeedbackCorrection,
    ) -> Result<FeedbackView, ClientError> {
        Self::decode(
            self.http
                .put(self.url(&format!("v1/feedback/{id}/learning")))
                .json(request)
                .send()
                .await,
        )
        .await
    }
    pub async fn dashboard(&self) -> Result<Dashboard, ClientError> {
        self.get("v1/dashboard").await
    }
    pub async fn config(&self) -> Result<ConfigView, ClientError> {
        self.get("v1/config").await
    }
    pub async fn job(&self, id: Uuid) -> Result<JobView, ClientError> {
        self.get(&format!("v1/jobs/{id}")).await
    }
    pub async fn evaluations(&self, id: Uuid) -> Result<Vec<EvaluationView>, ClientError> {
        self.get(&format!("v1/jobs/{id}/evaluations")).await
    }
    pub async fn jobs(&self, query: &JobQuery) -> Result<JobPage, ClientError> {
        Self::decode(self.http.get(self.url("v1/jobs")).query(query).send().await).await
    }
    pub async fn feedback_history(
        &self,
        query: &FeedbackQuery,
    ) -> Result<FeedbackPage, ClientError> {
        Self::decode(
            self.http
                .get(self.url("v1/feedback"))
                .query(query)
                .send()
                .await,
        )
        .await
    }
    pub async fn snapshot(
        &self,
        jobs: &JobQuery,
        feedback: &FeedbackQuery,
    ) -> Result<Snapshot, ClientError> {
        let (
            dashboard,
            jobs,
            config,
            feedback,
            scheduler,
            notifications,
            upwork,
            preferences,
            profile,
        ) = tokio::try_join!(
            self.dashboard(),
            self.jobs(jobs),
            self.config(),
            self.feedback_history(feedback),
            self.scheduler(),
            self.notifications(),
            self.upwork_status(),
            self.preferences(),
            self.profile()
        )?;
        Ok(Snapshot {
            profile,
            preferences,
            dashboard,
            jobs,
            config,
            feedback,
            scheduler,
            notifications,
            upwork,
        })
    }
    pub async fn scheduler(&self) -> Result<SchedulerView, ClientError> {
        self.get("v1/scheduler").await
    }
    pub async fn notifications(&self) -> Result<NotificationHistory, ClientError> {
        self.get("v1/notifications").await
    }
    pub async fn upwork_status(&self) -> Result<UpworkStatus, ClientError> {
        self.get("v1/upwork").await
    }
    pub async fn upwork_scan(&self, request: &UpworkScanRequest) -> Result<ScanRun, ClientError> {
        Self::decode(
            self.http
                .post(self.url("v1/upwork/scan"))
                .json(request)
                .send()
                .await,
        )
        .await
    }
    pub async fn scan(&self, request: &ScanRequest) -> Result<ScanRun, ClientError> {
        self.post("v1/scheduler/scan", request).await
    }
    pub async fn set_paused(&self, request: &ModeRequest) -> Result<ConfigView, ClientError> {
        self.post("v1/scheduler/mode", request).await
    }
    pub async fn import(&self, request: &ImportRequest) -> Result<ImportResult, ClientError> {
        self.post("v1/imports", request).await
    }
    pub async fn feedback(
        &self,
        id: Uuid,
        request: &FeedbackRequest,
    ) -> Result<FeedbackView, ClientError> {
        self.post(&format!("v1/jobs/{id}/feedback"), request).await
    }
    pub async fn update_config(&self, request: &ConfigUpdate) -> Result<ConfigView, ClientError> {
        Self::decode(
            self.http
                .put(self.url("v1/config"))
                .json(request)
                .send()
                .await,
        )
        .await
    }
    async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        request: &impl Serialize,
    ) -> Result<T, ClientError> {
        Self::decode(self.http.post(self.url(path)).json(request).send().await).await
    }
}

pub async fn execute(
    client: &ApiClient,
    request: &crate::ui::UiRequest,
    jobs: &JobQuery,
    feedback: &FeedbackQuery,
    selected: Option<Uuid>,
) -> crate::ui::NetworkResult {
    use crate::ui::{NetworkResult, UiRequest};
    let mut review = None;
    let mut profile_preview = None;
    let acknowledged = match request {
        UiRequest::SaveProfile(request)=>client.save_profile(request).await.map(|p|Some(format!("Profile saved, revision {}. Existing jobs retain their historical scores; reimport to rescore.",p.revision))),
        UiRequest::PreviewProfile(request)=>client.preview_profile(request).await.map(|e|{profile_preview=Some(e);None}),
        UiRequest::Rebuild(request) => client.rebuild_preferences(request).await.map(|model| {
            Some(format!(
                "Candidate built from {} jobs; inactive until owner activation",
                model.samples
            ))
        }),
        UiRequest::PreferenceControl(request) => {
            client.control_preferences(request).await.map(|view| {
                Some(format!(
                    "Preference control saved at revision {}",
                    view.revision
                ))
            })
        }
        UiRequest::Compare(id) => client.compare_preferences(*id).await.map(|comparison| {
            review = Some(crate::ui::LearningReview {
                comparison: Some(comparison),
                calibration: None,
            });
            None
        }),
        UiRequest::Calibrate => client.calibration().await.map(|calibration| {
            review = Some(crate::ui::LearningReview {
                comparison: None,
                calibration: Some(calibration),
            });
            None
        }),
        UiRequest::CorrectFeedback(id, request) => {
            client.correct_feedback(*id, request).await.map(|feedback| {
                Some(format!(
                    "Learning attributes saved at version {}",
                    feedback.learning_version
                ))
            })
        }
        UiRequest::Evaluate(id, request) => client
            .ai_start(*id, TaskKind::Evaluate, request)
            .await
            .map(|op| {
                Some(format!(
                    "AI evaluation {}; refresh to inspect output",
                    op.status
                ))
            }),
        UiRequest::Generate(id, request) => client
            .ai_start(*id, TaskKind::Proposal, request)
            .await
            .map(|op| {
                Some(format!(
                    "Proposal operation {}; refresh to inspect draft",
                    op.status
                ))
            }),
        UiRequest::ApproveAnalysis(id, op, request) => client
            .ai_approve(*id, *op, request)
            .await
            .map(|_| Some("Evaluation approved for draft generation".into())),
        UiRequest::CancelAi(id, op) => client
            .ai_cancel(*id, *op)
            .await
            .map(|_| Some("Cancellation state saved; refresh operation status".into())),
        UiRequest::EditProposal(id, request) => {
            client.proposal_edit(*id, request).await.map(|draft| {
                Some(format!(
                    "Draft revision {} saved; approved={}",
                    draft.revision,
                    draft.approved_text.is_some()
                ))
            })
        }
        UiRequest::Refresh => Ok(None),
        UiRequest::Upwork(request) => client.upwork_scan(request).await.map(|r| {
            Some(format!(
                "Manual Upwork scan queued: {} ({:?})",
                r.id, r.status
            ))
        }),
        UiRequest::Scan(request) => client.scan(request).await.map(|r| {
            Some(format!(
                "Scan queued: {} ({:?}); refresh to see execution",
                r.id, r.status
            ))
        }),
        UiRequest::Mode(request) => client.set_paused(request).await.map(|r| {
            Some(format!(
                "Server confirmed {:?} mode at revision {}",
                r.effective.radar.app.active_mode, r.revision
            ))
        }),
        UiRequest::Import(request) => client.import(request).await.map(|r| {
            Some(format!(
                "Import: {} • {} records processed • {} canonical jobs",
                r.status, r.processed, r.canonical_jobs
            ))
        }),
        UiRequest::Feedback(id, request) => client.feedback(*id, request).await.map(|r| {
            Some(format!(
                "Server saved {:?} feedback at job revision {}",
                r.action, r.revision
            ))
        }),
        UiRequest::Config(request) => client.update_config(request).await.map(|r| {
            Some(format!(
                "Server saved configuration revision {}",
                r.revision
            ))
        }),
    };
    let acknowledged = match acknowledged {
        Ok(value) => value,
        Err(error) => {
            return NetworkResult {
                profile_preview: None,
                review: None,
                ai: None,
                snapshot: Err(error),
                selected: None,
                acknowledged: None,
            };
        }
    };
    let snapshot = client.snapshot(jobs, feedback).await;
    let selected = if snapshot.is_ok() {
        if let Some(id) = selected {
            match client.job(id).await {
                Ok(job) => Some(job),
                Err(error) => {
                    return NetworkResult {
                        profile_preview: None,
                        review: None,
                        ai: None,
                        snapshot: Err(error),
                        selected: None,
                        acknowledged,
                    };
                }
            }
        } else {
            None
        }
    } else {
        None
    };
    let ai = if let Some(job) = &selected {
        match client.ai_history(job.id).await {
            Ok(history) => Some(history),
            Err(error) => {
                return NetworkResult {
                    profile_preview: None,
                    review: None,
                    snapshot: Err(error),
                    selected: None,
                    acknowledged,
                    ai: None,
                };
            }
        }
    } else {
        None
    };
    NetworkResult {
        profile_preview,
        review,
        ai,
        snapshot,
        selected,
        acknowledged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_explicit_loopback_http_is_allowed_and_urls_cannot_embed_credentials() {
        assert!(ApiClient::new("http://example.com", "test", true).is_err());
        assert!(ApiClient::new("http://127.0.0.1:8080", "test", false).is_err());
        assert!(ApiClient::new("http://127.0.0.1:8080", "test", true).is_ok());
        assert!(ApiClient::new("https://user:secret@example.com", "test", false).is_err());
        assert!(ApiClient::new("https://example.com", "test", false).is_ok());
    }
}
