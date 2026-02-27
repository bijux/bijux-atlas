// SPDX-License-Identifier: Apache-2.0

fn load_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-LOAD-001".to_string()),
            title: "load scenario schema contract",
            tests: vec![TestCase {
                id: TestId("ops.load.scenarios_schema_valid".to_string()),
                title: "load scenarios are parseable and include required fields",
                kind: TestKind::Pure,
                run: test_ops_load_001_scenarios_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-LOAD-002".to_string()),
            title: "load thresholds coverage contract",
            tests: vec![TestCase {
                id: TestId("ops.load.thresholds_exist_for_each_suite".to_string()),
                title: "every load suite has a matching thresholds file",
                kind: TestKind::Pure,
                run: test_ops_load_002_thresholds_exist_for_each_suite,
            }],
        },
        Contract {
            id: ContractId("OPS-LOAD-003".to_string()),
            title: "load pinned query lock contract",
            tests: vec![TestCase {
                id: TestId("ops.load.pinned_queries_lock_consistent".to_string()),
                title: "pinned query lock digests match source query payload",
                kind: TestKind::Pure,
                run: test_ops_load_003_pinned_queries_lock_consistent,
            }],
        },
        Contract {
            id: ContractId("OPS-LOAD-004".to_string()),
            title: "load baseline schema contract",
            tests: vec![TestCase {
                id: TestId("ops.load.baselines_schema_valid".to_string()),
                title: "load baselines are parseable and contain required fields",
                kind: TestKind::Pure,
                run: test_ops_load_004_baselines_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-LOAD-005".to_string()),
            title: "load scenario to slo mapping contract",
            tests: vec![TestCase {
                id: TestId("ops.load.no_scenario_without_slo_mapping".to_string()),
                title: "smoke/pr load suites must be represented in inventory SLO mappings",
                kind: TestKind::Pure,
                run: test_ops_load_005_no_scenario_without_slo_mapping,
            }],
        },
        Contract { id: ContractId("OPS-LOAD-006".to_string()), title: "load drift report schema contract", tests: vec![TestCase { id: TestId("ops.load.drift_report_generator_schema_valid".to_string()), title: "load drift report exists and is schema-valid", kind: TestKind::Pure, run: test_ops_load_006_drift_report_generator_schema_valid, }] },
        Contract { id: ContractId("OPS-LOAD-007".to_string()), title: "load result schema sample contract", tests: vec![TestCase { id: TestId("ops.load.result_schema_validates_sample_output".to_string()), title: "load result schema validates generated sample summary envelope", kind: TestKind::Pure, run: test_ops_load_007_result_schema_validates_sample_output, }] },
        Contract { id: ContractId("OPS-LOAD-008".to_string()), title: "load cheap survival suite gate contract", tests: vec![TestCase { id: TestId("ops.load.cheap_survival_in_minimal_gate_suite".to_string()), title: "cheap-only-survival suite is present in minimal gate lanes", kind: TestKind::Pure, run: test_ops_load_008_cheap_survival_in_minimal_gate_suite, }] },
        Contract { id: ContractId("OPS-LOAD-009".to_string()), title: "load cold start p99 suite gate contract", tests: vec![TestCase { id: TestId("ops.load.cold_start_p99_in_minimal_gate_suite".to_string()), title: "cold-start-p99 suite is present in required confidence lanes", kind: TestKind::Pure, run: test_ops_load_009_cold_start_p99_in_minimal_gate_suite, }] },
        Contract { id: ContractId("OPS-LOAD-010".to_string()), title: "load scenario slo impact mapping contract", tests: vec![TestCase { id: TestId("ops.load.every_scenario_has_slo_impact_class".to_string()), title: "every load suite maps to a scenario slo impact class entry", kind: TestKind::Pure, run: test_ops_load_010_every_scenario_has_slo_impact_class, }] },
        Contract {
            id: ContractId("OPS-LOAD-E-001".to_string()),
            title: "load effect k6 execution contract",
            tests: vec![TestCase {
                id: TestId("ops.load.effect.k6_suite_executes_contract".to_string()),
                title: "effect lane requires at least one k6 load suite definition",
                kind: TestKind::Subprocess,
                run: test_ops_load_e_001_k6_suite_executes_contract,
            }],
        },
        Contract {
            id: ContractId("OPS-LOAD-E-002".to_string()),
            title: "load effect thresholds report contract",
            tests: vec![TestCase {
                id: TestId("ops.load.effect.thresholds_enforced_report_emitted".to_string()),
                title: "effect lane requires thresholds contract and emitted load summary report",
                kind: TestKind::Subprocess,
                run: test_ops_load_e_002_thresholds_enforced_report_emitted,
            }],
        },
        Contract {
            id: ContractId("OPS-LOAD-E-003".to_string()),
            title: "load effect baseline comparison contract",
            tests: vec![TestCase {
                id: TestId("ops.load.effect.baseline_comparison_produced".to_string()),
                title: "effect lane requires emitted load drift comparison report",
                kind: TestKind::Subprocess,
                run: test_ops_load_e_003_baseline_comparison_produced,
            }],
        },
    ]
}
