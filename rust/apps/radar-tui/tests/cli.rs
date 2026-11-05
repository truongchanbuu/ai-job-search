use std::process::Command;

#[test]
fn request_timeout_is_validated_before_contacting_the_api() {
    for timeout in ["0", "121", "invalid"] {
        let output = Command::new(env!("CARGO_BIN_EXE_radar-tui"))
            .args([
                "--api-url",
                "http://127.0.0.1:1",
                "--allow-local-http",
                "--check",
            ])
            .env("RADAR_API_TOKEN", "synthetic-test-token")
            .env("RADAR_HTTP_TIMEOUT_SECONDS", timeout)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("RADAR_HTTP_TIMEOUT_SECONDS must be an integer from 1 to 120")
        );
    }
}
