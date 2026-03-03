// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use std::fs;

use bijux_dev_atlas::registry::{
    ReportRegistry, REPORTS_REGISTRY_PATH, REPORTS_REGISTRY_SCHEMA_PATH,
};

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

#[test]
fn report_registry_catalog_matches_schema_and_report_schemas() {
    let validation = ReportRegistry::validate_catalog(&repo_root()).expect("catalog validation");
    assert_eq!(validation.report_count, 5);
    assert!(
        validation.errors.is_empty(),
        "unexpected catalog errors: {:?}",
        validation.errors
    );
    assert!(
        repo_root().join(REPORTS_REGISTRY_SCHEMA_PATH).exists(),
        "missing {}",
        REPORTS_REGISTRY_SCHEMA_PATH
    );
}

#[test]
fn report_registry_validates_report_artifacts() {
    let temp_root = std::env::temp_dir().join(format!(
        "bijux-dev-atlas-report-validation-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_root);
    fs::create_dir_all(&temp_root).expect("create temp report dir");
    let payload = serde_json::json!({
      "report_id": "ops-profiles",
      "version": 1,
      "schema_version": 1,
      "kind": "ops_profiles_matrix",
      "inputs": {
        "chart_dir": "ops/helm/chart",
        "values_root": "ops/helm/values",
        "schema_path": "configs/contracts/reports/ops-profiles.schema.json",
        "dataset_manifest_path": "ops/datasets/manifest.json",
        "profile_selector": "all"
      },
      "tooling": [],
      "rows": [],
      "summary": {
        "total": 0,
        "helm_failures": 0,
        "schema_failures": 0,
        "dataset_failures": 0,
        "kubeconform_failures": 0
      }
    });
    fs::write(
        temp_root.join("ops-profiles.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&payload).expect("encode")
        ),
    )
    .expect("write temp report");

    let validation =
        ReportRegistry::validate_reports_dir(&repo_root(), &temp_root).expect("report validation");
    assert_eq!(validation.scanned_reports, 1);
    assert!(
        validation.errors.is_empty(),
        "unexpected report errors: {:?}",
        validation.errors
    );

    let _ = fs::remove_dir_all(&temp_root);
}
