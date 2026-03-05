// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};
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

fn parse_markdown_command_catalog(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).expect("read catalog");
    let mut commands = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("- `") && trimmed.ends_with('`')) {
            continue;
        }
        commands.push(
            trimmed
                .trim_start_matches("- `")
                .trim_end_matches('`')
                .to_string(),
        );
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
fn dev_cli_commands_match_documented_catalog() {
    let root = repo_root();
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let observed = parse_help_commands(&String::from_utf8_lossy(&out.stdout));
    let catalog_path = root.join("docs/architecture/dev-cli-command-catalog.md");
    let documented = parse_markdown_command_catalog(&catalog_path);
    assert_eq!(
        observed,
        documented,
        "dev CLI command list must stay in sync with {}",
        catalog_path.display()
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
    let cargo_toml = fs::read_to_string(root.join("crates/bijux-atlas-server/Cargo.toml"))
        .expect("read runtime Cargo.toml");
    assert!(
        !cargo_toml.contains("bijux-dev-atlas"),
        "runtime crate must not depend on dev-atlas"
    );
}

#[test]
fn cli_command_migrations_must_have_documented_notes() {
    let root = repo_root();
    let migrations_path = root.join("configs/governance/cli-command-migrations.json");
    let migrations_text = fs::read_to_string(&migrations_path).expect("read migrations");
    let migrations: serde_json::Value =
        serde_json::from_str(&migrations_text).expect("parse migrations");
    let entries = migrations["entries"].as_array().expect("entries array");
    assert!(
        !entries.is_empty(),
        "command migration registry must not be empty"
    );
    for entry in entries {
        let note_rel = entry["migration_note"]
            .as_str()
            .expect("migration_note path");
        let note_path = root.join(note_rel);
        assert!(
            note_path.exists(),
            "migration note `{}` must exist for command `{}`",
            note_rel,
            entry["command"].as_str().unwrap_or("unknown")
        );
        let note_text = fs::read_to_string(&note_path).expect("read migration note");
        let command = entry["command"].as_str().expect("command string");
        assert!(
            note_text.contains(command),
            "migration note `{}` must mention command `{}`",
            note_rel,
            command
        );
    }
}

#[test]
fn docs_must_not_reference_user_cli_self_check_command() {
    let root = repo_root();
    let docs_root = root.join("docs");
    let mut violations = Vec::new();
    let mut stack = vec![docs_root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read docs dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("read markdown");
            if text.contains("bijux-atlas self-check")
                || text.contains("bijux-atlas print-config-schema")
            {
                let rel = path
                    .strip_prefix(&root)
                    .expect("relative path")
                    .display()
                    .to_string();
                violations.push(rel);
            }
        }
    }
    assert!(
        violations.is_empty(),
        "docs must not reference deprecated user CLI runtime diagnostics:\n{}",
        violations.join("\n")
    );
}
