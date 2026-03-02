// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use super::*;

#[test]
fn discovers_profiles_in_lexicographic_order() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("z.yaml"), "").expect("write z");
    std::fs::write(temp.path().join("a.yaml"), "").expect("write a");
    std::fs::write(temp.path().join("notes.txt"), "").expect("write txt");
    let rows = discover_profiles(temp.path()).expect("discover");
    let names = rows
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["a.yaml", "z.yaml"]);
}

#[test]
fn validates_report_against_schema_shape() {
    let report = build_report(
        vec![ProfileMatrixRow {
            profile: "ci".to_string(),
            values_file: "ops/k8s/values/ci.yaml".to_string(),
            helm_template: StatusReport {
                status: "pass".to_string(),
                note: String::new(),
                errors: Vec::new(),
                event: ToolInvocationReport {
                    binary: "helm".to_string(),
                    args: vec!["template".to_string()],
                    cwd: ".".to_string(),
                    status: "pass".to_string(),
                    stderr: String::new(),
                },
            },
            values_schema: StatusReport {
                status: "pass".to_string(),
                note: String::new(),
                errors: Vec::new(),
                event: ToolInvocationReport {
                    binary: "values.schema.json".to_string(),
                    args: vec!["values.schema.json".to_string()],
                    cwd: ".".to_string(),
                    status: "pass".to_string(),
                    stderr: String::new(),
                },
            },
            dataset_validation: StatusReport {
                status: "pass".to_string(),
                note: String::new(),
                errors: Vec::new(),
                event: ToolInvocationReport {
                    binary: "ops/datasets/manifest.json".to_string(),
                    args: vec!["ops/datasets/manifest.json".to_string()],
                    cwd: ".".to_string(),
                    status: "pass".to_string(),
                    stderr: String::new(),
                },
            },
            kubeconform: StatusReport {
                status: "skipped".to_string(),
                note: String::new(),
                errors: Vec::new(),
                event: ToolInvocationReport {
                    binary: "kubeconform".to_string(),
                    args: vec!["-strict".to_string()],
                    cwd: ".".to_string(),
                    status: "skipped".to_string(),
                    stderr: String::new(),
                },
            },
            rendered_resources: 1,
        }],
        ProfilesMatrixInputs {
            chart_dir: "ops/k8s/charts/bijux-atlas".to_string(),
            values_root: "ops/k8s/values".to_string(),
            schema_path: "ops/k8s/charts/bijux-atlas/values.schema.json".to_string(),
            dataset_manifest_path: "ops/datasets/manifest.json".to_string(),
            profile_selector: "all".to_string(),
        },
        vec![ToolVersionRow {
            binary: "helm".to_string(),
            probe_argv: vec!["version".to_string(), "--short".to_string()],
            declared: true,
        }],
    );
    let report_value = serde_json::to_value(report).expect("report json");
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .join("configs/contracts/reports/ops-profiles.schema.json");
    validate_report_value(&report_value, &schema_path).expect("report schema");
}

#[test]
fn detects_invalid_pinned_dataset_ids() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf();
    let manifest_path = repo_root.join("ops/datasets/manifest.json");
    let manifest_ids = load_dataset_manifest_ids(&manifest_path).expect("manifest ids");
    let merged_values = serde_json::json!({
        "cache": {
            "pinnedDatasets": ["missing/dataset/id"]
        }
    });
    let status =
        dataset_validation_status(&repo_root, &manifest_path, &manifest_ids, &merged_values);
    assert_eq!(status.status, "fail");
    assert!(
        status.errors[0].contains("missing/dataset/id"),
        "dataset validation must identify the missing dataset id"
    );
}

#[test]
fn accepts_valid_pinned_dataset_ids() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf();
    let manifest_path = repo_root.join("ops/datasets/manifest.json");
    let manifest_ids = load_dataset_manifest_ids(&manifest_path).expect("manifest ids");
    let merged_values = serde_json::json!({
        "cache": {
            "pinnedDatasets": ["110/homo_sapiens/GRCh38"]
        }
    });
    let status =
        dataset_validation_status(&repo_root, &manifest_path, &manifest_ids, &merged_values);
    assert_eq!(status.status, "pass");
    assert!(status.errors.is_empty());
}
