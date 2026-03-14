// SPDX-License-Identifier: Apache-2.0

use bijux_cli::contracts::known_bijux_tool;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use tempfile::TempDir;

static BUILD_ONCE: OnceLock<PathBuf> = OnceLock::new();

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn cargo_command() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn bijux_cli_manifest_path() -> PathBuf {
    let output = Command::new(cargo_command())
        .current_dir(repo_root())
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let packages = metadata["packages"].as_array().expect("packages");
    let manifest = packages
        .iter()
        .find(|package| package["name"].as_str() == Some("bijux-cli"))
        .and_then(|package| package["manifest_path"].as_str())
        .expect("bijux-cli manifest path");
    PathBuf::from(manifest)
}

fn bijux_cli_binary() -> PathBuf {
    BUILD_ONCE
        .get_or_init(|| {
            let target_dir = repo_root().join("artifacts/target/bijux-cli-runtime-contracts");
            let status = Command::new(cargo_command())
                .current_dir(repo_root())
                .arg("build")
                .arg("-q")
                .arg("--manifest-path")
                .arg(bijux_cli_manifest_path())
                .arg("--bin")
                .arg("bijux")
                .arg("--target-dir")
                .arg(&target_dir)
                .status()
                .expect("build bijux-cli binary");
            assert!(status.success(), "failed to build bijux-cli::bijux");
            target_dir.join("debug").join("bijux")
        })
        .clone()
}

fn copy_executable(source: &Path, destination: &Path) {
    fs::copy(source, destination).expect("copy executable");
    let mut permissions = fs::metadata(destination).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(destination, permissions).expect("chmod");
}

fn runtime_bin_dir() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    copy_executable(
        Path::new(env!("CARGO_BIN_EXE_bijux-atlas")),
        &temp.path().join("bijux-atlas"),
    );
    temp
}

fn path_with_runtime_binaries(runtime_bin_dir: &Path) -> std::ffi::OsString {
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(runtime_bin_dir.to_path_buf())
        .chain(std::env::split_paths(&current_path))
        .collect::<Vec<_>>();
    std::env::join_paths(paths).expect("join PATH")
}

fn run_output(program: &Path, args: &[&str], runtime_bin_dir: &Path) -> Output {
    Command::new(program)
        .current_dir(repo_root())
        .env("PATH", path_with_runtime_binaries(runtime_bin_dir))
        .args(args)
        .output()
        .expect("run command")
}

fn assert_same_output(left: &Output, right: &Output) {
    assert_eq!(
        left.status.code(),
        right.status.code(),
        "exit status mismatch"
    );
    assert_eq!(left.stdout, right.stdout, "stdout mismatch");
    assert_eq!(left.stderr, right.stderr, "stderr mismatch");
}

#[test]
fn atlas_namespace_stays_registered_with_bijux_cli() {
    let tool = known_bijux_tool("atlas").expect("atlas namespace");
    assert_eq!(tool.runtime_binary(), "bijux-atlas");
    assert_eq!(tool.control_binary(), "bijux-dev-atlas");
}

#[test]
fn bijux_cli_dispatch_matches_bijux_atlas_runtime() {
    let runtime_bin_dir = runtime_bin_dir();
    let atlas = runtime_bin_dir.path().join("bijux-atlas");
    let bijux = bijux_cli_binary();

    for args in [
        vec!["--help"],
        vec!["version"],
        vec!["print-config", "--canonical"],
        vec!["--json", "--bijux-plugin-metadata"],
    ] {
        let direct = run_output(&atlas, &args, runtime_bin_dir.path());
        let mut umbrella_args = vec!["atlas"];
        umbrella_args.extend(args.iter().copied());
        let delegated = run_output(&bijux, &umbrella_args, runtime_bin_dir.path());
        assert_same_output(&direct, &delegated);
    }
}
