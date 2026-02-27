// SPDX-License-Identifier: Apache-2.0

fn observe_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-OBS-001".to_string()),
            title: "observability alert rules contract",
            tests: vec![TestCase {
                id: TestId("ops.observe.alert_rules_exist_parseable".to_string()),
                title: "required alert rule sources exist and are parseable",
                kind: TestKind::Pure,
                run: test_ops_obs_001_alert_rules_exist_parseable,
            }],
        },
        Contract {
            id: ContractId("OPS-OBS-002".to_string()),
            title: "observability dashboard golden contract",
            tests: vec![TestCase {
                id: TestId("ops.observe.dashboard_json_parseable_golden_diff".to_string()),
                title: "dashboard json parses and matches golden identity and panel structure",
                kind: TestKind::Pure,
                run: test_ops_obs_002_dashboard_json_parseable_golden_diff,
            }],
        },
        Contract {
            id: ContractId("OPS-OBS-003".to_string()),
            title: "observability telemetry golden profile contract",
            tests: vec![TestCase {
                id: TestId("ops.observe.telemetry_goldens_required_profiles".to_string()),
                title: "telemetry goldens exist for required profiles and are indexed",
                kind: TestKind::Pure,
                run: test_ops_obs_003_telemetry_goldens_required_profiles,
            }],
        },
        Contract {
            id: ContractId("OPS-OBS-004".to_string()),
            title: "observability readiness schema contract",
            tests: vec![TestCase {
                id: TestId("ops.observe.readiness_schema_valid".to_string()),
                title: "readiness contract is parseable and uses canonical requirement set",
                kind: TestKind::Pure,
                run: test_ops_obs_004_readiness_schema_valid,
            }],
        },
        Contract { id: ContractId("OPS-OBS-005".to_string()), title: "observability alert catalog generation contract", tests: vec![TestCase { id: TestId("ops.observe.alert_catalog_generated_consistency".to_string()), title: "alert catalog is populated and aligned with parsed alert rules", kind: TestKind::Pure, run: test_ops_obs_005_alert_catalog_generated_consistency, }] },
        Contract { id: ContractId("OPS-OBS-006".to_string()), title: "observability slo burn-rate consistency contract", tests: vec![TestCase { id: TestId("ops.observe.slo_definitions_burn_rate_consistent".to_string()), title: "slo definitions and burn-rate rules remain aligned", kind: TestKind::Pure, run: test_ops_obs_006_slo_definitions_burn_rate_consistent, }] },
        Contract { id: ContractId("OPS-OBS-007".to_string()), title: "observability public surface coverage contract", tests: vec![TestCase { id: TestId("ops.observe.public_surface_coverage_matches_rules".to_string()), title: "public surface coverage rules include required surfaces", kind: TestKind::Pure, run: test_ops_obs_007_public_surface_coverage_matches_rules, }] },
        Contract { id: ContractId("OPS-OBS-008".to_string()), title: "observability telemetry index determinism contract", tests: vec![TestCase { id: TestId("ops.observe.telemetry_index_generated_deterministic".to_string()), title: "telemetry index is schema-versioned and sorted deterministically", kind: TestKind::Pure, run: test_ops_obs_008_telemetry_index_generated_deterministic, }] },
        Contract { id: ContractId("OPS-OBS-009".to_string()), title: "observability drills manifest contract", tests: vec![TestCase { id: TestId("ops.observe.drills_manifest_exists_runnable".to_string()), title: "drills manifest is populated with runnable drill definitions", kind: TestKind::Pure, run: test_ops_obs_009_drills_manifest_exists_runnable, }] },
        Contract { id: ContractId("OPS-OBS-010".to_string()), title: "observability overload behavior contract", tests: vec![TestCase { id: TestId("ops.observe.overload_behavior_contract_enforced".to_string()), title: "overload behavior contract exists and maps to load suite coverage", kind: TestKind::Pure, run: test_ops_obs_010_overload_behavior_contract_enforced, }] },
        Contract {
            id: ContractId("OPS-OBS-E-001".to_string()),
            title: "observe effect metrics scrape contract",
            tests: vec![TestCase {
                id: TestId("ops.observe.effect.scrape_metrics_contract".to_string()),
                title: "effect lane requires non-empty metrics scrape contract",
                kind: TestKind::Network,
                run: test_ops_obs_e_001_scrape_metrics_contract,
            }],
        },
        Contract {
            id: ContractId("OPS-OBS-E-002".to_string()),
            title: "observe effect trace structure contract",
            tests: vec![TestCase {
                id: TestId("ops.observe.effect.trace_structure_contract".to_string()),
                title: "effect lane requires trace structure golden contract",
                kind: TestKind::Network,
                run: test_ops_obs_e_002_trace_structure_contract,
            }],
        },
        Contract {
            id: ContractId("OPS-OBS-E-003".to_string()),
            title: "observe effect alerts load contract",
            tests: vec![TestCase {
                id: TestId("ops.observe.effect.alerts_load_contract".to_string()),
                title: "effect lane requires parseable alert rule inputs",
                kind: TestKind::Network,
                run: test_ops_obs_e_003_alerts_load_contract,
            }],
        },
    ]
}
