//! Never enabled by ordinary CI or database integration tests.
#![cfg(feature = "live-upwork-tests")]
use radar_adapters::{
    config::Config,
    upwork::{OfficialUpwork, UpworkGateway},
};
#[tokio::test]
#[ignore = "requires Radar-owned OAuth, reviewed binding and RADAR_TEST_UPWORK_LIVE=1"]
async fn official_upwork_live_read_only_smoke() {
    assert_eq!(
        std::env::var("RADAR_TEST_UPWORK_LIVE").as_deref(),
        Ok("1"),
        "explicit live opt-in required"
    );
    let dir = std::env::var_os("RADAR_CONFIG_DIR").expect("private config directory required");
    let config = Config::load(std::path::Path::new(&dir)).unwrap();
    assert!(config.radar.job_sources.upwork_mcp.enabled);
    let settings = config.radar.job_sources.upwork_mcp.runtime.unwrap();
    let result = OfficialUpwork
        .search(&settings, &settings.default_profile, 1)
        .await
        .unwrap();
    assert!(result.jobs.iter().all(|j| j.provider == "upwork"));
    // Empty matches are legitimate; this test neither writes marketplace state nor touches PostgreSQL.
    assert!(result.stats.pages > 0);
}

/// Inspect one real response before mapping its fields; never guesses result pointers.
#[tokio::test]
#[ignore = "requires RADAR_TEST_UPWORK_SAMPLE=1, Radar OAuth and private output/parameters paths"]
async fn official_upwork_live_read_only_sample() {
    use radar_adapters::upwork::{
        Account, Discovery,
        mcp::{McpSession, OfficialSession},
        state::PrivateState,
    };
    use serde_json::json;
    use std::{io::Write, sync::Arc};
    assert_eq!(
        std::env::var("RADAR_TEST_UPWORK_SAMPLE").as_deref(),
        Ok("1")
    );
    let dir = std::env::var_os("RADAR_CONFIG_DIR").expect("private config directory required");
    let config = Config::load(std::path::Path::new(&dir)).unwrap();
    let settings = config.radar.job_sources.upwork_mcp.runtime.unwrap();
    let state = Arc::new(PrivateState::from_settings(&settings).unwrap());
    let _guard = state.lock().unwrap();
    let account: Account = state
        .read("account.enc")
        .unwrap()
        .expect("run discovery first");
    let discovery: Discovery = state
        .read("discovery.enc")
        .unwrap()
        .expect("run discovery first");
    assert_eq!(discovery.help.get("read_only"), Some(&json!(true)));
    assert_eq!(account.role, "TALENT");
    assert!(!account.suspended);
    let output = std::env::var_os("RADAR_TEST_UPWORK_SAMPLE_OUTPUT")
        .expect("fresh private output path required");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut output = options.open(output).unwrap();
    let parameters = std::env::var_os("RADAR_TEST_UPWORK_SAMPLE_PARAMETERS")
        .expect("reviewed parameters file required");
    let params: serde_json::Value =
        serde_json::from_slice(&std::fs::read(parameters).unwrap()).unwrap();
    assert_eq!(params["limit"], 1);
    assert!(params.get("cursor").is_none());
    state
        .reserve(&settings, chrono::Utc::now().timestamp())
        .unwrap();
    let mut session = OfficialSession::connect(&settings, state.clone())
        .await
        .unwrap();
    let tools = session.tools().await.unwrap();
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "upwork__find_jobs" || tool["name"] == "find_jobs")
        .expect("search tool required");
    let args = json!({"action":"search","org_uid":account.org_uid,"params":params});
    assert!(
        jsonschema::validator_for(&tool["inputSchema"])
            .unwrap()
            .is_valid(&args)
    );
    assert!(
        jsonschema::validator_for(&discovery.help["input_schema"])
            .unwrap()
            .is_valid(&args)
    );
    state
        .reserve(&settings, chrono::Utc::now().timestamp())
        .unwrap();
    let result = session.call(tool["name"].as_str().unwrap(), args).await;
    session.close().await;
    let result = result.unwrap();
    output
        .write_all(&serde_json::to_vec_pretty(&result).unwrap())
        .unwrap();
}
