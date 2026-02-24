use std::path::PathBuf;

use bijux_atlas_policies::{
    evaluate_policy_set, evaluate_repository_metrics, load_policy_set_from_workspace,
    RepositoryMetrics,
};

fn fixture(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join(path)).expect("read fixture")
}

#[test]
fn policy_evaluation_table_matches_golden_violations() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let valid = load_policy_set_from_workspace(&root).expect("load policy");
    let valid_violations = evaluate_policy_set(&valid);
    let valid_golden: serde_json::Value = serde_json::from_str(&fixture("tests/fixtures/evaluation/valid_policy.json")).expect("golden");
    assert_eq!(serde_json::to_value(valid_violations).expect("encode"), valid_golden);

    let mut invalid = valid.clone();
    invalid.allow_override = true;
    invalid.network_in_unit_tests = true;
    invalid.telemetry.metrics_enabled = false;

    let invalid_violations = evaluate_policy_set(&invalid)
        .into_iter()
        .filter(|v| {
            v.id == "policy.global.allow_override.forbidden"
                || v.id == "policy.global.network_in_unit_tests.forbidden"
                || v.id == "policy.telemetry.metrics.required"
        })
        .collect::<Vec<_>>();
    let invalid_golden: serde_json::Value = serde_json::from_str(&fixture("tests/fixtures/evaluation/invalid_policy.json")).expect("golden");
    assert_eq!(serde_json::to_value(invalid_violations).expect("encode"), invalid_golden);
}

#[test]
fn repository_metric_evaluation_matches_golden_violations() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let policy = load_policy_set_from_workspace(&root).expect("load policy");

    let metrics = RepositoryMetrics {
        dataset_count: 99,
        open_shards_per_pod: 32,
        disk_bytes: 999_999_999_999,
    };

    let violations = evaluate_repository_metrics(&policy, &metrics);
    let golden: serde_json::Value = serde_json::from_str(&fixture("tests/fixtures/evaluation/repo_budget_exceeded.json")).expect("golden");
    assert_eq!(serde_json::to_value(violations).expect("encode"), golden);
}
