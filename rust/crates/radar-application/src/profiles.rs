//! Private owner-authored Markdown modules; no inference or file writes.
use radar_domain::Profile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MODULES: [&str; 7] = [
    "identity",
    "mindset",
    "availability",
    "job-preferences",
    "portfolio",
    "proposal-style",
    "lessons-learned",
];
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileUpdate {
    pub expected_revision: i64,
    pub documents: BTreeMap<String, String>,
    pub confirmed_facts: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileVersion {
    pub revision: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileView {
    pub revision: i64,
    pub source: String,
    pub profile: Profile,
    pub evidence: crate::ai::EvidenceSnapshot,
    pub versions: Vec<ProfileVersion>,
}
