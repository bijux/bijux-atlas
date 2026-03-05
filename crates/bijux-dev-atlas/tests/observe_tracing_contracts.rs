// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;
use std::{fs, path::Path};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn observe_traces_explain_emits_tracing_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "traces", "explain", "--format", "json"])
        .output()
        .expect("observe traces explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("observe_traces_explain")
    );
    let spans = payload
        .get("contract")
        .and_then(|v| v.get("span_registry"))
        .and_then(|v| v.as_array())
        .expect("span registry array");
    assert!(spans.len() >= 10);
}

#[test]
fn observe_traces_verify_runs_integrity_checks() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "traces", "verify", "--format", "json"])
        .output()
        .expect("observe traces verify");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("observe_traces_verify")
    );
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
}

#[test]
fn observe_traces_coverage_emits_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "traces", "coverage", "--format", "json"])
        .output()
        .expect("observe traces coverage");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("observe_traces_coverage")
    );
}

#[test]
fn observe_traces_topology_emits_artifact() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "traces", "topology", "--format", "json"])
        .output()
        .expect("observe traces topology");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("observe_traces_topology")
    );
}

#[test]
fn tracing_stability_contract_covers_all_required_ids() {
    let root = repo_root();
    let path = root.join("ops/observe/contracts/tracing-stability-contract.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read tracing stability contract"))
            .expect("parse tracing stability contract");
    let ids = value
        .get("stable_trace_ids")
        .and_then(|v| v.as_array())
        .expect("stable_trace_ids");
    assert!(ids.len() >= 10);
}

#[test]
fn simulated_trace_workload_fixture_is_valid_json() {
    let path =
        repo_root().join("crates/bijux-dev-atlas/tests/fixtures/trace/simulated-workload.json");
    assert!(Path::new(&path).exists());
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("fixture read"))
            .expect("fixture parse");
    assert_eq!(
        value.get("kind").and_then(|v| v.as_str()),
        Some("trace_simulated_workload")
    );
}
