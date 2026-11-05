//! Owner-editable drafting briefs, never assertions about candidate experience.
use crate::{ai::Fact, config::digest};
use anyhow::{Result, ensure};
use radar_domain::Job;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalTemplate {
    pub id: String,
    pub name: String,
    pub use_when: String,
    pub instructions: String,
    pub mock_approach: String,
    pub question: String,
    pub max_words: usize,
    pub max_evidence: usize,
    pub max_questions: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalTemplates {
    pub default_id: Option<String>,
    pub items: Vec<ProposalTemplate>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSnapshot {
    pub revision: String,
    pub definition: ProposalTemplate,
}
impl ProposalTemplates {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (1..=20).contains(&self.items.len()),
            "template catalog must contain one to twenty entries"
        );
        let mut ids = BTreeSet::new();
        for template in &self.items {
            ensure!(
                !template.id.is_empty()
                    && template.id.len() <= 64
                    && template
                        .id
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
                    && ids.insert(&template.id),
                "invalid/duplicate template ID"
            );
            ensure!(
                (60..=300).contains(&template.max_words)
                    && (1..=2).contains(&template.max_evidence)
                    && template.max_questions <= 2,
                "invalid template bounds"
            );
            for text in [
                &template.name,
                &template.use_when,
                &template.instructions,
                &template.mock_approach,
                &template.question,
            ] {
                ensure!(
                    !text.trim().is_empty()
                        && text.len() <= 3000
                        && !text.chars().any(|c| c.is_control() && c != '\n'),
                    "invalid template text"
                );
            }
        }
        ensure!(
            self.default_id.as_ref().is_none_or(|id| ids.contains(id)),
            "default proposal template unavailable"
        );
        Ok(())
    }
    pub fn select(&self, id: Option<&str>) -> Result<Option<TemplateSnapshot>> {
        let Some(id) = id.or(self.default_id.as_deref()) else {
            return Ok(None);
        };
        let definition = self
            .items
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| anyhow::anyhow!("proposal template unavailable"))?
            .clone();
        Ok(Some(TemplateSnapshot {
            revision: digest(&serde_json::to_vec(&definition)?),
            definition,
        }))
    }
}
/// Deterministic relevance ordering uses supplied job vocabulary, not invented experience.
pub fn relevant_evidence(job: &Job, facts: &[Fact], limit: usize) -> Vec<String> {
    let terms: BTreeSet<_> = job
        .skills
        .iter()
        .chain(std::iter::once(&job.title))
        .chain(std::iter::once(&job.category))
        .flat_map(|v| v.split(|c: char| !c.is_alphanumeric()))
        .filter(|s| s.len() >= 3)
        .map(str::to_lowercase)
        .collect();
    let mut ranked: Vec<_> = facts
        .iter()
        .filter(|f| f.verified && f.shareable)
        .map(|f| {
            let text = format!("{} {}", f.id, f.statement).to_lowercase();
            let words: BTreeSet<_> = text.split(|c: char| !c.is_alphanumeric()).collect();
            let score = terms.iter().filter(|t| words.contains(t.as_str())).count();
            (score, f.id.clone())
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    ranked
        .into_iter()
        .filter(|(score, _)| *score > 0)
        .take(limit)
        .map(|(_, id)| id)
        .collect()
}
