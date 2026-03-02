// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use bijux_dev_atlas::ops::helm_env::{build_subset_report, HelmEnvInputs};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("read golden")
}

fn default_inputs() -> HelmEnvInputs {
    HelmEnvInputs {
        chart_dir: "ops/k8s/charts/bijux-atlas".to_string(),
        values_files: vec!["ops/k8s/charts/bijux-atlas/values.yaml".to_string()],
        release_name: "bijux-atlas".to_string(),
        helm_binary: "helm".to_string(),
    }
}

#[test]
fn mismatch_snapshot_matches_golden() {
    let emitted = BTreeSet::from(["ATLAS_DECLARED".to_string(), "ATLAS_UNDECLARED".to_string()]);
    let allowed = BTreeSet::from(["ATLAS_DECLARED".to_string()]);
    let report = build_subset_report(&emitted, &allowed, default_inputs());
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::to_value(report).expect("json")).expect("pretty")
    );
    assert_eq!(
        rendered,
        read("crates/bijux-dev-atlas/tests/goldens/ops_helm_env_subset_fail.json")
    );
}

#[test]
fn success_snapshot_matches_golden() {
    let emitted = BTreeSet::from(["ATLAS_DECLARED".to_string()]);
    let allowed = BTreeSet::from(["ATLAS_DECLARED".to_string()]);
    let report = build_subset_report(&emitted, &allowed, default_inputs());
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::to_value(report).expect("json")).expect("pretty")
    );
    assert_eq!(
        rendered,
        read("crates/bijux-dev-atlas/tests/goldens/ops_helm_env_subset_pass.json")
    );
}
