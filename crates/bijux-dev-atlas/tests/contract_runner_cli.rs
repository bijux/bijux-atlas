// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn effect_only_contract_skips_in_static_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contract",
            "run",
            "OPS-DATASET-001",
            "--mode",
            "static",
            "--output-format",
            "json",
        ])
        .output()
        .expect("run contract command");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json output");
    assert_eq!(json["counts"]["failed"].as_u64().unwrap_or(0), 0);
    assert!(json["counts"]["skipped"].as_u64().unwrap_or(0) > 0);
    assert!(json["cases"]
        .as_array()
        .into_iter()
        .flatten()
        .all(|case| case["status"].as_str() == Some("skip")));
}
