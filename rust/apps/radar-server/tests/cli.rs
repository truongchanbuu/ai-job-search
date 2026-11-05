#[path = "../../../fixtures/config.rs"]
mod config_fixture;
use std::{fs, path::PathBuf, process::Command};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn binary(config: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_radar-server"));
    command.env("RADAR_CONFIG_DIR", config);
    command
}

#[test]
fn demo_round_trips_ranked_jobs_snapshots_and_duplicate_history() {
    let config = config_fixture::directory();
    let dir = tempfile::tempdir().unwrap();
    let export = dir.path().join("snapshot.json");
    let output = binary(config.path())
        .args(["demo", "--at", "2026-10-30T12:00:00Z", "--output"])
        .arg(&export)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SYNTHETIC fixture"));
    assert!(stdout.contains("processed: 7 | canonical jobs: 6"));
    assert!(stdout.contains("1. ApplyCandidate score=100.00"));
    let snapshot: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export).unwrap()).unwrap();
    let records = snapshot["report"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 6);
    assert!(
        records
            .iter()
            .any(|r| r["ingestions"].as_array().unwrap().len() == 2)
    );
    assert_eq!(
        records[0]["evaluation"]["config_revision"],
        snapshot["config"]["revision"]
    );
    assert_eq!(
        records[0]["evaluation"]["profile_revision"],
        snapshot["profile"]["revision"]
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&export).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let before = fs::read(&export).unwrap();
    let repeated = binary(config.path())
        .args(["demo", "--output"])
        .arg(&export)
        .output()
        .unwrap();
    assert!(!repeated.status.success());
    assert_eq!(fs::read(&export).unwrap(), before);
}

#[test]
fn manual_import_and_private_profile_directory_work_without_credentials() {
    let config = config_fixture::directory();
    let output = binary(config.path())
        .args(["demo", "--profile-dir"])
        .arg(root().join("rust/fixtures/profiles"))
        .arg("--manual")
        .arg(root().join("rust/fixtures/manual-jobs.json"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("canonical jobs: 1")
    );
}

#[test]
fn paused_config_prevents_import_and_export_contains_no_jobs() {
    let config = config_fixture::directory();
    let dir = tempfile::tempdir().unwrap();
    for name in ["radar", "scoring", "notifications"] {
        let text = fs::read_to_string(root().join(format!("config/{name}.example.yaml"))).unwrap();
        fs::write(
            dir.path().join(format!("{name}.yaml")),
            text.replace("active_mode: employed", "active_mode: paused"),
        )
        .unwrap();
    }
    let export = dir.path().join("paused.json");
    let output = binary(config.path())
        .args(["demo", "--config-dir"])
        .arg(dir.path())
        .arg("--output")
        .arg(&export)
        .output()
        .unwrap();
    assert!(output.status.success());
    let snapshot: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(export).unwrap()).unwrap();
    assert_eq!(snapshot["report"]["records"], serde_json::json!([]));
    assert!(
        snapshot["report"]["status"]
            .as_str()
            .unwrap()
            .starts_with("paused")
    );
}

#[test]
fn bad_config_or_missing_profile_is_a_visible_error() {
    let config = config_fixture::directory();
    let dir = tempfile::tempdir().unwrap();
    let output = binary(config.path())
        .args(["demo", "--config-dir"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("reading"));
    let output = binary(config.path())
        .args(["demo", "--profile-dir"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("profile module"));
}

#[test]
fn upwork_login_loads_plain_config_names_before_checking_private_state() {
    let config = config_fixture::directory();
    let output = binary(std::path::Path::new("/nonexistent-radar-config"))
        .args(["upwork", "--config-dir"])
        .arg(config.path())
        .arg("login")
        .env_remove("RADAR_UPWORK_STATE_DIR")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("state_not_configured"), "{error}");
    assert!(!error.contains("reading"), "{error}");
}

#[test]
fn database_connection_timeout_is_validated_before_connecting() {
    let config = config_fixture::directory();
    for value in ["0", "121", "invalid"] {
        let output = binary(config.path())
            .arg("migrate")
            .env(
                "RADAR_DATABASE_URL",
                "postgres://synthetic@127.0.0.1:1/synthetic",
            )
            .env("RADAR_DATABASE_ACQUIRE_TIMEOUT_SECONDS", value)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(
                "RADAR_DATABASE_ACQUIRE_TIMEOUT_SECONDS must be an integer from 1 to 120"
            ),
            "{error}"
        );
        assert!(!error.contains("synthetic@"));
    }
}
