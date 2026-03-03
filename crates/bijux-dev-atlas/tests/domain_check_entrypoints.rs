// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn canonical_domain_check_entrypoints_resolve_for_ops_and_governance() {
    let root = repo_root();

    assert!(!bijux_dev_atlas::domains::ops::checks::checks(&root)
        .expect("ops checks")
        .is_empty());
    assert!(!bijux_dev_atlas::domains::governance::checks::checks(&root)
        .expect("governance checks")
        .is_empty());
}
