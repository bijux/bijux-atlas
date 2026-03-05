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
fn drift_fixtures_exist_and_are_parseable() {
    let fixtures = [
        "ops/drift/fixtures/modified-config-file.json",
        "ops/drift/fixtures/missing-artifact.json",
        "ops/drift/fixtures/altered-registry-entry.json",
        "ops/drift/fixtures/altered-helm-values.json",
        "ops/drift/fixtures/modified-profile.json",
    ];
    for rel in fixtures {
        let path = repo_root().join(rel);
        let text = fs::read_to_string(&path).expect("read fixture");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        assert_eq!(
            value.get("fixture_id").and_then(|v| v.as_str()).is_some(),
            true,
            "fixture_id missing in {rel}"
        );
        assert_eq!(
            value.get("expected_drift_type").and_then(|v| v.as_str()).is_some(),
            true,
            "expected_drift_type missing in {rel}"
        );
    }
}
