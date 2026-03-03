// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::model::{
    ContractCaseResult, ContractCaseStatus, ContractRunCounts, ContractRunPreflight,
    ContractRunSummary, ReportHeader, ReportRef,
};

#[test]
fn contract_run_summary_roundtrips() {
    let summary = ContractRunSummary {
        header: ReportHeader::new(
            "contract-run-summary",
            1,
            serde_json::json!({
                "mode": "static",
                "jobs": "auto",
                "fail_fast": false,
            }),
            vec!["contracts/docs.json".to_string()],
        ),
        mode: "static".to_string(),
        jobs: "auto".to_string(),
        fail_fast: false,
        preflight: ContractRunPreflight {
            required_tools: vec!["sh".to_string()],
            missing_tools: Vec::new(),
        },
        counts: ContractRunCounts {
            total: 1,
            passed: 1,
            failed: 0,
            skipped: 0,
            not_run: 0,
        },
        reports: vec![ReportRef {
            report_id: "contracts-docs".to_string(),
            path: "contracts/docs.json".to_string(),
        }],
        cases: vec![ContractCaseResult {
            contract_id: "DOC-001".to_string(),
            contract_name: "docs::DOC-001".to_string(),
            case_name: "docs.surface.allowed_root_dirs".to_string(),
            status: ContractCaseStatus::Pass,
            duration_ms: 16,
            message: None,
            artifact_paths: vec![
                "artifacts/contracts/DOC-001/cases/docs.surface.allowed_root_dirs.json".to_string(),
            ],
        }],
    };

    let json = serde_json::to_string_pretty(&summary).expect("encode");
    let restored: ContractRunSummary = serde_json::from_str(&json).expect("decode");
    assert_eq!(summary, restored);
}
