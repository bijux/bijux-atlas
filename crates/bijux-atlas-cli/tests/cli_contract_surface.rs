// SPDX-License-Identifier: Apache-2.0

use assert_cmd::Command;

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
    assert!(text.contains(env!("CARGO_PKG_VERSION")));
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
