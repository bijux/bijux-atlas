// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use bijux_dev_atlas::contracts::{
    lint_registry_rows, run, Contract, ContractId, Mode, RegistrySnapshotRow, RunContext,
    RunOptions, TestCase, TestId, TestKind, TestResult,
};

fn pass_case(_: &RunContext) -> TestResult {
    TestResult::Pass
}

#[test]
fn registry_lints_detect_duplicate_contract_and_test_ids() {
    let rows = vec![
        RegistrySnapshotRow {
            domain: "docker".to_string(),
            id: "DOCKER-001".to_string(),
            title: "first".to_string(),
            test_ids: vec!["docker.sample.same".to_string()],
        },
        RegistrySnapshotRow {
            domain: "ops".to_string(),
            id: "DOCKER-001".to_string(),
            title: "second".to_string(),
            test_ids: vec!["docker.sample.same".to_string()],
        },
    ];
    let lints = lint_registry_rows(&rows);
    assert!(lints.iter().any(|lint| lint.code == "duplicate-contract-id"));
    assert!(lints.iter().any(|lint| lint.code == "duplicate-test-id"));
}

#[test]
fn run_honors_only_skip_and_tag_filters() {
    fn registry(_: &Path) -> Result<Vec<Contract>, String> {
        Ok(vec![
            Contract {
                id: ContractId("OPS-ROOT-001".to_string()),
                title: "root",
                tests: vec![TestCase {
                    id: TestId("ops.root.static".to_string()),
                    title: "static",
                    kind: TestKind::Pure,
                    run: pass_case,
                }],
            },
            Contract {
                id: ContractId("OPS-STACK-001".to_string()),
                title: "stack",
                tests: vec![TestCase {
                    id: TestId("ops.stack.effect".to_string()),
                    title: "effect",
                    kind: TestKind::Subprocess,
                    run: pass_case,
                }],
            },
        ])
    }

    let options = RunOptions {
        mode: Mode::Static,
        allow_subprocess: false,
        allow_network: false,
        skip_missing_tools: false,
        timeout_seconds: 30,
        fail_fast: false,
        contract_filter: None,
        test_filter: None,
        only_contracts: vec!["OPS-*".to_string()],
        only_tests: Vec::new(),
        skip_contracts: vec!["OPS-STACK-*".to_string()],
        tags: vec!["static".to_string()],
        list_only: false,
        artifacts_root: None,
    };
    let report = run("ops", registry, Path::new("."), &options).expect("run");
    assert_eq!(report.total_contracts(), 1);
    assert_eq!(report.contracts[0].id, "OPS-ROOT-001");
}
