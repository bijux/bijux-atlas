// SPDX-License-Identifier: Apache-2.0

#[test]
fn planner_module_is_pure_and_db_free() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let planner = std::fs::read_to_string(root.join("src/planner/mod.rs")).expect("read planner");
    for forbidden in ["rusqlite", "Connection", "std::fs", "reqwest", "tokio"] {
        assert!(
            !planner.contains(forbidden),
            "forbidden dependency in planner module: {forbidden}"
        );
    }
}

#[test]
fn query_crate_cargo_has_no_axum_or_server_dependency() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
    for forbidden in ["axum", "bijux-atlas-server"] {
        assert!(
            !cargo_toml.contains(forbidden),
            "forbidden dependency in query crate: {forbidden}"
        );
    }
}
