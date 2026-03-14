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

fn parse_markdown_command_catalog(path: &PathBuf) -> Vec<String> {
    let text = fs::read_to_string(path).expect("read catalog");
    let mut commands = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("- `") && trimmed.ends_with('`')) {
            continue;
        }
        let command = trimmed
            .trim_start_matches("- `")
            .trim_end_matches('`')
            .to_string();
        commands.push(command);
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
fn user_cli_commands_match_documented_catalog() {
    let root = repo_root();
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let observed = parse_help_commands(&String::from_utf8_lossy(&out.stdout));
    let catalog_path = root.join("docs/architecture/user-cli-command-catalog.md");
    let documented = parse_markdown_command_catalog(&catalog_path);
    assert_eq!(
        observed,
        documented,
        "user CLI command list must stay in sync with {}",
        catalog_path.display()
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

#[test]
fn docs_must_reference_dev_runtime_self_check_command() {
    let root = repo_root();
    let docs_paths = [
        root.join("docs/operations/ghcr-runtime-usage.md"),
        root.join("docs/operations/image-offline-usage.md"),
        root.join("docs/operations/image-upgrade-guide.md"),
    ];
    for path in docs_paths {
        let text = fs::read_to_string(&path).expect("read docs file");
        assert!(
            text.contains("bijux-dev-atlas runtime self-check"),
            "{} must reference `bijux-dev-atlas runtime self-check`",
            path.display()
        );
    }
}
