// SPDX-License-Identifier: Apache-2.0

fn inventory_contracts() -> Vec<Contract> {
    vec![
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
            id: ContractId("OPS-INV-006".to_string()),
            title: "inventory contract id format contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_id_format".to_string()),
                title: "all ops contract ids follow OPS-<PILLAR>-NNN format",
                kind: TestKind::Pure,
                run: test_ops_inv_006_contract_id_format,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-007".to_string()),
            title: "inventory gates registry contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.gates_registry_mapped".to_string()),
                title: "gates registry exists and maps each gate to one action id",
                kind: TestKind::Pure,
                run: test_ops_inv_007_gates_registry_mapped,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-008".to_string()),
            title: "inventory drills registry contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.drills_registry_mapped".to_string()),
                title: "drills registry ids map to runnable observe drill definitions",
                kind: TestKind::Pure,
                run: test_ops_inv_008_drills_registry_mapped,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-009".to_string()),
            title: "inventory owners registry contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.owners_registry_complete".to_string()),
                title: "owners registry exists and includes all ops domain directories",
                kind: TestKind::Pure,
                run: test_ops_inv_009_owners_registry_complete,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-010".to_string()),
            title: "inventory schema coverage contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.schema_coverage".to_string()),
                title: "inventory schema directory includes required registry schemas",
                kind: TestKind::Pure,
                run: test_ops_inv_010_inventory_schema_coverage,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-011".to_string()),
            title: "inventory contracts listing pillar coverage contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contracts_listing_matches_pillars".to_string()),
                title: "contracts.json lists CONTRACT.md entries for every declared pillar required_dir",
                kind: TestKind::Pure,
                run: test_ops_inv_011_contracts_listing_matches_pillars,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-001".to_string()),
            title: "inventory contract gate map coverage contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.every_contract_mapped".to_string()),
                title: "every contract id is mapped in contract-gate-map",
                kind: TestKind::Pure,
                run: test_ops_inv_map_001_every_contract_id_mapped,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-002".to_string()),
            title: "inventory contract gate map gate reference contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.mapped_gates_exist".to_string()),
                title: "every mapped gate id exists in gates registry",
                kind: TestKind::Pure,
                run: test_ops_inv_map_002_mapped_gates_exist,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-003".to_string()),
            title: "inventory contract gate map command surface contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.mapped_commands_registered".to_string()),
                title: "every mapped command is registered in ops command surface",
                kind: TestKind::Pure,
                run: test_ops_inv_map_003_mapped_commands_registered,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-004".to_string()),
            title: "inventory contract gate map effects annotation contract",
            tests: vec![TestCase {
                id: TestId(
                    "ops.inventory.contract_gate_map.effects_annotation_matches_mode".to_string(),
                ),
                title: "effects annotations match contract test kinds and execution mode",
                kind: TestKind::Pure,
                run: test_ops_inv_map_004_effects_annotation_matches_contract_mode,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-005".to_string()),
            title: "inventory contract gate map orphan gate contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.no_orphan_gates".to_string()),
                title: "every gate id is referenced by at least one contract mapping",
                kind: TestKind::Pure,
                run: test_ops_inv_map_005_no_orphan_gates,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-006".to_string()),
            title: "inventory contract gate map orphan contract contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.no_orphan_contracts".to_string()),
                title: "every contract maps to gate ids or is explicitly static-only",
                kind: TestKind::Pure,
                run: test_ops_inv_map_006_no_orphan_contracts,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-007".to_string()),
            title: "inventory contract gate map static purity contract",
            tests: vec![TestCase {
                id: TestId(
                    "ops.inventory.contract_gate_map.static_only_contracts_are_pure".to_string(),
                ),
                title: "static-only mappings are restricted to pure test contracts",
                kind: TestKind::Pure,
                run: test_ops_inv_map_007_static_only_contracts_are_pure,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-008".to_string()),
            title: "inventory contract gate map effect kind contract",
            tests: vec![TestCase {
                id: TestId(
                    "ops.inventory.contract_gate_map.effect_contracts_require_effect_kind"
                        .to_string(),
                ),
                title: "effect mappings require subprocess or network test kinds and annotations",
                kind: TestKind::Pure,
                run: test_ops_inv_map_008_effect_contracts_require_effect_kind,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-009".to_string()),
            title: "inventory contract gate map explain coverage contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.explain_shows_mapped_gates".to_string()),
                title: "contract explain source mappings expose gate ids for non-static contracts",
                kind: TestKind::Pure,
                run: test_ops_inv_map_009_explain_shows_mapped_gates,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-010".to_string()),
            title: "inventory contract gate map canonical order contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.mapping_sorted_canonical".to_string()),
                title: "contract-gate-map is sorted by contract id and canonical json",
                kind: TestKind::Pure,
                run: test_ops_inv_map_010_mapping_sorted_canonical,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-MAP-011".to_string()),
            title: "inventory contract gate map effect metadata contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_gate_map.effect_metadata_declared".to_string()),
                title: "effect mappings declare external tools, artifacts, and reproducible commands",
                kind: TestKind::Pure,
                run: test_ops_inv_map_011_effect_metadata_declared,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-PILLARS-001".to_string()),
            title: "inventory pillars registry contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.pillars.exists_and_validates".to_string()),
                title: "pillars.json exists and validates",
                kind: TestKind::Pure,
                run: test_ops_inv_pillars_001_exists_and_validates,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-PILLARS-002".to_string()),
            title: "inventory pillar directory contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.pillars.every_pillar_dir_exists".to_string()),
                title: "every declared non-root pillar has a matching ops directory",
                kind: TestKind::Pure,
                run: test_ops_inv_pillars_002_every_pillar_dir_exists,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-PILLARS-003".to_string()),
            title: "inventory pillar strictness contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.pillars.no_extra_pillar_dirs".to_string()),
                title: "ops root has no undeclared pillar directories",
                kind: TestKind::Pure,
                run: test_ops_inv_pillars_003_no_extra_pillar_dirs,
            }],
        },
        Contract {
            id: ContractId("OPS-INV-DEBT-001".to_string()),
            title: "inventory contract debt registry contract",
            tests: vec![TestCase {
                id: TestId("ops.inventory.contract_debt.exists_and_complete".to_string()),
                title: "contract debt registry exists with owner and target milestone for each entry",
                kind: TestKind::Pure,
                run: test_ops_inv_debt_001_debt_list_exists_and_complete,
            }],
        },
    ]
}
