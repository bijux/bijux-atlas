// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(args)
        .output()
        .expect("run bijux-dev-atlas")
}

#[test]
fn docker_contracts_lists_contract_ids() {
    let output = run(&["docker", "contracts", "--format", "json"]);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("parse json");
    let rows = payload["rows"].as_array().expect("rows");
    assert!(
        rows.iter().any(|row| row["contract_id"] == "DOCKER-001"),
        "missing DOCKER-001 in docker contracts output"
    );
}

#[test]
fn docker_gates_lists_gate_ids() {
    let output = run(&["docker", "gates", "--format", "json"]);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("parse json");
    let rows = payload["rows"].as_array().expect("rows");
    assert!(
        rows.iter()
            .any(|row| row["gate_id"] == "docker.contract.no_latest"),
        "missing docker.contract.no_latest in docker gates output"
    );
}

#[test]
fn docker_scan_requires_network_permission() {
    let output = run(&["docker", "scan", "--allow-subprocess", "--format", "json"]);
    assert!(
        !output.status.success(),
        "docker scan must fail without --allow-network"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("docker scan requires --allow-network"),
        "unexpected stderr: {stderr}"
    );
}
