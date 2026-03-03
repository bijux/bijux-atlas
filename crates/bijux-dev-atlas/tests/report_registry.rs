// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_dev_atlas::registry::{ReportRegistry, REPORTS_REGISTRY_PATH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn report_registry_loads_canonical_report_catalog() {
    let registry = ReportRegistry::load(&repo_root()).expect("report registry");
    assert_eq!(registry.schema_version, 1);
    assert!(
        registry
            .reports
            .iter()
            .any(|entry| entry.report_id == "ops-profiles"),
        "expected ops-profiles in {}",
        REPORTS_REGISTRY_PATH
    );
    assert!(
        registry
            .reports
            .iter()
            .all(|entry| repo_root().join(&entry.schema_path).exists()),
        "all schema paths in {} must exist",
        REPORTS_REGISTRY_PATH
    );
}
