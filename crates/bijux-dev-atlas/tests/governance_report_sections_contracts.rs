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
fn governance_report_includes_automation_purity_sections() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["governance", "report", "--format", "json"])
        .output()
        .expect("run governance report");
    let raw = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(raw).expect("parse json");
    let report_rel = payload["report_path"].as_str().expect("report_path");
    let report_path = repo_root().join(report_rel);
    let report_text = std::fs::read_to_string(&report_path)
        .unwrap_or_else(|err| panic!("read {} failed: {err}", report_path.display()));
    let report: serde_json::Value = serde_json::from_str(&report_text).expect("parse report");
    let sections = report["sections"].as_object().expect("sections object");
    assert!(
        sections.contains_key("Automation purity"),
        "governance report must include Automation purity section"
    );
    assert!(
        sections.contains_key("Tutorials purity"),
        "governance report must include Tutorials purity section"
    );
    assert!(
        sections.contains_key("Clients tooling purity"),
        "governance report must include Clients tooling purity section"
    );
    assert!(
        sections.contains_key("Directory Purity"),
        "governance report must include Directory Purity section"
    );
    assert!(
        sections.contains_key("Repo purity"),
        "governance report must include Repo purity section"
    );
}
