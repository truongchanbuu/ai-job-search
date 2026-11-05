use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackAction {
    Interested,
    Save,
    Skip,
    NotAvailable,
    Applied,
    Interview,
    Hired,
    Rejected,
    Completed,
    Withdrawn,
    Other,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    #[default]
    None,
    Drafted,
    Submitted,
    Interviewing,
    Hired,
    Rejected,
    Completed,
    Withdrawn,
}

impl FeedbackAction {
    pub fn is_preference(self) -> bool {
        matches!(
            self,
            Self::Interested | Self::Save | Self::Skip | Self::NotAvailable | Self::Other
        )
    }
    pub fn excludes_recommendations(self) -> bool {
        matches!(self, Self::Skip | Self::NotAvailable)
    }
    pub fn application_status(
        self,
        current: ApplicationStatus,
        confirmed_submission: bool,
    ) -> Result<ApplicationStatus, &'static str> {
        use ApplicationStatus as A;
        if self.is_preference() {
            return Ok(current);
        }
        match (self, current) {
            (Self::Applied, A::None | A::Drafted) if confirmed_submission => Ok(A::Submitted),
            (Self::Applied, _) if !confirmed_submission => Err(
                "recording applied requires explicit human confirmation of an existing submission",
            ),
            (Self::Interview, A::Submitted) => Ok(A::Interviewing),
            (Self::Hired, A::Submitted | A::Interviewing) => Ok(A::Hired),
            (Self::Rejected, A::Submitted | A::Interviewing) => Ok(A::Rejected),
            (Self::Completed, A::Hired) => Ok(A::Completed),
            (Self::Withdrawn, A::Submitted | A::Interviewing) => Ok(A::Withdrawn),
            _ => Err("invalid application lifecycle transition"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn submission_is_human_reported_and_preferences_do_not_change_outcomes() {
        assert!(
            FeedbackAction::Applied
                .application_status(ApplicationStatus::None, false)
                .is_err()
        );
        assert_eq!(
            FeedbackAction::Applied
                .application_status(ApplicationStatus::None, true)
                .unwrap(),
            ApplicationStatus::Submitted
        );
        assert_eq!(
            FeedbackAction::Skip
                .application_status(ApplicationStatus::Interviewing, false)
                .unwrap(),
            ApplicationStatus::Interviewing
        );
        assert!(
            FeedbackAction::Completed
                .application_status(ApplicationStatus::None, false)
                .is_err()
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    TechnologyMismatch,
    TooDifficult,
    NotInteresting,
    BudgetTooLow,
    DeadlineUnrealistic,
    ScheduleConflict,
    ScopeUnclear,
    ClientRisk,
    TooCompetitive,
    BetterOpportunityAvailable,
    Other,
}
