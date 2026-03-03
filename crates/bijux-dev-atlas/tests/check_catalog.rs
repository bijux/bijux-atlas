// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_dev_atlas::registry::{CheckCatalog, CheckCatalogEntry};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn check_catalog_loads_in_deterministic_order() {
    let catalog = CheckCatalog::load(&repo_root()).expect("check catalog");
    let entries = catalog.entries();
    assert!(!entries.is_empty());

    let mut sorted = entries
        .iter()
        .map(|entry| (entry.domain, entry.check_id.as_str()))
        .collect::<Vec<_>>();
    let snapshot = sorted.clone();
    sorted.sort();
    assert_eq!(snapshot, sorted, "check catalog must stay sorted");
}

#[test]
fn check_catalog_rejects_duplicate_ids() {
    let catalog = CheckCatalog::from_entries(vec![
        CheckCatalogEntry {
            domain: "ops",
            check_id: "CHECK-DUP-001".to_string(),
            summary: "dup".to_string(),
            group: "ops".to_string(),
            tags: vec!["ops".to_string()],
            commands: vec!["make check".to_string()],
            doc_ref: bijux_dev_atlas::model::DocRef::new(
                "docs/reference/ops-surface.md",
                None,
                "Ops Surface",
            ),
        },
        CheckCatalogEntry {
            domain: "governance",
            check_id: "CHECK-DUP-001".to_string(),
            summary: "dup".to_string(),
            group: "governance".to_string(),
            tags: vec!["governance".to_string()],
            commands: vec!["make check".to_string()],
            doc_ref: bijux_dev_atlas::model::DocRef::new(
                "docs/_internal/governance/checks-and-contracts.md",
                None,
                "Checks And Contracts",
            ),
        },
    ]);

    let error = catalog
        .validate(&repo_root())
        .expect_err("duplicate id must fail");
    assert!(error.contains("duplicate check id `CHECK-DUP-001`"));
}
