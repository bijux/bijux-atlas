// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn api_module_dependency_guardrails() {
    let api_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/api");
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(&api_root).expect("read src/api") {
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
