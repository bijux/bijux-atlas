// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn observe_dashboards_list_returns_registry() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "dashboards", "list", "--format", "json"])
        .output()
        .expect("observe dashboards list");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload["kind"],
        serde_json::json!("observe_dashboards_list")
    );
    assert!(payload["registry"]["dashboards"].is_array());
}

#[test]
fn observe_dashboards_explain_returns_metadata_schema() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "dashboards", "explain", "--format", "json"])
        .output()
        .expect("observe dashboards explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload["kind"],
        serde_json::json!("observe_dashboards_explain")
    );
    assert!(payload["metadata_schema"].is_object());
}

#[test]
fn observe_dashboards_verify_generates_operational_artifacts() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["observe", "dashboards", "verify", "--format", "json"])
        .output()
        .expect("observe dashboards verify");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload["kind"],
        serde_json::json!("observe_dashboards_verify")
    );
    for rel in [
        "artifacts/observe/dashboard-coverage-report.json",
        "artifacts/observe/dashboard-health-summary.json",
        "artifacts/observe/operational-readiness-report.json",
        "artifacts/observe/operational-telemetry-summary.json",
    ] {
        let file = root.join(rel);
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(file).expect("read artifact"))
                .expect("parse artifact");
        assert_eq!(value["schema_version"], serde_json::json!(1));
    }
}

#[test]
fn dashboard_registry_entries_reference_existing_files_and_metrics() {
    let root = repo_root();
    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/observe/dashboard-registry.json"))
            .expect("read registry"),
    )
    .expect("parse registry");
    let dashboards = registry["dashboards"].as_array().expect("dashboards list");
    for row in dashboards {
        let rel = row["path"].as_str().expect("path");
        let dashboard: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(rel)).expect("read dashboard"))
                .expect("parse dashboard");
        let panels = dashboard["panels"].as_array().expect("panels");
        assert!(!panels.is_empty());
        for panel in panels {
            if let Some(targets) = panel.get("targets").and_then(|v| v.as_array()) {
                for target in targets {
                    let expr = target["expr"].as_str().expect("expr");
                    assert!(
                        expr.contains("atlas_")
                            || expr.contains("bijux_")
                            || expr.contains("rate(")
                            || expr.contains("sum by"),
                        "unexpected metric expression: {expr}"
                    );
                }
            }
        }
    }
}
