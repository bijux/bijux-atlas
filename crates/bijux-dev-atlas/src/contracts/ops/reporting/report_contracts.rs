// SPDX-License-Identifier: Apache-2.0

fn report_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-REPORT-001".to_string()),
            title: "report schema ssot contract",
            tests: vec![TestCase {
                id: TestId("ops.report.schema_is_ssot".to_string()),
                title: "report schema is parseable and mirrored under ops/schema/report",
                kind: TestKind::Pure,
                run: test_ops_rpt_001_report_schema_ssot,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-002".to_string()),
            title: "report generated payload contract",
            tests: vec![TestCase {
                id: TestId("ops.report.generated_reports_schema_valid".to_string()),
                title: "generated report payloads are parseable and include schema_version",
                kind: TestKind::Pure,
                run: test_ops_rpt_002_generated_reports_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-003".to_string()),
            title: "report evidence levels contract",
            tests: vec![TestCase {
                id: TestId("ops.report.evidence_levels_complete".to_string()),
                title: "evidence levels include minimal standard and forensic",
                kind: TestKind::Pure,
                run: test_ops_rpt_003_evidence_levels_complete,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-004".to_string()),
            title: "report diff structure contract",
            tests: vec![TestCase {
                id: TestId("ops.report.diff_contract_exists".to_string()),
                title: "generated report diff includes base target and change set",
                kind: TestKind::Pure,
                run: test_ops_rpt_004_report_diff_contract_exists,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-005".to_string()),
            title: "report readiness score determinism contract",
            tests: vec![TestCase {
                id: TestId("ops.report.readiness_score_deterministic".to_string()),
                title: "readiness score report is schema-versioned and uses canonical input keys",
                kind: TestKind::Pure,
                run: test_ops_rpt_005_readiness_score_deterministic,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-006".to_string()),
            title: "report release evidence bundle contract",
            tests: vec![TestCase {
                id: TestId("ops.report.release_evidence_bundle_schema_valid".to_string()),
                title: "release evidence bundle is parseable and references existing artifacts",
                kind: TestKind::Pure,
                run: test_ops_rpt_006_release_evidence_bundle_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-007".to_string()),
            title: "report historical comparison contract",
            tests: vec![TestCase {
                id: TestId("ops.report.historical_comparison_schema_valid".to_string()),
                title: "historical comparison report includes schema and readiness trend fields",
                kind: TestKind::Pure,
                run: test_ops_rpt_007_historical_comparison_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-008".to_string()),
            title: "report unified example contract",
            tests: vec![TestCase {
                id: TestId("ops.report.unified_report_example_schema_valid".to_string()),
                title: "unified report example includes required schema and summary sections",
                kind: TestKind::Pure,
                run: test_ops_rpt_008_unified_report_example_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-009".to_string()),
            title: "report canonical json output contract",
            tests: vec![TestCase {
                id: TestId("ops.report.outputs_canonical_json".to_string()),
                title: "report outputs are canonical pretty json with deterministic key ordering",
                kind: TestKind::Pure,
                run: test_ops_rpt_009_report_outputs_canonical_json,
            }],
        },
        Contract {
            id: ContractId("OPS-REPORT-010".to_string()),
            title: "report lane aggregation contract",
            tests: vec![TestCase {
                id: TestId("ops.report.lane_reports_aggregated_in_unified_report".to_string()),
                title: "unified report summary totals are derived from lane report statuses",
                kind: TestKind::Pure,
                run: test_ops_rpt_010_lane_reports_aggregated_in_unified_report,
            }],
        },
    ]
}
