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
    ]
}
