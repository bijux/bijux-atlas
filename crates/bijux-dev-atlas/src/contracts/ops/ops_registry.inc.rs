pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("OPS-000".to_string()),
            title: "ops directory contract",
            tests: vec![
                TestCase {
                    id: TestId("ops.dir.allowed_root_files".to_string()),
                    title: "ops root allows only contract/readme root files",
                    kind: TestKind::Pure,
                    run: test_ops_000_allowed_root_files,
                },
                TestCase {
                    id: TestId("ops.dir.forbid_extra_markdown_root".to_string()),
                    title: "ops root forbids extra markdown",
                    kind: TestKind::Pure,
                    run: test_ops_000_forbid_extra_markdown_root,
                },
                TestCase {
                    id: TestId("ops.dir.allow_only_known_domain_dirs".to_string()),
                    title: "ops root allows only canonical domain directories",
                    kind: TestKind::Pure,
                    run: test_ops_000_allow_only_known_domain_dirs,
                },
                TestCase {
                    id: TestId("ops.dir.forbid_extra_markdown_recursive".to_string()),
                    title: "ops forbids recursive markdown outside approved surface",
                    kind: TestKind::Pure,
                    run: test_ops_000_forbid_extra_markdown_recursive,
                },
            ],
        },
        Contract {
            id: ContractId("OPS-001".to_string()),
            title: "ops generated lifecycle contract",
            tests: vec![
                TestCase {
                    id: TestId("ops.generated.runtime.allowed_files".to_string()),
                    title: "ops/_generated allows only runtime artifact formats",
                    kind: TestKind::Pure,
                    run: test_ops_001_generated_runtime_allowed_files,
                },
                TestCase {
                    id: TestId("ops.generated.example.allowed_files".to_string()),
                    title: "ops/_generated.example allows only committed artifact formats",
                    kind: TestKind::Pure,
                    run: test_ops_001_generated_example_allowed_files,
                },
                TestCase {
                    id: TestId("ops.generated.runtime.no_example_files".to_string()),
                    title: "ops/_generated forbids example artifacts",
                    kind: TestKind::Pure,
                    run: test_ops_001_generated_runtime_forbid_example_files,
                },
            ],
        },
        Contract {
            id: ContractId("OPS-002".to_string()),
            title: "ops required domain files contract",
            tests: vec![
                TestCase {
                    id: TestId("ops.domain.required_contract_and_readme".to_string()),
                    title: "each ops domain includes README.md and CONTRACT.md",
                    kind: TestKind::Pure,
                    run: test_ops_002_domain_required_files,
                },
                TestCase {
                    id: TestId("ops.domain.forbid_legacy_docs".to_string()),
                    title: "legacy domain INDEX/OWNER/REQUIRED markdown files are forbidden",
                    kind: TestKind::Pure,
                    run: test_ops_002_forbid_legacy_domain_docs,
                },
            ],
        },
        Contract {
            id: ContractId("OPS-003".to_string()),
            title: "ops markdown budget contract",
            tests: vec![
                TestCase {
                    id: TestId("ops.markdown_budget.readme".to_string()),
                    title: "README markdown files stay within line budget",
                    kind: TestKind::Pure,
                    run: test_ops_003_readme_markdown_budget,
                },
                TestCase {
                    id: TestId("ops.markdown_budget.contract".to_string()),
                    title: "CONTRACT markdown files stay within line budget",
                    kind: TestKind::Pure,
                    run: test_ops_003_contract_markdown_budget,
                },
            ],
        },
        Contract {
            id: ContractId("OPS-004".to_string()),
            title: "ops docs ssot boundary contract",
            tests: vec![TestCase {
                id: TestId("ops.docs.readme_ssot_boundary".to_string()),
                title: "ops root readme remains navigation-only and references docs/operations",
                kind: TestKind::Pure,
                run: test_ops_004_readme_ssot_boundary,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-001".to_string()),
            title: "inventory completeness contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.completeness".to_string()),
                title: "inventory registers all domains and policy files",
                kind: TestKind::Pure,
                run: test_ops_inv_001_inventory_completeness,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-002".to_string()),
            title: "inventory orphan files contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.no_orphan_files".to_string()),
                title: "ops files must be mapped through inventory sources",
                kind: TestKind::Pure,
                run: test_ops_inv_002_no_orphan_files,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-003".to_string()),
            title: "inventory duplicate source contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.no_duplicate_ssot_sources".to_string()),
                title: "duplicate ssot markdown sources are forbidden",
                kind: TestKind::Pure,
                run: test_ops_inv_003_no_duplicate_ssot,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-004".to_string()),
            title: "inventory authority tier contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.authority_tiers_enforced".to_string()),
                title: "authority tier exceptions are structured and expiry-bound",
                kind: TestKind::Pure,
                run: test_ops_inv_004_authority_tiers_enforced,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-005".to_string()),
            title: "inventory control graph contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.control_graph_validated".to_string()),
                title: "control graph edges and node mappings are valid and acyclic",
                kind: TestKind::Pure,
                run: test_ops_inv_005_control_graph_validated,
            }],
        },
        Contract {
            id: ContractId("OPS-SCHEMA-001".to_string()),
            title: "schema parseability contract",
            tests: vec![TestCase {
                id: TestId("ops.schema.parseable_documents".to_string()),
                title: "ops json/yaml policy documents are parseable",
                kind: TestKind::Pure,
                run: test_ops_schema_001_parseable_documents,
            }],
        },
        Contract {
            id: ContractId("OPS-SCHEMA-002".to_string()),
            title: "schema index completeness contract",
            tests: vec![TestCase {
                id: TestId("ops.schema.index_complete".to_string()),
                title: "generated schema index covers all schema sources",
                kind: TestKind::Pure,
                run: test_ops_schema_002_schema_index_complete,
            }],
        },
        Contract {
            id: ContractId("OPS-SCHEMA-003".to_string()),
            title: "schema naming contract",
            tests: vec![TestCase {
                id: TestId("ops.schema.no_unversioned".to_string()),
                title: "schema sources use stable .schema.json naming",
                kind: TestKind::Pure,
                run: test_ops_schema_003_no_unversioned_schemas,
            }],
        },
        Contract {
            id: ContractId("OPS-SCHEMA-004".to_string()),
            title: "schema budget contract",
            tests: vec![TestCase {
                id: TestId("ops.schema.budget_policy".to_string()),
                title: "schema count stays within per-domain budgets",
                kind: TestKind::Pure,
                run: test_ops_schema_004_budget_policy,
            }],
        },
        Contract {
            id: ContractId("OPS-SCHEMA-005".to_string()),
            title: "schema evolution lock contract",
            tests: vec![TestCase {
                id: TestId("ops.schema.evolution_lock".to_string()),
                title: "compatibility lock tracks schema evolution targets",
                kind: TestKind::Pure,
                run: test_ops_schema_005_evolution_lock,
            }],
        },
        Contract {
            id: ContractId("OPS-DATASET-001".to_string()),
            title: "datasets manifest lock contract",
            tests: vec![TestCase {
                id: TestId("ops.dataset.manifest_and_lock_consistent".to_string()),
                title: "dataset manifest and lock ids are consistent",
                kind: TestKind::Pure,
                run: test_ops_dataset_001_manifest_and_lock,
            }],
        },
        Contract {
            id: ContractId("OPS-DATASET-002".to_string()),
            title: "datasets fixture inventory contract",
            tests: vec![TestCase {
                id: TestId("ops.dataset.fixture_inventory_matches_disk".to_string()),
                title: "fixture inventory matches fixture directories and references",
                kind: TestKind::Pure,
                run: test_ops_dataset_002_fixture_inventory_matches_disk,
            }],
        },
        Contract {
            id: ContractId("OPS-DATASET-003".to_string()),
            title: "datasets fixture drift promotion contract",
            tests: vec![TestCase {
                id: TestId("ops.dataset.no_fixture_drift_without_promotion_record".to_string()),
                title: "fixture drift requires explicit promotion rule coverage",
                kind: TestKind::Pure,
                run: test_ops_dataset_003_no_fixture_drift_without_promotion_record,
            }],
        },
        Contract {
            id: ContractId("OPS-DATASET-004".to_string()),
            title: "datasets release diff determinism contract",
            tests: vec![TestCase {
                id: TestId("ops.dataset.release_diff_fixtures_deterministic".to_string()),
                title: "release-diff fixture lock and golden payloads are deterministic",
                kind: TestKind::Pure,
                run: test_ops_dataset_004_release_diff_fixtures_are_deterministic,
            }],
        },
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
            id: ContractId("OPS-K8S-001".to_string()),
            title: "k8s static chart render contract",
            tests: vec![TestCase {
                id: TestId("ops.k8s.chart_renders_static".to_string()),
                title: "helm chart source includes required files and static render inputs",
                kind: TestKind::Pure,
                run: test_ops_k8s_001_chart_renders_static,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> &'static str {
    match contract_id {
        "OPS-000" => "Defines the only allowed root ops filesystem surface and markdown envelope.",
        "OPS-001" => "Governs generated artifact lifecycle boundaries under _generated and _generated.example.",
        "OPS-002" => "Requires per-domain README/CONTRACT and forbids legacy duplicate docs.",
        "OPS-003" => "Enforces markdown size budgets so contracts stay concise and reviewable.",
        "OPS-004" => "Enforces SSOT boundary: ops README is navigation-only and points to docs/operations.",
        "OPS-INV-001" => "Ensures domain and policy registration completeness in inventory sources.",
        "OPS-INV-002" => "Prevents orphan ops files that are not mapped by inventory references.",
        "OPS-INV-003" => "Forbids duplicate SSOT markdown documents when inventory is authoritative.",
        "OPS-INV-004" => "Enforces authority-tier exception structure with explicit expiry metadata.",
        "OPS-INV-005" => "Validates inventory control-graph integrity, mappings, and cycle safety.",
        "OPS-SCHEMA-001" => "Ensures ops json/yaml policy documents are parseable.",
        "OPS-SCHEMA-002" => "Ensures generated schema index matches on-disk schema sources.",
        "OPS-SCHEMA-003" => "Enforces stable .schema.json naming for schema source files.",
        "OPS-SCHEMA-004" => "Enforces per-domain schema count budgets to limit sprawl.",
        "OPS-SCHEMA-005" => "Requires compatibility lock coverage for schema evolution governance.",
        "OPS-DATASET-001" => "Ensures datasets manifest and lock use the same dataset id set.",
        "OPS-DATASET-002" => "Ensures fixture inventory maps cleanly to fixture directories and references.",
        "OPS-DATASET-003" => "Requires fixture drift to be covered by explicit promotion policy metadata.",
        "OPS-DATASET-004" => "Ensures release-diff fixture assets and goldens stay deterministic and pinned.",
        "OPS-E2E-001" => "Ensures e2e suites map to concrete scenario ids and runnable scripts.",
        "OPS-E2E-002" => "Ensures smoke manifest structure and lock references are valid.",
        "OPS-E2E-003" => "Ensures fixture lock digest and allowlist file policy remain consistent.",
        "OPS-E2E-004" => "Ensures realdata snapshots are parseable and pinned to canonical queries.",
        "OPS-ENV-001" => "Ensures required environment overlays satisfy structural schema rules.",
        "OPS-ENV-002" => "Ensures base/ci/dev/prod overlay coverage and profile identity consistency.",
        "OPS-ENV-003" => "Enforces strict known-key policy for environment overlay payloads.",
        "OPS-K8S-001" => "Ensures helm chart sources expose static render prerequisites.",
        _ => "No explanation registered for this contract id.",
    }
}
