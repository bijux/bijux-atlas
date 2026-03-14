// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
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

fn run_output(program: &std::path::Path, args: &[&str]) -> Output {
    Command::new(program)
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("run command")
}

fn assert_same_output(left: &Output, right: &Output) {
    assert_eq!(left.status.code(), right.status.code(), "exit status mismatch");
    assert_eq!(left.stdout, right.stdout, "stdout mismatch");
    assert_eq!(left.stderr, right.stderr, "stderr mismatch");
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

#[test]
fn umbrella_dispatch_preserves_dev_atlas_results() {
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

    for args in [
        vec!["--help"],
        vec!["version", "--format", "json"],
        vec!["--bijux-plugin-metadata"],
        vec!["check", "list", "--json"],
    ] {
        let direct = run_output(&plugin_path, &args);
        let mut umbrella_args = vec!["dev", "atlas"];
        umbrella_args.extend(args.iter().copied());
        let dispatched = run_output(&umbrella, &umbrella_args);
        assert_same_output(&direct, &dispatched);
    }
}
