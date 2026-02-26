// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn onboarding_root_points_to_start_here() {
    let root = repo_root();
    let start_here = root.join("docs/START_HERE.md");
    assert!(start_here.exists(), "missing docs/START_HERE.md");
    let text = fs::read_to_string(start_here).expect("read start here");
    assert!(
        text.contains("bijux dev atlas demo quickstart"),
        "docs/START_HERE.md must contain canonical quickstart command"
    );
}

#[test]
fn onboarding_docs_layer_stays_within_budget() {
    let root = repo_root();
    let quickstart_dir = root.join("docs/quickstart");
    let mut markdown_pages = Vec::new();
    for entry in fs::read_dir(&quickstart_dir).expect("read quickstart dir") {
        let path = entry.expect("quickstart entry").path();
        if path.extension().and_then(|v| v.to_str()) == Some("md") {
            markdown_pages.push(path);
        }
    }
    assert!(
        markdown_pages.len() <= 5,
        "onboarding layer budget exceeded: docs/quickstart has {} markdown pages",
        markdown_pages.len()
    );
}
