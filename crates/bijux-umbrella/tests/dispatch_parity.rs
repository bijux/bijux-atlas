// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use tempfile::TempDir;

static BUILD_ONCE: OnceLock<()> = OnceLock::new();

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn build_product_binary(package: &str, binary: &str) {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(repo_root())
        .args(["build", "-q", "-p", package, "--bin", binary])
        .status()
        .expect("build binary");
    assert!(status.success(), "failed to build {package}::{binary}");
}

fn ensure_product_binaries() {
    BUILD_ONCE.get_or_init(|| {
        build_product_binary("bijux-atlas", "bijux-atlas");
        build_product_binary("bijux-dev-atlas", "bijux-dev-atlas");
    });
}

fn debug_binary(binary: &str) -> PathBuf {
    repo_root().join("artifacts/target/debug").join(binary)
}

fn copy_executable(source: &Path, destination: &Path) {
    fs::copy(source, destination).expect("copy executable");
    let mut permissions = fs::metadata(destination).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(destination, permissions).expect("chmod");
}

fn runtime_bin_dir() -> TempDir {
    ensure_product_binaries();

    let temp = TempDir::new().expect("tempdir");
    copy_executable(
        &debug_binary("bijux-atlas"),
        &temp.path().join("bijux-atlas"),
    );
    copy_executable(
        &debug_binary("bijux-dev-atlas"),
        &temp.path().join("bijux-dev-atlas"),
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
fn atlas_runtime_matches_umbrella_dispatch() {
    let runtime_bin_dir = runtime_bin_dir();
    let atlas = runtime_bin_dir.path().join("bijux-atlas");
    let umbrella = PathBuf::from(env!("CARGO_BIN_EXE_bijux"));

    for args in [
        vec!["--help"],
        vec!["version"],
        vec!["print-config", "--canonical"],
        vec!["--json", "--bijux-plugin-metadata"],
    ] {
        let direct = run_output(&atlas, &args, runtime_bin_dir.path());
        let mut umbrella_args = vec!["atlas"];
        umbrella_args.extend(args.iter().copied());
        let dispatched = run_output(&umbrella, &umbrella_args, runtime_bin_dir.path());
        assert_same_output(&direct, &dispatched);
    }
}

#[test]
fn dev_atlas_runtime_matches_spaced_umbrella_dispatch() {
    let runtime_bin_dir = runtime_bin_dir();
    let dev_atlas = runtime_bin_dir.path().join("bijux-dev-atlas");
    let umbrella = PathBuf::from(env!("CARGO_BIN_EXE_bijux"));

    for args in [
        vec!["--help"],
        vec!["version", "--format", "json"],
        vec!["--bijux-plugin-metadata"],
        vec!["check", "list", "--json"],
    ] {
        let direct = run_output(&dev_atlas, &args, runtime_bin_dir.path());
        let mut umbrella_args = vec!["dev", "atlas"];
        umbrella_args.extend(args.iter().copied());
        let dispatched = run_output(&umbrella, &umbrella_args, runtime_bin_dir.path());
        assert_same_output(&direct, &dispatched);
    }
}
