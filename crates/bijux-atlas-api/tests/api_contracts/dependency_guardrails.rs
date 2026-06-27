// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn api_module_dependency_guardrails() {
    let api_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(&api_root).expect("read bijux-atlas-api/src") {
        let entry = entry.expect("api source entry");
        let path = entry.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
            sources.push(std::fs::read_to_string(path).expect("read api source"));
        }
    }
    let joined = sources.join("\n");

    for forbidden in ["tokio::", "reqwest::blocking", "rusqlite::", "crate::store"] {
        assert!(
            !joined.contains(forbidden),
            "forbidden dependency in api module: {forbidden}"
        );
    }
}

#[test]
fn api_test_surface_stays_contract_owned() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = std::fs::read_to_string(crate_root.join("Cargo.toml")).expect("Cargo.toml");

    for forbidden in [
        "bijux-atlas-runtime",
        "bijux-atlas-server",
        "axum =",
        "tokio =",
    ] {
        assert!(
            !cargo_toml.contains(forbidden),
            "api crate must not depend on server or runtime test harness surface: {forbidden}"
        );
    }

    for forbidden in [
        "tests/http_contracts.rs",
        "tests/http_surface.rs",
        "tests/http_observability.rs",
    ] {
        assert!(
            !crate_root.join(forbidden).exists(),
            "api crate must not own server-backed HTTP integration suites: {forbidden}"
        );
    }
}
