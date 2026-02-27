// SPDX-License-Identifier: Apache-2.0

fn env_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-ENV-001".to_string()),
            title: "environment overlay schema contract",
            tests: vec![TestCase {
                id: TestId("ops.env.overlays_schema_valid".to_string()),
                title: "all required environment overlays are structurally valid",
                kind: TestKind::Pure,
                run: test_ops_env_001_overlays_schema_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-002".to_string()),
            title: "environment profile completeness contract",
            tests: vec![TestCase {
                id: TestId("ops.env.profiles_complete".to_string()),
                title: "base/ci/dev/prod overlays exist and match profile identity",
                kind: TestKind::Pure,
                run: test_ops_env_002_env_profiles_complete,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-003".to_string()),
            title: "environment key strictness contract",
            tests: vec![TestCase {
                id: TestId("ops.env.no_unknown_keys".to_string()),
                title: "environment overlays reject unknown keys",
                kind: TestKind::Pure,
                run: test_ops_env_003_no_unknown_keys,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-004".to_string()),
            title: "environment overlay merge determinism contract",
            tests: vec![TestCase {
                id: TestId("ops.env.overlay_merge_deterministic".to_string()),
                title: "overlay merge with identical inputs is deterministic across profiles",
                kind: TestKind::Pure,
                run: test_ops_env_004_overlay_merge_deterministic,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-005".to_string()),
            title: "environment prod safety toggles contract",
            tests: vec![TestCase {
                id: TestId("ops.env.prod_forbids_dev_toggles".to_string()),
                title: "prod overlay forbids dev-only effect toggles and unrestricted network",
                kind: TestKind::Pure,
                run: test_ops_env_005_prod_forbids_dev_toggles,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-006".to_string()),
            title: "environment ci effect restriction contract",
            tests: vec![TestCase {
                id: TestId("ops.env.ci_restricts_effects".to_string()),
                title: "ci overlay disables subprocess effects and keeps restricted network mode",
                kind: TestKind::Pure,
                run: test_ops_env_006_ci_restricts_effects,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-007".to_string()),
            title: "environment base defaults contract",
            tests: vec![TestCase {
                id: TestId("ops.env.base_overlay_required_defaults".to_string()),
                title: "base overlay defines required default keys for all profiles",
                kind: TestKind::Pure,
                run: test_ops_env_007_base_overlay_required_defaults,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-008".to_string()),
            title: "environment overlay key stability contract",
            tests: vec![TestCase {
                id: TestId("ops.env.overlay_keys_stable".to_string()),
                title: "overlay key sets are stable and schema-versioned across base/dev/ci/prod",
                kind: TestKind::Pure,
                run: test_ops_env_008_overlay_keys_stable,
            }],
        },
        Contract {
            id: ContractId("OPS-ENV-009".to_string()),
            title: "environment overlays directory boundary contract",
            tests: vec![TestCase {
                id: TestId("ops.env.overlays_dir_no_stray_files".to_string()),
                title: "overlays directory has no stray files and portability matrix is present",
                kind: TestKind::Pure,
                run: test_ops_env_009_overlays_dir_no_stray_files,
            }],
        },
    ]
}
