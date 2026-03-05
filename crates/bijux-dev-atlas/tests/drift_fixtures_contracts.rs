// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(workspace_root) = manifest_dir.parent() else {
        panic!(
            "missing workspace root for manifest dir: {}",
            manifest_dir.display()
        );
    };
    let Some(repo_root) = workspace_root.parent() else {
        panic!(
            "missing repository root for workspace dir: {}",
            workspace_root.display()
        );
    };
    repo_root.to_path_buf()
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
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => panic!("failed to read fixture {}: {error}", path.display()),
        };
        let value: serde_json::Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(error) => panic!("failed to parse fixture {}: {error}", path.display()),
        };
        assert!(
            value.get("fixture_id").and_then(|v| v.as_str()).is_some(),
            "fixture_id missing in {rel}"
        );
        assert!(
            value
                .get("expected_drift_type")
                .and_then(|v| v.as_str())
                .is_some(),
            "expected_drift_type missing in {rel}"
        );
    }
}
