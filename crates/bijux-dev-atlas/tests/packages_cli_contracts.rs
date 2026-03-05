// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn packages_list_reports_python_package_inventory() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["packages", "list", "--format", "json"])
        .output()
        .expect("run packages list");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json payload");
    assert_eq!(payload["domain"], "packages");
    assert_eq!(payload["action"], "list");
    assert_eq!(
        payload["packages"][0]["path"],
        "packages/bijux-atlas-python"
    );
}

#[test]
fn packages_verify_passes_for_current_layout() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["packages", "verify", "--format", "json"])
        .output()
        .expect("run packages verify");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json payload");
    assert_eq!(payload["domain"], "packages");
    assert_eq!(payload["action"], "verify");
    assert_eq!(payload["success"], true);
}
