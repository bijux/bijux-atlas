use std::fs;
use std::path::PathBuf;

use bijux_atlas_dev_core::{expand_suite, list_output, load_registry, select_checks, Selectors};
use bijux_atlas_dev_model::SuiteId;

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
        root.join("crates/bijux-atlas-dev-core/tests/goldens/suite_ops_fast.txt"),
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
        fs::read_to_string(root.join("crates/bijux-atlas-dev-core/tests/goldens/list_default.txt"))
            .expect("golden");
    assert_eq!(rendered, golden);
}
