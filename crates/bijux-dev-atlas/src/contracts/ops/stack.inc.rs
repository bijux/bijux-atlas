// SPDX-License-Identifier: Apache-2.0

fn stack_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-STACK-001".to_string()),
            title: "stack toml profile contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.stack_toml_parseable_complete".to_string()),
                title: "stack.toml parses and includes canonical ci kind local profiles",
                kind: TestKind::Pure,
                run: test_ops_stack_001_stack_toml_parseable_complete,
            }],
        },
        Contract {
            id: ContractId("OPS-STACK-002".to_string()),
            title: "stack service dependency contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.service_dependency_contract_valid".to_string()),
                title: "service dependency contract entries are parseable and resolve to files",
                kind: TestKind::Pure,
                run: test_ops_stack_002_service_dependency_contract_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-STACK-003".to_string()),
            title: "stack version manifest contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.versions_manifest_schema_valid".to_string()),
                title: "version manifest is parseable and image refs are digest pinned",
                kind: TestKind::Pure,
                run: test_ops_stack_003_versions_manifest_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-STACK-004".to_string()),
            title: "stack dependency graph contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.dependency_graph_generated_acyclic".to_string()),
                title: "dependency graph is parseable and references real cluster/components",
                kind: TestKind::Pure,
                run: test_ops_stack_004_dependency_graph_generated_acyclic,
            }],
        },
        Contract { id: ContractId("OPS-STACK-005".to_string()), title: "stack kind profile consistency contract", tests: vec![TestCase { id: TestId("ops.stack.kind_profiles_consistent".to_string()), title: "dev perf and small kind profiles exist and reference valid cluster configs", kind: TestKind::Pure, run: test_ops_stack_005_kind_profiles_consistent, }] },
        Contract { id: ContractId("OPS-STACK-006".to_string()), title: "stack ports inventory consistency contract", tests: vec![TestCase { id: TestId("ops.stack.ports_inventory_matches_stack".to_string()), title: "ports inventory endpoints are unique and aligned with stack components", kind: TestKind::Pure, run: test_ops_stack_006_ports_inventory_matches_stack, }] },
        Contract { id: ContractId("OPS-STACK-007".to_string()), title: "stack health report generator contract", tests: vec![TestCase { id: TestId("ops.stack.health_report_generator_contract".to_string()), title: "health report sample has schema envelope and stack generator provenance", kind: TestKind::Pure, run: test_ops_stack_007_health_report_generator_contract, }] },
        Contract { id: ContractId("OPS-STACK-008".to_string()), title: "stack command surface contract", tests: vec![TestCase { id: TestId("ops.stack.stack_commands_registered".to_string()), title: "stack command surface snapshot contains up and down verbs", kind: TestKind::Pure, run: test_ops_stack_008_stack_commands_registered, }] },
        Contract { id: ContractId("OPS-STACK-009".to_string()), title: "stack offline profile policy contract", tests: vec![TestCase { id: TestId("ops.stack.offline_profile_policy".to_string()), title: "offline claims require offline or airgap profile coverage", kind: TestKind::Pure, run: test_ops_stack_009_offline_profile_policy, }] },
        Contract {
            id: ContractId("OPS-STACK-E-001".to_string()),
            title: "stack effect kind cluster contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.effect.kind_cluster_up_profile_dev".to_string()),
                title: "effect lane requires kind dev cluster contract inputs",
                kind: TestKind::Subprocess,
                run: test_ops_stack_e_001_kind_cluster_up_profile_dev,
            }],
        },
        Contract {
            id: ContractId("OPS-STACK-E-002".to_string()),
            title: "stack effect component rollout contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.effect.core_components_present".to_string()),
                title: "effect lane requires core stack component manifests",
                kind: TestKind::Subprocess,
                run: test_ops_stack_e_002_core_components_present,
            }],
        },
        Contract {
            id: ContractId("OPS-STACK-E-003".to_string()),
            title: "stack effect ports inventory contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.effect.ports_inventory_mapped".to_string()),
                title: "effect lane requires stack ports inventory contract sample",
                kind: TestKind::Subprocess,
                run: test_ops_stack_e_003_ports_inventory_mapped,
            }],
        },
        Contract {
            id: ContractId("OPS-STACK-E-004".to_string()),
            title: "stack effect health report contract",
            tests: vec![TestCase {
                id: TestId("ops.stack.effect.health_report_generated".to_string()),
                title: "effect lane requires stack health report contract sample",
                kind: TestKind::Subprocess,
                run: test_ops_stack_e_004_stack_health_report_generated,
            }],
        },
    ]
}
