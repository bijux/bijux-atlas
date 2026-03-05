// SPDX-License-Identifier: Apache-2.0

use std::process::Command;
use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn runtime_command_surface_exposes_schema_and_self_check() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["runtime", "--help"])
        .output()
        .expect("run runtime help");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    let config_text = fs::read_to_string(
        repo_root().join("configs/governance/cli-dev-command-surface.json"),
    )
    .expect("read dev cli governance surface");
    let config_json: serde_json::Value = serde_json::from_str(&config_text).expect("parse json");
    for command in config_json["runtime_required_subcommands"]
        .as_array()
        .expect("runtime_required_subcommands array")
    {
        let command = command.as_str().expect("string command");
        assert!(
            text.contains(command),
            "runtime help must include required subcommand `{command}`"
        );
    }
}

#[test]
fn runtime_self_check_returns_ok_payload() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["runtime", "self-check", "--format", "json"])
        .output()
        .expect("run self-check");
    assert!(out.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json payload");
    assert_eq!(payload["kind"], "runtime_self_check");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn runtime_help_matches_golden_snapshot() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["runtime", "--help"])
        .output()
        .expect("run runtime help");
    assert!(out.status.success());
    let observed = String::from_utf8(out.stdout).expect("utf8");
    let expected = fs::read_to_string(
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/runtime-help.txt"),
    )
    .expect("read runtime help snapshot");
    assert_eq!(observed, expected);
}
