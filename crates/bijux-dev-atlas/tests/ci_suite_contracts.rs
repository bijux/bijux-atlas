// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
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

fn suite_ids(suite: &str) -> BTreeSet<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "check",
            "list",
            "--suite",
            suite,
            "--include-internal",
            "--include-slow",
            "--format",
            "json",
        ])
        .output()
        .expect("check list suite");
    assert!(output.status.success(), "suite `{suite}` must resolve");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    payload["checks"]
        .as_array()
        .expect("checks array")
        .iter()
        .filter_map(|row| row.get("id").and_then(|v| v.as_str()).map(ToString::to_string))
        .collect()
}

#[test]
fn ci_suites_are_distinct_and_progressive() {
    let fast = suite_ids("ci_fast");
    let pr = suite_ids("ci_pr");
    let nightly = suite_ids("ci_nightly");

    assert!(!fast.is_empty(), "ci_fast must not be empty");
    assert!(!pr.is_empty(), "ci_pr must not be empty");
    assert!(!nightly.is_empty(), "ci_nightly must not be empty");

    assert!(fast.is_subset(&pr), "ci_fast must be subset of ci_pr");
    assert!(pr.is_subset(&nightly), "ci_pr must be subset of ci_nightly");
    assert!(
        fast.len() < pr.len(),
        "ci_pr must contain strictly more checks than ci_fast"
    );
    assert!(
        pr.len() < nightly.len(),
        "ci_nightly must contain strictly more checks than ci_pr"
    );
}
