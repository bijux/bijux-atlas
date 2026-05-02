// SPDX-License-Identifier: Apache-2.0

use assert_cmd::Command;
use tempfile::tempdir;

fn parse_commands_from_help(text: &str) -> Vec<String> {
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
fn help_command_surface_is_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 help");
    let observed = parse_commands_from_help(&text);
    let expected = include_str!("snapshots/help.commands.txt")
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(observed, expected);
}

#[test]
fn version_output_contains_crate_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("version")
        .output()
        .expect("run version");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 version output");
    assert!(text.contains(bijux_atlas::version::runtime_version()));
}

#[test]
fn unknown_flag_returns_usage_exit_code_with_machine_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "--unknown-flag"])
        .output()
        .expect("run bad cli");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("usage_error"));
}

#[test]
fn missing_query_database_returns_dependency_failure_exit_code() {
    let tmp = tempdir().expect("tempdir");
    let missing_db = tmp.path().join("missing-dir").join("db.sqlite");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "query", "run", "--db"])
        .arg(&missing_db)
        .args(["--gene-id", "gene1"])
        .output()
        .expect("run query with missing db");
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("dependency_failure"));
}

#[test]
fn query_refusal_returns_validation_exit_code() {
    let tmp = tempdir().expect("tempdir");
    let db_path = tmp.path().join("empty.sqlite");
    rusqlite::Connection::open(&db_path).expect("create sqlite file");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "query", "run", "--db"])
        .arg(&db_path)
        .output()
        .expect("run query refusal case");
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("validation_error"));
}
