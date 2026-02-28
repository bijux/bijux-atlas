// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn command_surface_ssot_matches_doc() {
    let expected = [
        "catalog validate",
        "catalog publish",
        "catalog rollback",
        "catalog promote",
        "catalog latest-alias-update",
        "completion",
        "dataset verify",
        "dataset validate",
        "dataset publish",
        "dataset pack",
        "dataset verify-pack",
        "diff build",
        "gc plan",
        "gc apply",
        "ingest",
        "openapi generate",
        "policy validate",
        "policy explain",
        "config",
        "version",
    ]
    .join("\n");

    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/CLI_COMMAND_LIST.md");
    let current = std::fs::read_to_string(path).expect("read CLI command list");
    assert_eq!(current.trim(), expected.trim());
}

#[test]
fn command_surface_contract_json_matches_doc() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let contract_path = root.join("docs/reference/contracts/schemas/CLI_COMMANDS.json");
    let contract: serde_json::Value =
        serde_json::from_slice(&std::fs::read(contract_path).expect("read cli contract"))
            .expect("parse cli contract");
    let commands = contract
        .get("commands")
        .and_then(|v| v.as_array())
        .expect("commands array")
        .iter()
        .map(|v| v.as_str().expect("string command").to_string())
        .collect::<Vec<_>>();
    let doc = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/CLI_COMMAND_LIST.md"),
    )
    .expect("read command list")
    .lines()
    .map(ToString::to_string)
    .collect::<Vec<_>>();
    assert_eq!(commands, doc);
}

#[test]
fn help_output_command_surface_matches_doc_exactly() {
    fn parse_commands_from_help(text: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut in_commands = false;
        for line in text.lines() {
            if line.trim() == "Commands:" {
                in_commands = true;
                continue;
            }
            if in_commands {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("Environment:") {
                    break;
                }
                let cmd = trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                if !cmd.is_empty() && cmd != "help" {
                    out.push(cmd);
                }
            }
        }
        out
    }

    let top = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("top help");
    assert!(top.status.success());
    let top_help = String::from_utf8(top.stdout).expect("utf8 top help");
    let top_cmds = parse_commands_from_help(&top_help);

    let mut observed = Vec::new();
    for sub in &top_cmds {
        if matches!(
            sub.as_str(),
            "catalog" | "dataset" | "diff" | "gc" | "policy" | "openapi"
        ) {
            let nested = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
                .args([sub, "--help"])
                .output()
                .expect("nested help");
            assert!(nested.status.success());
            let nested_help = String::from_utf8(nested.stdout).expect("utf8 nested help");
            for subsub in parse_commands_from_help(&nested_help) {
                observed.push(format!("{sub} {subsub}"));
            }
        } else {
            observed.push(sub.clone());
        }
    }
    observed.sort();

    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/CLI_COMMAND_LIST.md");
    let mut expected = std::fs::read_to_string(path)
        .expect("read command list")
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(observed, expected);
}

#[test]
fn help_template_includes_required_sections() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());
    let rendered = String::from_utf8(output.stdout).expect("utf8 help");
    for section in ["Usage:", "Options:", "Commands:", "Environment:"] {
        assert!(
            rendered.contains(section),
            "help output missing section `{section}`"
        );
    }
}

#[test]
fn top_level_subcommands_avoid_reserved_umbrella_verbs() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());
    let rendered = String::from_utf8(output.stdout).expect("utf8 help");
    for reserved in [" plugin", " plugins", " dev"] {
        assert!(
            !rendered.contains(reserved),
            "reserved verb exposed: {reserved}"
        );
    }
}
