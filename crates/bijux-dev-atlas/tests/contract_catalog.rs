// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_dev_atlas::contracts::{Contract, ContractId, TestCase, TestId, TestKind, TestResult};
use bijux_dev_atlas::model::DocRef;
use bijux_dev_atlas::registry::{ContractCatalog, ContractCatalogEntry};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn contract_catalog_loads_primary_domains_in_deterministic_order() {
    let catalog = ContractCatalog::load(&repo_root()).expect("contract catalog");
    let entries = catalog.entries();
    assert!(!entries.is_empty());

    let mut sorted = entries
        .iter()
        .map(|entry| (entry.domain, entry.id().to_string()))
        .collect::<Vec<_>>();
    let snapshot = sorted.clone();
    sorted.sort();
    assert_eq!(snapshot, sorted, "contract catalog must stay sorted");

    for domain in ["configs", "docs", "docker", "governance", "ops"] {
        assert!(
            !catalog.for_domain(domain).is_empty(),
            "expected contracts for domain `{domain}`"
        );
    }
}

#[test]
fn contract_catalog_rejects_duplicate_ids() {
    let duplicate = || Contract {
        id: ContractId("DUP-001".to_string()),
        title: "duplicate",
        tests: vec![TestCase {
            id: TestId("main".to_string()),
            title: "main",
            kind: TestKind::Pure,
            run: |_ctx| TestResult::Pass,
        }],
    };

    let catalog = ContractCatalog::from_entries(vec![
        ContractCatalogEntry {
            domain: "docs",
            doc_ref: DocRef::new(
                "docs/_internal/contracts/docs/README.md",
                None,
                "Docs Contracts",
            ),
            contract: duplicate(),
        },
        ContractCatalogEntry {
            domain: "ops",
            doc_ref: DocRef::new("docs/reference/ops-surface.md", None, "Ops Surface"),
            contract: duplicate(),
        },
    ]);

    let error = catalog.validate().expect_err("duplicate id must fail");
    assert!(error.contains("duplicate contract id `DUP-001`"));
}

#[test]
fn contract_catalog_entries_resolve_docs_files() {
    let catalog = ContractCatalog::load(&repo_root()).expect("contract catalog");
    for entry in catalog.entries() {
        assert!(
            repo_root().join(entry.doc_ref.path).is_file(),
            "expected docs file for {} at {}",
            entry.id(),
            entry.doc_ref.path
        );
        assert!(
            !entry.doc_ref.title.trim().is_empty(),
            "expected docs title for {}",
            entry.id()
        );
    }
}
