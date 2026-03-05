// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn reproducibility_fixtures_and_specs_are_parseable() {
    let files = [
        "ops/reproducibility/spec.json",
        "ops/reproducibility/report.schema.json",
        "ops/reproducibility/scenarios.json",
        "ops/reproducibility/evidence-integration.json",
        "ops/reproducibility/fixtures/reproducible-crate-build.json",
        "ops/reproducibility/fixtures/reproducible-chart-package.json",
        "ops/reproducibility/fixtures/reproducible-docs-build.json",
    ];
    for rel in files {
        let text = fs::read_to_string(repo_root().join(rel)).expect("read");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        if rel.ends_with("report.schema.json") {
            assert!(
                value
                    .get("properties")
                    .and_then(|v| v.get("schema_version"))
                    .is_some(),
                "schema_version property missing in {rel}"
            );
        } else {
            assert!(
                value.get("schema_version").is_some(),
                "schema_version missing in {rel}"
            );
        }
    }
}
