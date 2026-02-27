// SPDX-License-Identifier: Apache-2.0

fn schema_contracts() -> Vec<Contract> {
    vec![
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
        Contract { id: ContractId("OPS-SCHEMA-006".to_string()), title: "schema id consistency contract", tests: vec![TestCase { id: TestId("ops.schema.id_and_naming_consistency".to_string()), title: "schema files define stable $id values aligned with file names", kind: TestKind::Pure, run: test_ops_schema_006_id_and_naming_consistency, }] },
        Contract { id: ContractId("OPS-SCHEMA-007".to_string()), title: "schema example validation contract", tests: vec![TestCase { id: TestId("ops.schema.examples_validate_required_fields".to_string()), title: "schema examples satisfy required field coverage from compatibility lock", kind: TestKind::Pure, run: test_ops_schema_007_examples_validate_required_fields, }] },
        Contract { id: ContractId("OPS-SCHEMA-008".to_string()), title: "schema intent uniqueness contract", tests: vec![TestCase { id: TestId("ops.schema.forbid_duplicate_intent".to_string()), title: "schema ids and titles are unique to avoid duplicated intent", kind: TestKind::Pure, run: test_ops_schema_008_forbid_duplicate_schema_intent, }] },
        Contract { id: ContractId("OPS-SCHEMA-009".to_string()), title: "schema canonical formatting contract", tests: vec![TestCase { id: TestId("ops.schema.canonical_json_formatting".to_string()), title: "generated schema artifacts use canonical pretty json formatting", kind: TestKind::Pure, run: test_ops_schema_009_canonical_json_formatting, }] },
        Contract { id: ContractId("OPS-SCHEMA-010".to_string()), title: "schema example coverage contract", tests: vec![TestCase { id: TestId("ops.schema.example_coverage".to_string()), title: "schema compatibility targets declare existing example fixtures", kind: TestKind::Pure, run: test_ops_schema_010_example_coverage, }] },
    ]
}
