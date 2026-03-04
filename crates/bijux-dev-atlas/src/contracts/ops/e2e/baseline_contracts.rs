// SPDX-License-Identifier: Apache-2.0

fn e2e_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-E2E-001".to_string()),
            title: "e2e suites reference contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.suites_reference_real_scenarios".to_string()),
                title: "e2e suites reference concrete scenario ids and runnable entrypoints",
                kind: TestKind::Pure,
                run: test_ops_e2e_001_suites_reference_real_scenarios,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-002".to_string()),
            title: "e2e smoke manifest contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.smoke_manifest_valid".to_string()),
                title: "smoke manifest is structured and points to existing lock",
                kind: TestKind::Pure,
                run: test_ops_e2e_002_smoke_manifest_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-003".to_string()),
            title: "e2e fixtures lock contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.fixtures_lock_matches_allowlist".to_string()),
                title: "fixtures lock digest and fixture files match allowlist policy",
                kind: TestKind::Pure,
                run: test_ops_e2e_003_fixtures_lock_matches_allowlist,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-004".to_string()),
            title: "e2e realdata snapshot contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.realdata_snapshots_schema_valid_and_pinned".to_string()),
                title: "realdata snapshots are parseable and pinned to canonical queries",
                kind: TestKind::Pure,
                run: test_ops_e2e_004_realdata_snapshots_schema_valid_and_pinned,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-005".to_string()),
            title: "e2e taxonomy coverage contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.taxonomy_covers_scenarios".to_string()),
                title: "taxonomy categories cover canonical scenario classification",
                kind: TestKind::Pure,
                run: test_ops_e2e_005_taxonomy_covers_scenarios,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-006".to_string()),
            title: "e2e reproducibility enforcement contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.reproducibility_policy_enforced".to_string()),
                title: "reproducibility policy checks and deterministic summary ordering are enforced",
                kind: TestKind::Pure,
                run: test_ops_e2e_006_reproducibility_policy_enforced,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-007".to_string()),
            title: "e2e coverage matrix determinism contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.coverage_matrix_deterministic".to_string()),
                title: "coverage matrix rows and coverage sets are complete and deterministic",
                kind: TestKind::Pure,
                run: test_ops_e2e_007_coverage_matrix_deterministic,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-008".to_string()),
            title: "e2e realdata scenario registry contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.realdata_registry_and_snapshots_valid".to_string()),
                title: "realdata scenarios and snapshots are structurally valid and runnable",
                kind: TestKind::Pure,
                run: test_ops_e2e_008_realdata_registry_and_snapshots_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-009".to_string()),
            title: "e2e surface artifact boundary contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.no_stray_e2e_artifacts".to_string()),
                title: "e2e root contains only declared artifact directories and files",
                kind: TestKind::Pure,
                run: test_ops_e2e_009_no_stray_e2e_artifacts,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-010".to_string()),
            title: "e2e summary schema contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.summary_schema_valid".to_string()),
                title: "e2e summary is schema-valid and aligned with suite/scenario registries",
                kind: TestKind::Pure,
                run: test_ops_e2e_010_summary_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-E-001".to_string()),
            title: "e2e effect smoke suite contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.effect.smoke_suite_passes_contract".to_string()),
                title: "effect lane requires smoke suite declaration in e2e suite registry",
                kind: TestKind::Subprocess,
                run: test_ops_e2e_e_001_smoke_suite_passes_contract,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-E-002".to_string()),
            title: "e2e effect realdata suite contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.effect.realdata_scenario_passes_contract".to_string()),
                title: "effect lane requires non-empty realdata scenario contract set",
                kind: TestKind::Subprocess,
                run: test_ops_e2e_e_002_realdata_scenario_passes_contract,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-011".to_string()),
            title: "scenario runner compatibility registry contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.scenario_runner_compatibility_registry_valid".to_string()),
                title: "scenario compatibility registry is present and structurally valid",
                kind: TestKind::Pure,
                run: test_ops_e2e_011_scenario_runner_compatibility_registry_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-012".to_string()),
            title: "scenario golden snapshots contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.scenario_goldens_are_present_and_parseable".to_string()),
                title: "required scenario golden snapshots are present and parseable",
                kind: TestKind::Pure,
                run: test_ops_e2e_012_scenario_goldens_are_present_and_parseable,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-013".to_string()),
            title: "scenario output contract fields",
            tests: vec![TestCase {
                id: TestId("ops.e2e.scenario_output_contract_fields_are_complete".to_string()),
                title: "scenario result contract includes deterministic run and required evidence fields",
                kind: TestKind::Pure,
                run: test_ops_e2e_013_scenario_output_contract_fields_are_complete,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-014".to_string()),
            title: "scenario prerequisite guard contract",
            tests: vec![TestCase {
                id: TestId(
                    "ops.e2e.scenario_runner_fails_fast_on_missing_prerequisites".to_string(),
                ),
                title: "scenario runner reports clear prerequisite failures and stays git-independent",
                kind: TestKind::Pure,
                run: test_ops_e2e_014_scenario_runner_fails_fast_on_missing_prerequisites,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-015".to_string()),
            title: "scenario tools registry coverage contract",
            tests: vec![TestCase {
                id: TestId(
                    "ops.e2e.scenario_required_tools_registry_covers_all_scenarios".to_string(),
                ),
                title: "required tools registry covers each declared scenario exactly once",
                kind: TestKind::Pure,
                run: test_ops_e2e_015_scenario_required_tools_registry_covers_all_scenarios,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-016".to_string()),
            title: "upgrade contracts and specs presence",
            tests: vec![TestCase {
                id: TestId("ops.e2e.upgrade_contracts_and_specs_exist".to_string()),
                title: "upgrade and rollback contracts plus scenario specs are present and parseable",
                kind: TestKind::Pure,
                run: test_ops_e2e_016_upgrade_contracts_and_specs_exist,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-017".to_string()),
            title: "upgrade compatibility table contract",
            tests: vec![TestCase {
                id: TestId("ops.e2e.upgrade_compatibility_table_is_complete".to_string()),
                title: "upgrade compatibility table includes patch minor and rollback support rows",
                kind: TestKind::Pure,
                run: test_ops_e2e_017_upgrade_compatibility_table_is_complete,
            }],
        },
        Contract {
            id: ContractId("OPS-E2E-018".to_string()),
            title: "upgrade and rollback evidence contract",
            tests: vec![TestCase {
                id: TestId(
                    "ops.e2e.upgrade_and_rollback_evidence_requirements_declared".to_string(),
                ),
                title: "scenario runner declares required before-after and rollback evidence artifacts",
                kind: TestKind::Pure,
                run: test_ops_e2e_018_upgrade_and_rollback_evidence_requirements_declared,
            }],
        },
    ]
}
