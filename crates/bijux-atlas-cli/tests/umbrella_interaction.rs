use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn write_executable(path: &Path, content: &str) {
    fs::write(path, content).expect("write script");
    let mut perms = fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod");
}

#[test]
fn umbrella_dispatches_to_bijux_atlas_plugin() {
    let temp = TempDir::new().expect("tempdir");
    let plugin_path = temp.path().join("bijux-atlas");
    fs::copy(env!("CARGO_BIN_EXE_bijux-atlas"), &plugin_path).expect("copy plugin binary");
    let mut perms = fs::metadata(&plugin_path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&plugin_path, perms).expect("chmod plugin");

    let umbrella = temp.path().join("bijux");
    write_executable(
        &umbrella,
        r##"#!/bin/sh
if [ $# -lt 1 ]; then
  echo "missing subsystem" >&2
  exit 2
fi
subsystem="$1"
shift
exec "$(dirname "$0")/bijux-$subsystem" "$@"
"##,
    );

    Command::new(&umbrella)
        .args(["atlas", "--bijux-plugin-metadata"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"bijux-atlas\""));

    Command::new(&umbrella)
        .args(["atlas", "atlas", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ingest"));

    Command::new(&umbrella)
        .args(["atlas", "version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bijux-atlas"));

    Command::new(&umbrella)
        .args(["atlas", "atlas", "print-config", "--canonical"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"cache_dir\""));

    Command::new(&umbrella)
        .args(["atlas", "completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("atlas"));

    Command::new(&umbrella)
        .args(["atlas", "--json", "atlas", "dev-atlas", "doctor"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "\"code\":\"legacy_command_redirect\"",
        ))
        .stderr(predicate::str::contains("bijux dev atlas <command>"));
}
