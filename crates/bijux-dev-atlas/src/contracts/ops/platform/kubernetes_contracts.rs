// SPDX-License-Identifier: Apache-2.0

fn k8s_contracts() -> Vec<Contract> {
    vec![
        Contract { id: ContractId("OPS-K8S-001".to_string()), title: "k8s static chart render contract", tests: vec![TestCase { id: TestId("ops.k8s.chart_renders_static".to_string()), title: "helm chart source includes required files and static render inputs", kind: TestKind::Pure, run: test_ops_k8s_001_chart_renders_static, }] },
        Contract { id: ContractId("OPS-K8S-002".to_string()), title: "k8s values schema validation contract", tests: vec![TestCase { id: TestId("ops.k8s.values_files_validate_schema".to_string()), title: "install-matrix values files exist and are parseable against chart schema surface", kind: TestKind::Pure, run: test_ops_k8s_002_values_files_validate_schema, }] },
        Contract { id: ContractId("OPS-K8S-003".to_string()), title: "k8s install matrix completeness contract", tests: vec![TestCase { id: TestId("ops.k8s.install_matrix_complete".to_string()), title: "install matrix covers canonical profile set and references existing files", kind: TestKind::Pure, run: test_ops_k8s_003_install_matrix_complete, }] },
        Contract { id: ContractId("OPS-K8S-004".to_string()), title: "k8s forbidden object policy contract", tests: vec![TestCase { id: TestId("ops.k8s.no_forbidden_k8s_objects".to_string()), title: "helm templates must not introduce forbidden cluster-scope object kinds", kind: TestKind::Pure, run: test_ops_k8s_004_no_forbidden_k8s_objects, }] },
        Contract { id: ContractId("OPS-K8S-005".to_string()), title: "k8s rbac minimalism contract", tests: vec![TestCase { id: TestId("ops.k8s.rbac_minimalism".to_string()), title: "helm templates must not declare cluster-admin or wildcard rbac grants", kind: TestKind::Pure, run: test_ops_k8s_005_rbac_minimalism, }] },
        Contract { id: ContractId("OPS-K8S-006".to_string()), title: "k8s pod security and probes contract", tests: vec![TestCase { id: TestId("ops.k8s.pod_security_and_probes".to_string()), title: "deployment template includes pod security hardening and readiness/liveness probes", kind: TestKind::Pure, run: test_ops_k8s_006_pod_security_and_probes, }] },
        Contract { id: ContractId("OPS-K8S-007".to_string()), title: "k8s rollout safety contract", tests: vec![TestCase { id: TestId("ops.k8s.rollout_safety_enforced".to_string()), title: "rollout safety contract is valid and rollout template enforces rollout steps", kind: TestKind::Pure, run: test_ops_k8s_007_rollout_safety_enforced, }] },
        Contract { id: ContractId("OPS-K8S-008".to_string()), title: "k8s conformance suite contract", tests: vec![TestCase { id: TestId("ops.k8s.conformance_suite_runnable".to_string()), title: "k8s conformance suite exists and control-plane exposes conformance verb", kind: TestKind::Pure, run: test_ops_k8s_008_conformance_suite_runnable, }] },
        Contract { id: ContractId("OPS-K8S-009".to_string()), title: "k8s install matrix generated consistency contract", tests: vec![TestCase { id: TestId("ops.k8s.install_matrix_and_generated_consistency".to_string()), title: "install matrix and generated k8s artifacts stay aligned and schema-versioned", kind: TestKind::Pure, run: test_ops_k8s_009_install_matrix_and_generated_consistency, }] },
        Contract { id: ContractId("OPS-K8S-010".to_string()), title: "k8s generated index determinism contract", tests: vec![TestCase { id: TestId("ops.k8s.generated_indexes_deterministic_schema_valid".to_string()), title: "generated k8s indexes are schema-versioned and deterministic", kind: TestKind::Pure, run: test_ops_k8s_010_generated_indexes_deterministic_schema_valid, }] },
        Contract {
            id: ContractId("OPS-K8S-E-001".to_string()),
            title: "k8s effect chart defaults render contract",
            tests: vec![TestCase {
                id: TestId("ops.k8s.effect.chart_defaults_rendered".to_string()),
                title: "effect lane renders chart defaults with helm template and emits rendered manifest",
                kind: TestKind::Subprocess,
                run: test_ops_k8s_e_001_chart_defaults_rendered,
            }],
        },
        Contract {
            id: ContractId("OPS-K8S-E-002".to_string()),
            title: "k8s effect minimal values render contract",
            tests: vec![TestCase {
                id: TestId("ops.k8s.effect.chart_minimal_values_rendered".to_string()),
                title: "effect lane renders a minimal chart values profile",
                kind: TestKind::Subprocess,
                run: test_ops_k8s_e_002_chart_minimal_values_rendered,
            }],
        },
        Contract {
            id: ContractId("OPS-K8S-E-003".to_string()),
            title: "k8s effect kubeconform contract",
            tests: vec![TestCase {
                id: TestId("ops.k8s.effect.kubeconform_render_validation".to_string()),
                title: "effect lane validates rendered manifests with kubeconform",
                kind: TestKind::Subprocess,
                run: test_ops_k8s_e_003_kubeconform_render_validation,
            }],
        },
        Contract {
            id: ContractId("OPS-K8S-E-004".to_string()),
            title: "k8s effect install matrix contract",
            tests: vec![TestCase {
                id: TestId("ops.k8s.effect.helm_install_contract_defined".to_string()),
                title: "effect lane requires kind install profile in k8s install matrix",
                kind: TestKind::Subprocess,
                run: test_ops_k8s_e_004_helm_install_contract_defined,
            }],
        },
        Contract {
            id: ContractId("OPS-K8S-E-005".to_string()),
            title: "k8s effect rollout safety contract",
            tests: vec![TestCase {
                id: TestId("ops.k8s.effect.rollout_safety_contract_satisfied".to_string()),
                title: "effect lane requires rollout safety contract checks",
                kind: TestKind::Subprocess,
                run: test_ops_k8s_e_005_rollout_safety_contract_satisfied,
            }],
        },
        Contract {
            id: ContractId("OPS-K8S-E-006".to_string()),
            title: "k8s effect tool versions contract",
            tests: vec![TestCase {
                id: TestId("ops.k8s.effect.tool_versions_recorded".to_string()),
                title: "effect lane records helm and kubeconform versions and enforces allowed major versions",
                kind: TestKind::Subprocess,
                run: test_ops_k8s_e_006_tool_versions_recorded,
            }],
        },
    ]
}
