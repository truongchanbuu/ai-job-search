mod profile;
mod view;
pub use view::render;

use crate::client::{ClientError, Snapshot};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use radar_application::ai::*;
use radar_application::learning::*;
use radar_application::{api::*, config::Config};
use radar_domain::ReasonCode;
use radar_domain::{FeedbackAction, Mode, Verdict};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Jobs,
    Details,
    Evaluation,
    Config,
    Feedback,
    Notifications,
    Preferences,
    Profiles,
}
impl Screen {
    fn index(self) -> usize {
        match self {
            Self::Dashboard => 0,
            Self::Jobs => 1,
            Self::Details => 2,
            Self::Evaluation => 3,
            Self::Config => 4,
            Self::Feedback => 5,
            Self::Notifications => 6,
            Self::Preferences => 7,
            Self::Profiles => 8,
        }
    }
}
#[derive(Clone)]
pub enum UiRequest {
    SaveProfile(radar_application::profiles::ProfileUpdate),
    PreviewProfile(radar_application::profiles::ProfileUpdate),
    Rebuild(RebuildRequest),
    PreferenceControl(PreferenceControl),
    Compare(Uuid),
    Calibrate,
    CorrectFeedback(Uuid, FeedbackCorrection),
    Evaluate(Uuid, AiRequest),
    Generate(Uuid, AiRequest),
    ApproveAnalysis(Uuid, Uuid, ReviewRequest),
    CancelAi(Uuid, Uuid),
    EditProposal(Uuid, DraftEdit),
    Upwork(UpworkScanRequest),
    Scan(ScanRequest),
    Mode(ModeRequest),
    Refresh,
    Import(ImportRequest),
    Feedback(Uuid, FeedbackRequest),
    Config(Box<ConfigUpdate>),
}
pub enum Effect {
    Copy(String),
    None,
    Quit,
    Request(UiRequest),
}
#[derive(Default)]
pub struct LearningReview {
    pub comparison: Option<ModelComparison>,
    pub calibration: Option<Calibration>,
}
pub struct NetworkResult {
    pub profile_preview: Option<EvidenceSnapshot>,
    pub review: Option<LearningReview>,
    pub ai: Option<AiHistory>,
    pub snapshot: Result<Snapshot, ClientError>,
    pub selected: Option<JobView>,
    pub acknowledged: Option<String>,
}
pub struct Draft {
    pub config: Config,
    pub revision: i64,
    pub dirty: bool,
}
pub enum Input {
    PreferenceReset(String),
    LearningCorrection {
        id: Uuid,
        version: i64,
        code: usize,
        excluded: bool,
        hours: String,
    },
    Proposal {
        revision: i64,
        text: String,
        approve: bool,
    },
    Search(String),
    ConfigValue {
        path: String,
        label: String,
        value: String,
    },
    Feedback {
        reason_code: usize,
        job_id: Uuid,
        revision: i64,
        action: usize,
        reason: String,
        confirmed: bool,
    },
}
pub const ACTIONS: [FeedbackAction; 11] = [
    FeedbackAction::Interested,
    FeedbackAction::Save,
    FeedbackAction::Skip,
    FeedbackAction::NotAvailable,
    FeedbackAction::Applied,
    FeedbackAction::Interview,
    FeedbackAction::Hired,
    FeedbackAction::Rejected,
    FeedbackAction::Completed,
    FeedbackAction::Withdrawn,
    FeedbackAction::Other,
];
pub const REASONS: [Option<ReasonCode>; 12] = [
    None,
    Some(ReasonCode::TechnologyMismatch),
    Some(ReasonCode::TooDifficult),
    Some(ReasonCode::NotInteresting),
    Some(ReasonCode::BudgetTooLow),
    Some(ReasonCode::DeadlineUnrealistic),
    Some(ReasonCode::ScheduleConflict),
    Some(ReasonCode::ScopeUnclear),
    Some(ReasonCode::ClientRisk),
    Some(ReasonCode::TooCompetitive),
    Some(ReasonCode::BetterOpportunityAvailable),
    Some(ReasonCode::Other),
];
pub struct App {
    pub proposal_template_id: Option<String>,
    pub profile_editor: profile::Editor,
    pub learning_review: LearningReview,
    pub model_row: usize,
    pub feature_row: usize,
    pub ai: AiHistory,
    pub ai_view: bool,
    pub proposal_view: bool,
    pub screen: Screen,
    pub snapshot: Option<Snapshot>,
    pub selected: Option<JobView>,
    pub draft: Option<Draft>,
    pub jobs: JobQuery,
    pub feedback: FeedbackQuery,
    pub row: usize,
    pub feedback_row: usize,
    pub config_row: usize,
    pub scroll: u16,
    pub busy: bool,
    pub connected: bool,
    pub status: String,
    pub input: Option<Input>,
    pub failed: Option<UiRequest>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Dashboard,
            proposal_template_id: None,
            profile_editor: profile::Editor::default(),
            learning_review: LearningReview::default(),
            model_row: 0,
            feature_row: 0,
            ai: AiHistory::default(),
            ai_view: false,
            proposal_view: false,
            snapshot: None,
            selected: None,
            draft: None,
            jobs: JobQuery::default(),
            feedback: FeedbackQuery::default(),
            row: 0,
            feedback_row: 0,
            config_row: 0,
            scroll: 0,
            busy: false,
            connected: false,
            status: "Connecting…".into(),
            input: None,
            failed: None,
        }
    }
}
impl App {
    pub fn complete(&mut self, request: UiRequest, result: NetworkResult) {
        self.busy = false;
        let profile_preview = result.profile_preview;
        if matches!(request, UiRequest::SaveProfile(..)) && result.acknowledged.is_some() {
            self.profile_editor.dirty = false;
        }
        if matches!(
            request,
            UiRequest::Rebuild(..)
                | UiRequest::PreferenceControl(..)
                | UiRequest::CorrectFeedback(..)
        ) {
            self.learning_review = LearningReview::default();
        }
        if let Some(review) = result.review {
            if review.comparison.is_some() {
                self.learning_review.comparison = review.comparison;
            }
            if review.calibration.is_some() {
                self.learning_review.calibration = review.calibration;
            }
        }
        if let Some(ai) = result.ai {
            self.ai = ai;
        }
        match result.snapshot {
            Ok(snapshot) => {
                if self.profile_editor.draft.is_none()
                    || (!self.profile_editor.dirty && !self.profile_editor.editing())
                    || matches!(request, UiRequest::SaveProfile(..))
                {
                    self.profile_editor.load(&snapshot.profile);
                }
                if matches!(request, UiRequest::EditProposal(..)) {
                    self.input = None;
                }
                self.connected = true;
                self.row = self.row.min(snapshot.jobs.items.len().saturating_sub(1));
                self.feedback_row = self
                    .feedback_row
                    .min(snapshot.feedback.items.len().saturating_sub(1));
                if let Some(job) = result.selected {
                    self.selected = Some(job);
                }
                if self.draft.as_ref().is_none_or(|d| !d.dirty)
                    || matches!(request, UiRequest::Config(_))
                {
                    self.draft = Some(Draft {
                        config: snapshot.config.effective.clone(),
                        revision: snapshot.config.revision,
                        dirty: false,
                    });
                }
                self.snapshot = Some(snapshot);
                self.failed = None;
                self.status = result
                    .acknowledged
                    .unwrap_or_else(|| "Synced with server".into());
            }
            Err(error) => {
                self.connected = self.snapshot.is_some() && matches!(error.status, Some(400 | 403));
                if let Some(ack) = result.acknowledged {
                    self.status = format!("{ack}; refresh failed: {error}");
                    self.failed = Some(UiRequest::Refresh);
                } else {
                    self.status = if error.status == Some(409) {
                        "Conflict: server has newer state. Refresh; discard/reload the config draft with d before reapplying edits.".into()
                    } else {
                        error.to_string()
                    };
                    self.failed = Some(request);
                }
            }
        }
        if let Some(evidence) = profile_preview {
            self.profile_editor.preview = Some(evidence);
        }
    }
    fn request(&mut self, request: UiRequest) -> Effect {
        if self.busy {
            return Effect::None;
        }
        if !self.connected && !matches!(request, UiRequest::Refresh) {
            self.status="Offline or stale. Press r to refresh before starting another mutation; t retries the same failed operation.".into();
            return Effect::None;
        }
        Effect::Request(request)
    }
    pub fn handle_key(&mut self, key: KeyEvent) -> Effect {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Effect::Quit;
        }
        if self.screen == Screen::Profiles
            && !self.busy
            && let Some(effect) = self.profile_key(key)
        {
            return effect;
        }
        if self.input.is_some() {
            if self.busy {
                return Effect::None;
            }
            return self.input_key(key);
        }
        if key.code == KeyCode::Char('q')
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return Effect::Quit;
        }
        if matches!(self.screen, Screen::Details | Screen::Evaluation)
            && !self.busy
            && let Some(job) = self.selected.clone()
        {
            match key.code {
                KeyCode::Char('e') => {
                    self.ai_view = true;
                    self.proposal_view = false;
                    return self.request(UiRequest::Evaluate(
                        job.id,
                        AiRequest {
                            template_id: None,
                            idempotency_key: Uuid::new_v4(),
                            expected_revision: job.revision,
                            evaluation_id: None,
                        },
                    ));
                }
                KeyCode::Char('a') => {
                    self.ai_view = true;
                    self.proposal_view = false;
                    return self.request(UiRequest::Refresh);
                }
                KeyCode::Char('P') => {
                    self.ai_view = true;
                    self.proposal_view = true;
                    return self.request(UiRequest::Refresh);
                }
                KeyCode::Char('b') => {
                    if let Some(op) = self.ai.operations.iter().find(|o| {
                        o.kind == TaskKind::Evaluate && o.status == "completed" && !o.stale
                    }) {
                        return self.request(UiRequest::ApproveAnalysis(
                            job.id,
                            op.id,
                            ReviewRequest {
                                expected_revision: job.revision,
                            },
                        ));
                    }
                    self.status = "No current completed AI evaluation to approve".into();
                    return Effect::None;
                }
                KeyCode::Char('T') => {
                    self.ai_view = true;
                    self.proposal_view = true;
                    let items = self
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.config.effective.radar.proposals.templates.as_ref())
                        .map(|catalog| catalog.items.as_slice())
                        .unwrap_or(&[]);
                    self.proposal_template_id = match self
                        .proposal_template_id
                        .as_ref()
                        .and_then(|id| items.iter().position(|item| &item.id == id))
                    {
                        Some(index) => items.get(index + 1).map(|t| t.id.clone()),
                        None => items.first().map(|t| t.id.clone()),
                    };
                    self.status = format!(
                        "Proposal template: {}. G explicitly generates a draft.",
                        self.proposal_template_id
                            .as_deref()
                            .unwrap_or("configured default / legacy")
                    );
                    return Effect::None;
                }
                KeyCode::Char('G') => {
                    self.ai_view = true;
                    self.proposal_view = true;
                    let evaluation_id = self
                        .ai
                        .operations
                        .iter()
                        .find(|o| o.kind == TaskKind::Evaluate && o.approved && !o.stale)
                        .map(|o| o.id);
                    return self.request(UiRequest::Generate(
                        job.id,
                        AiRequest {
                            template_id: self.proposal_template_id.clone(),
                            idempotency_key: Uuid::new_v4(),
                            expected_revision: job.revision,
                            evaluation_id,
                        },
                    ));
                }
                KeyCode::Char('E') => {
                    if let Some(draft) = self.ai.drafts.first() {
                        self.input = Some(Input::Proposal {
                            revision: draft.revision,
                            text: draft
                                .user_text
                                .clone()
                                .unwrap_or_else(|| draft.ai_text.clone()),
                            approve: false,
                        });
                    }
                    return Effect::None;
                }
                KeyCode::Char('Y') => {
                    if let Some(text) = self
                        .ai
                        .drafts
                        .first()
                        .filter(|draft| {
                            self.ai
                                .operations
                                .iter()
                                .any(|op| op.id == draft.operation_id && !op.stale)
                        })
                        .and_then(|d| d.approved_text.clone())
                    {
                        return Effect::Copy(text);
                    }
                    self.status = "Review and approve the final draft before copying".into();
                    return Effect::None;
                }
                KeyCode::Char('C') => {
                    if let Some(op) = self.ai.operations.iter().find(|o| o.status == "running") {
                        return self.request(UiRequest::CancelAi(job.id, op.id));
                    }
                    return Effect::None;
                }
                KeyCode::Char('3' | '4') => self.ai_view = false,
                _ => {}
            }
        }
        if self.screen == Screen::Preferences && !self.busy {
            let view = self
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.preferences.clone());
            if let Some(view) = view {
                let selected = view.models.get(self.model_row);
                let feature = selected
                    .and_then(|model| model.weights.keys().nth(self.feature_row))
                    .cloned();
                let action = match key.code {
                    KeyCode::Char('a') => Some(PreferenceAction::Activate),
                    KeyCode::Char('b') => Some(PreferenceAction::Rollback),
                    KeyCode::Char('d') => Some(PreferenceAction::Disable),
                    KeyCode::Char('e') => Some(PreferenceAction::Enable),
                    KeyCode::Char('s') => Some(PreferenceAction::Suppress),
                    KeyCode::Char('u') => Some(PreferenceAction::Unsuppress),
                    KeyCode::Char('z') => Some(PreferenceAction::Reject),
                    _ => None,
                };
                if let Some(action) = action {
                    return self.request(UiRequest::PreferenceControl(PreferenceControl {
                        expected_revision: view.revision,
                        action,
                        model_id: selected.map(|model| model.id),
                        feature,
                        confirm_reset: false,
                    }));
                }
                match key.code {
                    KeyCode::Char('R') => {
                        self.model_row = 0;
                        return self.request(UiRequest::Rebuild(RebuildRequest {
                            idempotency_key: Uuid::new_v4(),
                            cutoff: None,
                        }));
                    }
                    KeyCode::Char('v') => {
                        if let Some(model) = selected {
                            return self.request(UiRequest::Compare(model.id));
                        }
                    }
                    KeyCode::Char('c') => return self.request(UiRequest::Calibrate),
                    KeyCode::Char('x') => {
                        self.input = Some(Input::PreferenceReset(String::new()));
                        return Effect::None;
                    }
                    KeyCode::Char('Y') => {
                        if let Some(model) = selected {
                            return Effect::Copy(
                                serde_json::to_string_pretty(model).unwrap_or_default(),
                            );
                        }
                    }
                    KeyCode::Char('[') => {
                        self.feature_row = self.feature_row.saturating_sub(1);
                        return Effect::None;
                    }
                    KeyCode::Char(']') => {
                        self.feature_row = (self.feature_row + 1)
                            .min(selected.map_or(0, |m| m.weights.len().saturating_sub(1)));
                        return Effect::None;
                    }
                    _ => {}
                }
            }
        }
        if self.screen == Screen::Feedback && !self.busy && key.code == KeyCode::Char('R') {
            if let Some(feedback) = self
                .snapshot
                .as_ref()
                .and_then(|s| s.feedback.items.get(self.feedback_row))
            {
                self.input = Some(Input::LearningCorrection {
                    id: feedback.id,
                    version: feedback.learning_version,
                    code: REASONS
                        .iter()
                        .position(|c| *c == feedback.reason_code)
                        .unwrap_or(0),
                    excluded: feedback.learning_excluded,
                    hours: feedback
                        .actual_effort_hours
                        .map(|h| h.to_string())
                        .unwrap_or_default(),
                });
            }
            return Effect::None;
        }
        match key.code {
            KeyCode::Char('1') => self.screen = Screen::Dashboard,
            KeyCode::Char('2') => self.screen = Screen::Jobs,
            KeyCode::Char('3') => self.screen = Screen::Details,
            KeyCode::Char('4') => self.screen = Screen::Evaluation,
            KeyCode::Char('5') => self.screen = Screen::Config,
            KeyCode::Char('6') => self.screen = Screen::Feedback,
            KeyCode::Char('7') => self.screen = Screen::Notifications,
            KeyCode::Char('8') => self.screen = Screen::Preferences,
            KeyCode::Char('9') => self.screen = Screen::Profiles,
            KeyCode::Tab => {
                self.screen = [
                    Screen::Dashboard,
                    Screen::Jobs,
                    Screen::Details,
                    Screen::Evaluation,
                    Screen::Config,
                    Screen::Feedback,
                    Screen::Notifications,
                    Screen::Preferences,
                    Screen::Profiles,
                ][(self.screen.index() + 1) % 9]
            }
            KeyCode::Char('r') => return self.request(UiRequest::Refresh),
            KeyCode::Char('t') if !self.busy => {
                if let Some(request) = self.failed.clone() {
                    return Effect::Request(request);
                }
            }
            KeyCode::Char('u') => {
                return self.request(UiRequest::Upwork(UpworkScanRequest {
                    idempotency_key: Uuid::new_v4(),
                    profile: None,
                }));
            }
            KeyCode::Char('m') => {
                return self.request(UiRequest::Scan(ScanRequest {
                    idempotency_key: Uuid::new_v4(),
                }));
            }
            KeyCode::Char('p') => {
                if let Some(snapshot) = &self.snapshot {
                    return self.request(UiRequest::Mode(ModeRequest {
                        idempotency_key: Uuid::new_v4(),
                        expected_revision: snapshot.config.revision,
                        paused: snapshot.config.effective.radar.app.active_mode != Mode::Paused,
                    }));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_row(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_row(-1),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(10),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
            KeyCode::Enter if self.screen == Screen::Jobs => {
                self.selected = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.jobs.items.get(self.row))
                    .cloned();
                self.ai = AiHistory::default();
                self.ai_view = false;
                self.screen = Screen::Details;
                self.scroll = 0;
            }
            KeyCode::Char('/') if self.screen == Screen::Jobs && !self.busy => {
                self.input = Some(Input::Search(self.jobs.search.clone().unwrap_or_default()))
            }
            KeyCode::Char('v') if self.screen == Screen::Jobs && !self.busy => {
                self.jobs.verdict = match self.jobs.verdict {
                    None => Some(Verdict::ApplyCandidate),
                    Some(Verdict::ApplyCandidate) => Some(Verdict::Review),
                    Some(Verdict::Review) => Some(Verdict::Skip),
                    _ => None,
                };
                self.jobs.page = Some(1);
                self.row = 0;
                return self.request(UiRequest::Refresh);
            }
            KeyCode::Char('g') if self.screen == Screen::Jobs && !self.busy => {
                self.jobs.decision = match self.jobs.decision {
                    None => Some(FeedbackAction::Interested),
                    Some(FeedbackAction::Interested) => Some(FeedbackAction::Save),
                    Some(FeedbackAction::Save) => Some(FeedbackAction::Skip),
                    Some(FeedbackAction::Skip) => Some(FeedbackAction::NotAvailable),
                    _ => None,
                };
                self.jobs.page = Some(1);
                return self.request(UiRequest::Refresh);
            }
            KeyCode::Char('o') if self.screen == Screen::Jobs && !self.busy => {
                self.jobs.sort = Some(match self.jobs.sort.unwrap_or_default() {
                    JobSort::Score => JobSort::Newest,
                    JobSort::Newest => JobSort::Title,
                    JobSort::Title => JobSort::Score,
                });
                return self.request(UiRequest::Refresh);
            }
            KeyCode::Char('[') | KeyCode::Char(']')
                if !self.busy && matches!(self.screen, Screen::Jobs | Screen::Feedback) =>
            {
                let next = key.code == KeyCode::Char(']');
                let (current, size, total) = match self.snapshot.as_ref() {
                    Some(s) if self.screen == Screen::Jobs => {
                        (s.jobs.page, s.jobs.page_size, s.jobs.total)
                    }
                    Some(s) => (s.feedback.page, s.feedback.page_size, s.feedback.total),
                    None => return Effect::None,
                };
                let page = if next && i64::from(current) * i64::from(size) < total {
                    current + 1
                } else if !next {
                    current.saturating_sub(1).max(1)
                } else {
                    current
                };
                if self.screen == Screen::Jobs {
                    self.jobs.page = Some(page);
                    self.row = 0;
                } else {
                    self.feedback.page = Some(page);
                    self.feedback_row = 0;
                }
                return self.request(UiRequest::Refresh);
            }
            KeyCode::Char('e') | KeyCode::Enter if self.screen == Screen::Config && !self.busy => {
                if let Some(draft) = &self.draft
                    && let Some((label, path, value)) =
                        config_fields(&draft.config).get(self.config_row)
                {
                    self.input = Some(Input::ConfigValue {
                        path: path.clone(),
                        label: label.clone(),
                        value: value.clone(),
                    });
                }
            }
            KeyCode::Char('s') if self.screen == Screen::Config => {
                if let Some(draft) = &self.draft {
                    if !draft.dirty {
                        self.status = "No config changes to save".into();
                        return Effect::None;
                    }
                    return self.request(UiRequest::Config(Box::new(ConfigUpdate {
                        expected_revision: draft.revision,
                        radar: Some(draft.config.radar.clone()),
                        scoring: Some(draft.config.scoring.clone()),
                        notifications: Some(draft.config.notifications.clone()),
                    })));
                }
            }
            KeyCode::Char('d') if self.screen == Screen::Config && !self.busy => {
                if let Some(snapshot) = &self.snapshot {
                    self.draft = Some(Draft {
                        config: snapshot.config.effective.clone(),
                        revision: snapshot.config.revision,
                        dirty: false,
                    });
                    self.status="Draft discarded; loaded last fetched server configuration. Press r if offline.".into();
                }
            }
            KeyCode::Char('f')
            | KeyCode::Char('i')
            | KeyCode::Char('s')
            | KeyCode::Char('x')
            | KeyCode::Char('n')
                if matches!(
                    self.screen,
                    Screen::Jobs | Screen::Details | Screen::Evaluation
                ) && !self.busy =>
            {
                let selected = if self.screen == Screen::Jobs {
                    self.snapshot
                        .as_ref()
                        .and_then(|s| s.jobs.items.get(self.row))
                } else {
                    self.selected.as_ref()
                };
                if let Some(job) = selected {
                    let action = match key.code {
                        KeyCode::Char('s') => 1,
                        KeyCode::Char('x') => 2,
                        KeyCode::Char('n') => 3,
                        _ => 0,
                    };
                    self.input = Some(Input::Feedback {
                        reason_code: 0,
                        job_id: job.id,
                        revision: job.revision,
                        action,
                        reason: String::new(),
                        confirmed: false,
                    });
                }
            }
            KeyCode::Esc => {
                self.screen = Screen::Jobs;
                self.scroll = 0;
            }
            _ => {}
        }
        Effect::None
    }
    fn move_row(&mut self, delta: i32) {
        let step = |value: &mut usize, len: usize| {
            if delta > 0 {
                *value = (*value + 1).min(len.saturating_sub(1));
            } else {
                *value = value.saturating_sub(1);
            }
        };
        match self.screen {
            Screen::Jobs => step(
                &mut self.row,
                self.snapshot.as_ref().map_or(0, |s| s.jobs.items.len()),
            ),
            Screen::Config => step(
                &mut self.config_row,
                self.draft
                    .as_ref()
                    .map_or(0, |d| config_fields(&d.config).len()),
            ),
            Screen::Profiles => {}
            Screen::Preferences => {
                self.feature_row = 0;
                step(
                    &mut self.model_row,
                    self.snapshot
                        .as_ref()
                        .map_or(0, |s| s.preferences.models.len()),
                );
            }
            Screen::Feedback => step(
                &mut self.feedback_row,
                self.snapshot.as_ref().map_or(0, |s| s.feedback.items.len()),
            ),
            Screen::Details | Screen::Evaluation | Screen::Notifications | Screen::Dashboard => {
                if delta > 0 {
                    self.scroll = self.scroll.saturating_add(1);
                } else {
                    self.scroll = self.scroll.saturating_sub(1);
                }
            }
        }
    }
    fn input_key(&mut self, key: KeyEvent) -> Effect {
        if key.code == KeyCode::Esc {
            self.input = None;
            return Effect::None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r') {
            match &mut self.input {
                Some(Input::Feedback { reason_code, .. }) => {
                    *reason_code = (*reason_code + 1) % REASONS.len()
                }
                Some(Input::LearningCorrection { code, .. }) => *code = (*code + 1) % REASONS.len(),
                _ => {}
            }
            return Effect::None;
        }
        if let Some(Input::LearningCorrection {
            id,
            version,
            code,
            excluded,
            hours,
        }) = &mut self.input
        {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('d') {
                *excluded = !*excluded;
                return Effect::None;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                let actual = if hours.is_empty() {
                    None
                } else {
                    match hours.parse::<f64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            self.status = "Enter actual effort as hours or leave empty".into();
                            return Effect::None;
                        }
                    }
                };
                let request = UiRequest::CorrectFeedback(
                    *id,
                    FeedbackCorrection {
                        expected_version: *version,
                        reason_code: REASONS[*code],
                        excluded: *excluded,
                        actual_effort_hours: actual,
                    },
                );
                self.input = None;
                return self.request(request);
            }
        }
        if let Some(Input::Proposal {
            revision,
            text,
            approve,
        }) = &mut self.input
        {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('a') {
                *approve = !*approve;
                return Effect::None;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                let request = self.selected.as_ref().map(|job| {
                    UiRequest::EditProposal(
                        job.id,
                        DraftEdit {
                            expected_revision: *revision,
                            text: text.clone(),
                            approve: *approve,
                        },
                    )
                });
                return request.map_or(Effect::None, |r| self.request(r));
            }
            if key.code == KeyCode::Enter {
                if text.len() < 32768 {
                    text.push('\n');
                }
                return Effect::None;
            }
        }
        if key.code == KeyCode::Char('s')
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && let Some(Input::Feedback {
                job_id,
                revision,
                action,
                reason,
                confirmed,
                reason_code,
            }) = &self.input
        {
            let request = UiRequest::Feedback(
                *job_id,
                FeedbackRequest {
                    reason_code: REASONS[*reason_code],
                    idempotency_key: Uuid::new_v4(),
                    expected_revision: *revision,
                    action: ACTIONS[*action],
                    reason: (!reason.trim().is_empty()).then(|| reason.clone()),
                    confirmed_submission: *confirmed,
                    submitted_at: None,
                },
            );
            self.input = None;
            return self.request(request);
        }
        if key.code == KeyCode::Enter {
            match self.input.take().unwrap() {
                Input::PreferenceReset(value) => {
                    if value != "RESET" {
                        self.status =
                            "Type RESET exactly to clear activation without deleting history"
                                .into();
                        self.input = Some(Input::PreferenceReset(value));
                        return Effect::None;
                    }
                    if let Some(snapshot) = &self.snapshot {
                        return self.request(UiRequest::PreferenceControl(PreferenceControl {
                            expected_revision: snapshot.preferences.revision,
                            action: PreferenceAction::Reset,
                            model_id: None,
                            feature: None,
                            confirm_reset: true,
                        }));
                    }
                }
                Input::Search(value) => {
                    self.jobs.search = Some(value);
                    self.jobs.page = Some(1);
                    self.row = 0;
                    return self.request(UiRequest::Refresh);
                }
                Input::ConfigValue { path, label, value } => {
                    if let Some(draft) = &mut self.draft {
                        match edit_config(&draft.config, &path, &value) {
                            Ok(config) => {
                                draft.config = config;
                                draft.dirty = true;
                                self.status =
                                    "Unsaved config draft; s sends it for server validation".into();
                            }
                            Err(message) => {
                                self.status = message;
                                self.input = Some(Input::ConfigValue { path, label, value });
                            }
                        }
                    }
                }
                input => self.input = Some(input),
            }
            return Effect::None;
        }
        if let Some(input) = &mut self.input {
            if let Input::Feedback {
                action, confirmed, ..
            } = input
            {
                match key.code {
                    KeyCode::Up => {
                        *action = action.saturating_sub(1);
                        return Effect::None;
                    }
                    KeyCode::Down => {
                        *action = (*action + 1).min(ACTIONS.len() - 1);
                        return Effect::None;
                    }
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        *confirmed = !*confirmed;
                        return Effect::None;
                    }
                    _ => {}
                }
            }
            let (text, limit) = match input {
                Input::Proposal { text, .. } => (text, 32768),
                Input::PreferenceReset(value) => (value, 5),
                Input::LearningCorrection { hours, .. } => (hours, 32),
                Input::Search(v) => (v, 200),
                Input::ConfigValue { value, .. } => (value, 100),
                Input::Feedback { reason, .. } => (reason, 2000),
            };
            match key.code {
                KeyCode::Backspace => {
                    text.pop();
                }
                KeyCode::Char(c)
                    if !c.is_control()
                        && !key.modifiers.contains(KeyModifiers::CONTROL)
                        && text.len() + c.len_utf8() <= limit =>
                {
                    text.push(c)
                }
                _ => {}
            }
        }
        Effect::None
    }
}
pub fn edit_config(config: &Config, path: &str, value: &str) -> Result<Config, String> {
    let mut json = serde_json::to_value(config).map_err(|_| "Unable to edit config")?;
    let target = json.pointer_mut(path).ok_or("Unknown config field")?;
    *target = if target.is_boolean() {
        serde_json::json!(value.parse::<bool>().map_err(|_| "Enter true or false")?)
    } else if target.is_number() || target.is_null() {
        let parsed: serde_json::Value =
            serde_json::from_str(value).map_err(|_| "Enter a JSON number or null")?;
        if !parsed.is_number() && !parsed.is_null() {
            return Err("Enter a JSON number or null".into());
        }
        parsed
    } else {
        serde_json::json!(value)
    };
    serde_json::from_value(json)
        .map_err(|_| "Invalid value; use employed, available, busy or paused for mode".into())
}
pub fn config_fields(config: &Config) -> Vec<(String, String, String)> {
    let mode = match config.radar.app.active_mode {
        Mode::Employed => "employed",
        Mode::Available => "available",
        Mode::Busy => "busy",
        Mode::Paused => "paused",
    };
    let mut paths = vec![
        ("Work mode".into(), "/radar/app/active_mode".into()),
        ("Timezone".into(), "/radar/app/timezone".into()),
    ];
    for (label, key) in [
        ("Weekly freelance hours", "weekly_freelance_hours"),
        ("Source requests / day", "max_source_requests_per_day"),
        ("Jobs processed / day", "max_jobs_processed_per_day"),
        ("Deep evaluations / day", "max_deep_evaluations_per_day"),
    ] {
        paths.push((
            format!("{label} ({mode})"),
            format!("/radar/modes/{mode}/{key}"),
        ));
    }
    paths.push((
        "Shortlist threshold".into(),
        "/scoring/thresholds/shortlist_min_score".into(),
    ));
    paths.push((
        "Instant alert threshold (inactive)".into(),
        "/scoring/thresholds/instant_alert_min_score".into(),
    ));
    for key in [
        "technical_fit",
        "verified_experience",
        "scope_clarity",
        "budget_effort_fit",
        "availability_fit",
        "client_risk",
        "personal_interest",
    ] {
        paths.push((format!("Weight: {key}"), format!("/scoring/weights/{key}")));
    }
    if config.radar.scheduler.is_some() {
        for (label, key) in [
            ("Scheduler enabled", "enabled"),
            ("Scan jobs / run", "max_jobs_per_run"),
            ("Pending scan limit", "max_pending_runs"),
            ("Terminal refresh seconds", "ui_refresh_seconds"),
            ("Worker poll seconds", "poll_interval_seconds"),
            ("Catch-up window minutes", "catch_up_minutes"),
        ] {
            paths.push((label.into(), format!("/radar/scheduler/{key}")));
        }
    }
    for (label, path) in [
        (
            "Notifications enabled",
            "/notifications/notifications/enabled",
        ),
        (
            "Alerts / day",
            "/notifications/notifications/max_alerts_per_day",
        ),
        (
            "Quiet hours start",
            "/notifications/notifications/quiet_hours/start",
        ),
        (
            "Quiet hours end",
            "/notifications/notifications/quiet_hours/end",
        ),
        (
            "Discord interactions enabled",
            "/notifications/interactions/discord_enabled",
        ),
    ] {
        paths.push((label.into(), path.into()));
    }
    paths.push(("AI provider".into(), "/radar/ai/provider".into()));
    if config.radar.ai.runtime.is_some() {
        for (label, key) in [
            ("Manual AI enabled", "enabled"),
            ("AI model", "model"),
            ("AI evaluations / day", "max_evaluations_per_day"),
            ("AI proposals / day", "max_proposals_per_day"),
            ("Proposal max words", "max_proposal_words"),
        ] {
            paths.push((label.into(), format!("/radar/ai/runtime/{key}")));
        }
    }
    paths.push((
        "AI timeout seconds".into(),
        "/radar/ai/timeout_seconds".into(),
    ));
    paths.push((
        "AI concurrency".into(),
        "/radar/ai/max_concurrent_tasks".into(),
    ));
    if config.scoring.personalization.runtime.is_some() {
        paths.push((
            "Personalization enabled".into(),
            "/scoring/personalization/runtime/enabled".into(),
        ));
    }
    if config.scoring.personalization.runtime.is_some() {
        for (label, path) in [
            ("Personalization cap", "max_adjustment"),
            ("Learning prior strength", "prior_strength"),
            ("Minimum feature samples", "min_feature_samples"),
            ("Training window days (null=all)", "training_window_days"),
        ] {
            paths.push((
                label.into(),
                format!("/scoring/personalization/runtime/{path}"),
            ));
        }
    }
    let json = serde_json::to_value(config).expect("typed config serializes");
    paths
        .into_iter()
        .map(|(label, path): (String, String)| {
            let value = json.pointer(&path).expect("known config path");
            (
                label,
                path,
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string()),
            )
        })
        .collect()
}
