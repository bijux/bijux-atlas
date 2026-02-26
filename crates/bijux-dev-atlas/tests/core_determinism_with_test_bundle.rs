// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_dev_atlas::adapters::TestBundle;
use bijux_dev_atlas::core::{run_checks, Capabilities, RunOptions, RunRequest, Selectors};
use bijux_dev_atlas::model::{CheckStatus, DomainId, RunId};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn subprocess_only_request() -> RunRequest {
    RunRequest {
        repo_root: repo_root(),
        domain: Some(DomainId::Crates),
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: Some(RunId::from_seed("deterministic_test")),
        command: Some("dev-atlas-test".to_string()),
    }
}

fn subprocess_only_selectors() -> Selectors {
    Selectors {
        id_glob: Some("*plugin_conformance_binaries*".to_string()),
        include_internal: true,
        include_slow: true,
        ..Selectors::default()
    }
}

#[test]
fn representative_ops_crate_check_runs_with_test_bundle_without_subprocess_execution() {
    let bundle = TestBundle::new().with_fixed_time(1_700_000_000);
    let report = run_checks(
        bundle.process_runner(),
        bundle.filesystem(),
        &subprocess_only_request(),
        &subprocess_only_selectors(),
        &RunOptions::default(),
    )
    .expect("run report");

    assert!(
        report
            .results
            .iter()
            .any(|row| row.id.as_str() == "checks_crates_plugin_conformance_binaries"
                && row.status == CheckStatus::Skip),
        "selected subprocess-only check must be skipped under denied subprocess capability"
    );
    assert!(report.summary.total >= 1);
    assert!(report.summary.skipped >= 1);
}

#[test]
fn run_report_json_is_deterministic_with_test_bundle_and_fixed_inputs() {
    let bundle = TestBundle::new().with_fixed_time(1_700_000_000);
    let req = subprocess_only_request();
    let selectors = subprocess_only_selectors();
    let opts = RunOptions::default();

    let first = run_checks(
        bundle.process_runner(),
        bundle.filesystem(),
        &req,
        &selectors,
        &opts,
    )
    .expect("first report");
    let second = run_checks(
        bundle.process_runner(),
        bundle.filesystem(),
        &req,
        &selectors,
        &opts,
    )
    .expect("second report");

    let a = serde_json::to_string(&first).expect("serialize first");
    let b = serde_json::to_string(&second).expect("serialize second");
    assert_eq!(a, b, "serialized reports must be byte-identical");
}

#[test]
fn run_report_json_has_no_nondeterministic_timestamp_fields() {
    let bundle = TestBundle::new().with_fixed_time(1_700_000_000);
    let report = run_checks(
        bundle.process_runner(),
        bundle.filesystem(),
        &subprocess_only_request(),
        &subprocess_only_selectors(),
        &RunOptions::default(),
    )
    .expect("report");

    let json = serde_json::to_string_pretty(&report).expect("serialize");
    for marker in ["generatedAt", "timestamp", "creationTimestamp"] {
        assert!(
            !json.contains(marker),
            "report JSON must not contain nondeterministic field marker `{marker}`"
        );
    }
}
