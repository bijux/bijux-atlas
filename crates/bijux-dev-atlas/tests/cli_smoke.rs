// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn slow_doctor_smoke() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "doctor", "--format", "json"])
        .output()
        .expect("doctor");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("json");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    let check_report = payload.get("check_report").expect("check_report");
    assert!(check_report
        .get("results")
        .and_then(|v| v.as_array())
        .is_some());
    assert!(check_report.get("counts").is_some());
}

#[test]
fn run_smoke() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "run", "--format", "text"])
        .output()
        .expect("run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() || !stdout.is_empty(),
        "stdout={stdout}\nstderr={stderr}"
    );
    assert!(stdout.contains("summary:") || stdout.contains("CI_SUMMARY"));
}

#[test]
fn help_snapshot_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("--help")
        .output()
        .expect("help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8");

    let golden_path = repo_root().join("crates/bijux-dev-atlas/tests/goldens/help.txt");
    let golden = fs::read_to_string(&golden_path).expect("golden");
    assert_eq!(text, golden);
}

fn write_executable(path: &std::path::Path, content: &str) {
    fs::write(path, content).expect("write script");
    let mut perms = fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod");
}

#[test]
fn dev_atlas_help_command_list_matches_doc() {
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
                if trimmed.is_empty() || trimmed.starts_with("Options:") {
                    break;
                }
                let cmd = trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                if !cmd.is_empty() {
                    out.push(cmd);
                }
            }
        }
        out
    }

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("--help")
        .output()
        .expect("dev help");
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).expect("utf8");
    let mut observed = parse_commands_from_help(&help);
    observed.sort();

    let mut expected =
        fs::read_to_string(repo_root().join("crates/bijux-dev-atlas/docs/cli-command-list.md"))
            .expect("dev command list")
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(observed, expected);
}

#[test]
fn dev_atlas_help_command_list_order_matches_doc_snapshot() {
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
                if trimmed.is_empty() || trimmed.starts_with("Options:") {
                    break;
                }
                let cmd = trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                if !cmd.is_empty() {
                    out.push(cmd);
                }
            }
        }
        out
    }

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("--help")
        .output()
        .expect("dev help");
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).expect("utf8");
    let observed = parse_commands_from_help(&help);

    let expected =
        fs::read_to_string(repo_root().join("crates/bijux-dev-atlas/docs/cli-command-list.md"))
            .expect("dev command list")
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
    assert_eq!(observed, expected);
}

#[test]
fn umbrella_dispatches_dev_atlas_help() {
    let temp = TempDir::new().expect("tempdir");
    let plugin_path = temp.path().join("bijux-dev-atlas");
    fs::copy(env!("CARGO_BIN_EXE_bijux-dev-atlas"), &plugin_path).expect("copy plugin binary");
    let mut perms = fs::metadata(&plugin_path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&plugin_path, perms).expect("chmod plugin");

    let umbrella = temp.path().join("bijux");
    write_executable(
        &umbrella,
        r##"#!/bin/sh
if [ "$1" = "dev" ] && [ "$2" = "atlas" ]; then
  shift 2
  exec "$(dirname "$0")/bijux-dev-atlas" "$@"
fi
echo "unsupported dispatch" >&2
exit 2
"##,
    );

    let output = Command::new(&umbrella)
        .args(["dev", "atlas", "--help"])
        .output()
        .expect("dev atlas via umbrella");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8");
    assert!(text.contains("bijux-dev-atlas"));
    assert!(text.contains("check"));
    assert!(text.contains("check"));
}
