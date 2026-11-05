use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use radar_adapters::{
    FixedClock, ManualJobSource, MockJobSource, SystemClock, config::Config, load_profile,
};
use radar_application::{Clock, JobSource, MemoryRepository, scan};
use std::{fs, io::Write, path::PathBuf};

pub fn run(
    root: PathBuf,
    config_dir: Option<PathBuf>,
    profile_dir: Option<PathBuf>,
    manual: Option<PathBuf>,
    at: Option<DateTime<Utc>>,
    output: Option<PathBuf>,
) -> Result<()> {
    let config = Config::load(
        &config_dir
            .or_else(|| std::env::var_os("RADAR_CONFIG_DIR").map(PathBuf::from))
            .unwrap_or_else(|| root.join("private/config")),
    )?;
    let synthetic = profile_dir.is_none();
    let profile =
        load_profile(&profile_dir.unwrap_or_else(|| root.join("rust/fixtures/profiles")))?;
    let clock: Box<dyn Clock> = match at {
        Some(at) => Box::new(FixedClock(at)),
        None => Box::new(SystemClock),
    };
    let settings = config.settings();
    let mut source: Box<dyn JobSource> = match manual {
        Some(path) => Box::new(ManualJobSource::load(
            &path,
            config.radar.job_sources.manual.enabled,
        )?),
        None => Box::new(MockJobSource::bundled(
            config.radar.job_sources.mock.enabled,
        )?),
    };
    let mut repository = MemoryRepository::default();
    let report = scan(
        source.as_mut(),
        &mut repository,
        clock.as_ref(),
        &settings,
        &profile,
        &config.policy(),
    )?;
    println!(
        "Profile: {} | mode: {:?}",
        if synthetic {
            "SYNTHETIC fixture"
        } else {
            "private Markdown"
        },
        settings.mode
    );
    println!(
        "upwork: disabled (authorized Phase 4 POC required) | AI: mock | notifications: mock only, no external delivery"
    );
    println!(
        "{} | processed: {} | canonical jobs: {}",
        report.status,
        report.processed,
        report.records.len()
    );
    for (i, record) in report.records.iter().enumerate() {
        let job = &record
            .ingestions
            .last()
            .context("record without ingestion")?
            .job;
        let e = &record.evaluation;
        println!(
            "{}. {:?} score={} confidence={:?} risk={} | {}",
            i + 1,
            e.verdict,
            e.score
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "veto".into()),
            e.confidence,
            e.risk,
            job.title
        );
        println!(
            "   {} | fetched={} | vetoes={:?} unknowns={:?}",
            job.url,
            record.ingestions.last().unwrap().fetched_at,
            e.vetoes,
            e.unknowns
        );
        println!("   breakdown={:?} evidence={:?}", e.breakdown, e.evidence);
    }
    if let Some(path) = output {
        let parent = path.parent().context("output path needs a directory")?;
        fs::create_dir_all(parent).context("creating export directory")?;
        let parent = parent.canonicalize()?;
        ensure!(
            !parent.starts_with(&root)
                || parent.starts_with(root.join("private"))
                || parent.starts_with(root.join("radar-output")),
            "exports inside the repository must be under private/ or radar-output/"
        );
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&path)
            .context("creating private export (existing files are never overwritten)")?;
        let snapshot = serde_json::json!({ "schema_version": 1, "config": config, "profile": profile, "report": report, "repository": repository });
        file.write_all(serde_json::to_string_pretty(&snapshot)?.as_bytes())?;
        writeln!(file)?;
        println!(
            "Exported snapshot: {} (local export; no automatic restart recovery)",
            path.display()
        );
    }
    Ok(())
}
