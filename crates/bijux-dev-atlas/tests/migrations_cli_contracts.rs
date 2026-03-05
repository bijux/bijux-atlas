// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn fixture_root() -> PathBuf {
    repo_root().join("crates/bijux-dev-atlas/tests/fixtures/automation-boundary-violations")
}

#[test]
fn migrations_status_reports_clean_repository() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["migrations", "status", "--format", "json"])
        .output()
        .expect("run migrations status");
    assert!(
        output.status.success(),
        "migrations status should pass for repository; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("parse json");
    assert_eq!(payload["status"].as_str(), Some("ok"));
    assert_eq!(
        payload["summary"]["legacy_path_count"].as_u64(),
        Some(0),
        "migration status must report zero legacy paths"
    );
}

#[test]
fn migrations_status_reports_legacy_fixture_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "migrations",
            "status",
            "--repo-root",
            fixture_root().to_str().expect("utf8"),
            "--format",
            "json",
        ])
        .output()
        .expect("run migrations status fixture");
    assert!(
        !output.status.success(),
        "fixture should fail migration status; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let raw = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(raw).expect("parse json");
    let rows = payload["legacy_paths"]
        .as_array()
        .expect("legacy_paths array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    assert!(
        rows.iter().any(|row| row.contains("tools/")),
        "fixture must report legacy tools path: {rows:?}"
    );
}
