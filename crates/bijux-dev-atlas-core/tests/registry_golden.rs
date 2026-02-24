// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

use bijux_dev_atlas_core::{expand_suite, list_output, load_registry, select_checks, Selectors};
use bijux_dev_atlas_model::SuiteId;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn suite_expansion_matches_golden() {
    let root = repo_root();
    let registry = load_registry(&root).expect("registry");
    let suite = SuiteId::parse("ops_fast").expect("suite");
    let expanded = expand_suite(&registry, &suite).expect("expand");
    let rendered = expanded
        .iter()
        .map(|row| row.id.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let golden = fs::read_to_string(
        root.join("crates/bijux-dev-atlas-core/tests/goldens/suite_ops_fast.txt"),
    )
    .expect("golden");
    assert_eq!(rendered, golden);
}

#[test]
fn default_list_output_matches_golden() {
    let root = repo_root();
    let registry = load_registry(&root).expect("registry");
    let selected = select_checks(&registry, &Selectors::default()).expect("select");
    let rendered = list_output(&selected) + "\n";
    let golden =
        fs::read_to_string(root.join("crates/bijux-dev-atlas-core/tests/goldens/list_default.txt"))
            .expect("golden");
    assert_eq!(rendered, golden);
}

#[test]
fn registry_golden_files_have_single_writer() {
    let root = repo_root();
    let src_root = root.join("crates/bijux-dev-atlas-core");
    let mut stack = vec![src_root.clone()];
    let mut references = Vec::new();
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(path).expect("read_dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("read");
            if text.contains("tests/goldens/list_default.txt")
                || text.contains("tests/goldens/suite_ops_fast.txt")
            {
                references.push(path);
            }
        }
    }
    assert_eq!(
        references.len(),
        1,
        "registry goldens must be referenced by a single writer test file"
    );
    assert!(
        references[0].ends_with("tests/registry_golden.rs"),
        "single writer must be registry_golden.rs"
    );
}
