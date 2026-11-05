pub mod ai;
pub mod bots;
pub mod config;
pub mod discord;
pub mod postgres;
pub mod upwork;

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use radar_application::{AiDraft, AiProvider, Capability, Clock, JobSource, Notification};
use radar_domain::{Identity, Job, Preferences, Profile};
use std::{collections::BTreeMap, fs, path::Path};

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
pub struct FixedClock(pub DateTime<Utc>);
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

pub fn load_profile(directory: &Path) -> Result<Profile> {
    let mut documents = BTreeMap::new();
    for name in [
        "identity",
        "mindset",
        "availability",
        "job-preferences",
        "portfolio",
        "proposal-style",
        "lessons-learned",
    ] {
        let path = directory.join(format!("{name}.md"));
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading profile module {}", path.display()))?;
        ensure!(!text.trim().is_empty(), "profile module {name} is empty");
        documents.insert(name.to_string(), text);
    }
    profile_from_documents(documents)
}

pub fn profile_from_documents(documents: BTreeMap<String, String>) -> Result<Profile> {
    use radar_application::profiles::MODULES;
    ensure!(
        documents.len() == MODULES.len()
            && MODULES.iter().all(|name| documents.contains_key(*name)),
        "exactly the seven approved profile modules are required"
    );
    ensure!(
        documents.values().all(|text| !text.trim().is_empty()
            && text.len() <= 32768
            && !text
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))),
        "profile modules must be nonempty text, at most 32 KiB each"
    );
    fn metadata<T: serde::de::DeserializeOwned>(text: &str, module: &str) -> Result<T> {
        let text = text.replace("\r\n", "\n");
        let (_, rest) = text
            .split_once("---\n")
            .filter(|(before, _)| before.is_empty())
            .with_context(|| {
                format!("{module}.md requires YAML front matter; see rust/fixtures/profiles")
            })?;
        let (yaml, _) = rest
            .split_once("\n---\n")
            .context("missing closing front matter delimiter")?;
        serde_yaml::from_str(yaml)
            .map_err(|_| anyhow::anyhow!("invalid typed metadata in {module}.md"))
    }
    let identity: Identity = metadata(&documents["identity"], "identity")?;
    let preferences: Preferences = metadata(&documents["job-preferences"], "job-preferences")?;
    ensure!(
        !identity.skills.is_empty()
            && identity.skills.iter().all(|(s, e)| !s.trim().is_empty()
                && *s == s.trim().to_lowercase()
                && !e.proof.trim().is_empty()),
        "identity skills require lowercase names and nonempty verified proof"
    );
    ensure!(
        preferences.minimum_hourly_usd.is_finite() && preferences.minimum_hourly_usd > 0.0,
        "minimum hourly USD must be positive"
    );
    let revision = config::digest(&serde_json::to_vec(&documents)?);
    let profile = Profile {
        revision,
        identity,
        preferences,
        documents,
    };
    radar_application::ai::evidence(&profile, 0)?;
    Ok(profile)
}

pub struct MockJobSource {
    pub jobs: Vec<Job>,
    pub enabled: bool,
}
impl MockJobSource {
    pub fn bundled(enabled: bool) -> Result<Self> {
        Ok(Self {
            jobs: serde_json::from_str(include_str!("../../../fixtures/jobs.json"))?,
            enabled,
        })
    }
}
impl JobSource for MockJobSource {
    fn name(&self) -> &str {
        "mock"
    }
    fn capability(&self) -> Capability {
        Capability {
            available: self.enabled,
            diagnostic: "synthetic source; enable job_sources.mock to import".into(),
        }
    }
    fn fetch(&mut self, limit: usize) -> Result<Vec<Job>> {
        Ok(self.jobs.iter().take(limit).cloned().collect())
    }
}

/// Local explicit import only. Import records must identify their provider as `manual`.
pub struct ManualJobSource {
    jobs: Vec<Job>,
    pub enabled: bool,
}
impl ManualJobSource {
    pub fn load(path: &Path, enabled: bool) -> Result<Self> {
        let text = fs::read_to_string(path).context("reading manual job JSON")?;
        let jobs: Vec<Job> = serde_json::from_str(&text)
            .map_err(|_| anyhow::anyhow!("invalid manual job JSON schema"))?;
        for job in &jobs {
            ensure!(
                job.provider == "manual",
                "manual import provider must be manual; original URLs retained"
            );
            job.validate().map_err(anyhow::Error::msg)?;
        }
        Self::from_jobs(jobs, enabled)
    }
    pub fn from_jobs(jobs: Vec<Job>, enabled: bool) -> Result<Self> {
        for job in &jobs {
            ensure!(
                job.provider == "manual",
                "manual import provider must be manual"
            );
            job.validate().map_err(anyhow::Error::msg)?;
        }
        Ok(Self { jobs, enabled })
    }
}
impl JobSource for ManualJobSource {
    fn name(&self) -> &str {
        "manual"
    }
    fn capability(&self) -> Capability {
        Capability {
            available: self.enabled,
            diagnostic: "manual file import disabled by configuration".into(),
        }
    }
    fn fetch(&mut self, limit: usize) -> Result<Vec<Job>> {
        Ok(self.jobs.iter().take(limit).cloned().collect())
    }
}

/// Deterministic, explicitly synthetic result for port-level tests; not a proposal workflow.
pub struct MockAiProvider;
#[async_trait::async_trait]
impl AiProvider for MockAiProvider {
    async fn infer(
        &self,
        _settings: &config::AiRuntime,
        input: &radar_application::ai::AiInput,
    ) -> Result<radar_application::ai::ProviderReply> {
        ai::mock(input)
    }
    fn draft(&self, _job: &Job, _profile: &Profile) -> Result<AiDraft> {
        Ok(AiDraft {
            text: "MOCK: no proposal generated or submitted".into(),
            evidence: vec![],
            estimated_cost_usd: 0.0,
        })
    }
}

#[derive(Default)]
pub struct MockNotifications {
    pub deliveries: BTreeMap<(String, String), Notification>,
}
impl MockNotifications {
    pub fn send(&mut self, destination: &str, event: &Notification) -> Result<String> {
        let key = (event.event_id.clone(), destination.into());
        self.deliveries.entry(key).or_insert_with(|| event.clone());
        Ok(format!("mock:{}:{}", event.event_id, destination))
    }
}
