use std::path::PathBuf;

#[test]
fn api_crate_dependency_guardrails() {
    let cargo =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");

    for forbidden in ["tokio", "reqwest", "rusqlite", "bijux-atlas-store"] {
        assert!(
            !cargo.contains(forbidden),
            "forbidden dependency in api crate: {forbidden}"
        );
    }
}
