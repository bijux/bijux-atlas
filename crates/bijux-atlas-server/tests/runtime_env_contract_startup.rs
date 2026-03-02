// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn sanitized_server_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas-server"));
    command.current_dir(repo_root());
    for (name, _) in std::env::vars() {
        if name.starts_with("ATLAS_") || name.starts_with("BIJUX_") {
            command.env_remove(name);
        }
    }
    command
}

#[test]
fn startup_rejects_unknown_prefixed_env_before_binding_a_port() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_UNKNOWN_CONTRACT_BREAKER", "1")
        .output()
        .expect("run atlas-server");

    assert!(!output.status.success(), "unknown env must fail startup");
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        stderr.contains(
            "unknown env vars rejected by contract; set ATLAS_DEV_ALLOW_UNKNOWN_ENV=1 only for local dev override: ATLAS_UNKNOWN_CONTRACT_BREAKER"
        ),
        "stderr must contain the explicit contract failure:\n{stderr}"
    );
    assert!(
        !stderr.contains("atlas-server listening on"),
        "unknown env validation must fail before binding a port:\n{stderr}"
    );
    assert!(
        !stderr.contains("bind failed:"),
        "unknown env validation must fail before any bind attempt:\n{stderr}"
    );
}

#[test]
fn startup_accepts_an_allowlisted_env_surface() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_ENV", "dev")
        .env("ATLAS_LOG_JSON", "false")
        .env("ATLAS_REQUEST_TIMEOUT_MS", "5000")
        .output()
        .expect("run atlas-server");

    assert!(
        output.status.success(),
        "allowlisted env keys must pass startup validation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        !stderr.contains("unknown env vars rejected by contract"),
        "allowlisted startup env must not trip unknown-env validation:\n{stderr}"
    );
}

#[test]
fn startup_only_allows_unknown_prefixed_env_when_the_dev_override_is_explicitly_enabled() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_DEV_ALLOW_UNKNOWN_ENV", "1")
        .env("ATLAS_UNKNOWN_CONTRACT_BREAKER", "1")
        .output()
        .expect("run atlas-server");

    assert!(
        output.status.success(),
        "explicit dev override must permit unknown env keys in local validation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
