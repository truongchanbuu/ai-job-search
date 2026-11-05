// Materialize committed templates as runtime configuration for isolated tests.
pub fn directory() -> tempfile::TempDir {
    let templates = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../config");
    let directory = tempfile::tempdir().unwrap();
    for name in ["radar", "scoring", "notifications"] {
        std::fs::copy(
            templates.join(format!("{name}.example.yaml")),
            directory.path().join(format!("{name}.yaml")),
        )
        .unwrap();
    }
    directory
}
