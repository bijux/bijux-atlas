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

fn parse_help_commands(text: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_commands = false;
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed == "Commands:" {
            in_commands = true;
            continue;
        }
        if in_commands {
            if trimmed.is_empty() || trimmed == "Options:" {
                break;
            }
            let entry = trimmed.trim_start();
            let name = entry.split_whitespace().next().unwrap_or("");
            if !name.is_empty() && name != "help" {
                commands.push(name.to_string());
            }
        }
    }
    commands.sort();
    commands
}

#[test]
fn dev_cli_help_contains_repository_operations() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    for command in ["checks", "contract", "docs", "governance", "runtime"] {
        assert!(
            text.contains(command),
            "dev CLI help must include repository command `{command}`"
        );
    }
}

#[test]
fn dev_cli_commands_match_governance_surface_registry() {
    let root = repo_root();
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let observed = parse_help_commands(&String::from_utf8_lossy(&out.stdout));
    let config_path = root.join("configs/governance/cli-dev-command-surface.json");
    let config_text = fs::read_to_string(&config_path).expect("read cli surface config");
    let config_json: serde_json::Value = serde_json::from_str(&config_text).expect("parse config");
    let mut expected: Vec<String> = config_json["top_level_commands"]
        .as_array()
        .expect("top_level_commands array")
        .iter()
        .filter_map(|v| v.as_str())
        .map(ToString::to_string)
        .collect();
    expected.sort();
    assert_eq!(
        observed,
        expected,
        "dev CLI command surface must match {}",
        config_path.display()
    );
}

#[test]
fn dev_cli_must_not_expose_user_runtime_flows() {
    let root = repo_root();
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    let config_text =
        fs::read_to_string(root.join("configs/governance/cli-dev-command-surface.json"))
            .expect("read cli surface config");
    let config_json: serde_json::Value = serde_json::from_str(&config_text).expect("parse config");
    for forbidden in config_json["forbidden_user_flow_commands"]
        .as_array()
        .expect("forbidden_user_flow_commands array")
    {
        let command = forbidden.as_str().expect("string command");
        assert!(
            !text.contains(command),
            "dev CLI help must not include user runtime flow `{command}`"
        );
    }
}

#[test]
fn runtime_code_must_not_depend_on_dev_atlas_crate() {
    let root = repo_root();
    let cargo_toml = fs::read_to_string(root.join("crates/bijux-atlas/Cargo.toml"))
        .expect("read runtime Cargo.toml");
    assert!(
        !cargo_toml.contains("bijux-dev-atlas"),
        "runtime crate must not depend on dev-atlas"
    );
}
