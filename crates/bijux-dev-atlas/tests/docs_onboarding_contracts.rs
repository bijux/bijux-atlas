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
    let start_here = root.join("docs/start-here.md");
    assert!(start_here.exists(), "missing docs/start-here.md");
    let text = fs::read_to_string(start_here).expect("read start here");
    assert!(
        text.contains("bijux dev atlas demo quickstart"),
        "docs/start-here.md must contain canonical quickstart command"
    );
}

#[test]
fn onboarding_docs_layer_stays_within_budget() {
    let root = repo_root();
    let markdown_pages = fs::read_dir(root.join("docs"))
        .expect("read docs root")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    assert!(
        markdown_pages.len() <= 8,
        "onboarding layer budget exceeded: docs root has {} markdown pages",
        markdown_pages.len()
    );
}
