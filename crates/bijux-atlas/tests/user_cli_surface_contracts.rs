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
            if trimmed.is_empty() {
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
fn user_cli_does_not_expose_dev_runtime_diagnostics() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.contains("self-check"));
    assert!(!text.contains("print-config-schema"));
    assert!(!text.contains("runtime"));
}

#[test]
fn user_cli_commands_match_governance_surface_registry() {
    let root = repo_root();
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let observed = parse_help_commands(&String::from_utf8_lossy(&out.stdout));
    let config_path = root.join("configs/governance/cli-user-command-surface.json");
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
        "user CLI command surface must match {}",
        config_path.display()
    );
}

#[test]
fn user_cli_cargo_manifest_must_not_depend_on_dev_atlas() {
    let root = repo_root();
    let cargo_toml = fs::read_to_string(root.join("crates/bijux-atlas/Cargo.toml"))
        .expect("read user cli Cargo.toml");
    assert!(
        !cargo_toml.contains("bijux-dev-atlas"),
        "user CLI must not depend on dev-only crates"
    );
}
