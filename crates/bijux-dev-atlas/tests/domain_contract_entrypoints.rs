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
fn canonical_domain_contract_entrypoints_resolve_for_primary_domains() {
    let root = repo_root();

    assert!(!bijux_dev_atlas::domains::configs::contracts(&root)
        .expect("configs contracts")
        .is_empty());
    assert!(!bijux_dev_atlas::domains::docs::contracts(&root)
        .expect("docs contracts")
        .is_empty());
    assert!(!bijux_dev_atlas::domains::docker::contracts(&root)
        .expect("docker contracts")
        .is_empty());
    assert!(!bijux_dev_atlas::domains::ops::contracts(&root)
        .expect("ops contracts")
        .is_empty());
    assert!(!bijux_dev_atlas::domains::governance::contracts(&root)
        .expect("governance contracts")
        .is_empty());

    assert!(
        !bijux_dev_atlas::domains::configs::contracts::contracts(&root)
            .expect("configs contracts module")
            .is_empty()
    );
    assert!(!bijux_dev_atlas::domains::docs::contracts::contracts(&root)
        .expect("docs contracts module")
        .is_empty());
    assert!(
        !bijux_dev_atlas::domains::docker::contracts::contracts(&root)
            .expect("docker contracts module")
            .is_empty()
    );
    assert!(!bijux_dev_atlas::domains::ops::contracts::contracts(&root)
        .expect("ops contracts module")
        .is_empty());
    assert!(
        !bijux_dev_atlas::domains::governance::contracts::contracts(&root)
            .expect("governance contracts module")
            .is_empty()
    );
}
