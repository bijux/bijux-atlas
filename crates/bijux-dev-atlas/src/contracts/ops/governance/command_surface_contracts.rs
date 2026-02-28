// SPDX-License-Identifier: Apache-2.0

fn root_surface_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-ROOT-017".to_string()),
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
            id: ContractId("OPS-ROOT-018".to_string()),
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
            id: ContractId("OPS-ROOT-019".to_string()),
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
            id: ContractId("OPS-ROOT-020".to_string()),
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
            id: ContractId("OPS-ROOT-021".to_string()),
            title: "ops docs ssot boundary contract",
            tests: vec![TestCase {
                id: TestId("ops.docs.readme_ssot_boundary".to_string()),
                title: "ops root readme remains navigation-only and references docs/operations",
                kind: TestKind::Pure,
                run: test_ops_004_readme_ssot_boundary,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-022".to_string()),
            title: "ops contract document generation contract",
            tests: vec![TestCase {
                id: TestId("ops.contract_doc.generated_match".to_string()),
                title: "ops CONTRACT.md matches generated output from contract registry",
                kind: TestKind::Pure,
                run: test_ops_contract_doc_generated_match,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-001".to_string()),
            title: "ops root allowed surface contract",
            tests: vec![TestCase {
                id: TestId("ops.root.allowed_surface".to_string()),
                title: "ops root contains only canonical files and domain directories",
                kind: TestKind::Pure,
                run: test_ops_root_001_allowed_surface,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-002".to_string()),
            title: "ops root markdown contract",
            tests: vec![TestCase {
                id: TestId("ops.root.forbid_extra_markdown".to_string()),
                title: "ops root forbids markdown files other than README.md and CONTRACT.md",
                kind: TestKind::Pure,
                run: test_ops_root_002_forbid_extra_root_markdown,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-003".to_string()),
            title: "ops no shell scripts contract",
            tests: vec![TestCase {
                id: TestId("ops.root.no_shell_script_files".to_string()),
                title: "ops tree contains no shell script files or bash shebangs",
                kind: TestKind::Pure,
                run: test_ops_root_003_no_shell_script_files,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-004".to_string()),
            title: "ops max directory depth contract",
            tests: vec![TestCase {
                id: TestId("ops.root.max_directory_depth".to_string()),
                title: "ops file paths remain within configured depth budget",
                kind: TestKind::Pure,
                run: test_ops_root_004_max_directory_depth,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-005".to_string()),
            title: "ops filename policy contract",
            tests: vec![TestCase {
                id: TestId("ops.root.filename_policy".to_string()),
                title: "ops filenames follow stable lowercase policy with explicit allowlist exceptions",
                kind: TestKind::Pure,
                run: test_ops_root_005_filename_policy,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-006".to_string()),
            title: "ops generated gitignore policy contract",
            tests: vec![TestCase {
                id: TestId("ops.root.generated_gitignore_policy".to_string()),
                title: "ops/_generated is gitignored with explicit .gitkeep exception",
                kind: TestKind::Pure,
                run: test_ops_root_006_generated_gitignore_policy,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-007".to_string()),
            title: "ops generated example secret guard contract",
            tests: vec![TestCase {
                id: TestId("ops.root.generated_example_secret_guard".to_string()),
                title: "ops/_generated.example is secret-free and json payloads are parseable",
                kind: TestKind::Pure,
                run: test_ops_root_007_generated_example_secret_guard,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-008".to_string()),
            title: "ops placeholder directory contract",
            tests: vec![TestCase {
                id: TestId("ops.root.placeholder_dirs_allowlist".to_string()),
                title: "ops placeholder directories are explicitly allowlisted",
                kind: TestKind::Pure,
                run: test_ops_root_008_placeholder_dirs_allowlist,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-009".to_string()),
            title: "ops policy inventory coverage contract",
            tests: vec![TestCase {
                id: TestId("ops.root.policy_files_inventory_coverage".to_string()),
                title: "ops policy/config files are covered by inventory sources",
                kind: TestKind::Pure,
                run: test_ops_root_009_inventory_coverage_for_policy_files,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-010".to_string()),
            title: "ops deleted doc name guard contract",
            tests: vec![TestCase {
                id: TestId("ops.root.forbid_deleted_doc_names".to_string()),
                title: "forbidden legacy ops markdown names must not be reintroduced",
                kind: TestKind::Pure,
                run: test_ops_root_010_forbid_deleted_doc_names,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-023".to_string()),
            title: "operations docs policy linkage contract",
            tests: vec![
                TestCase {
                    id: TestId("ops.docs.policy_keyword_requires_contract_id".to_string()),
                    title: "operations docs with policy keywords must reference OPS contract ids",
                    kind: TestKind::Pure,
                    run: test_ops_docs_001_policy_keyword_requires_contract_id,
                },
                TestCase {
                    id: TestId("ops.docs.index_crosslinks_contracts".to_string()),
                    title: "operations index must state docs/contracts boundary and include OPS references",
                    kind: TestKind::Pure,
                    run: test_ops_docs_002_index_crosslinks_contracts,
                },
            ],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-001".to_string()),
            title: "ops root command surface required commands contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.required_commands_exist".to_string()),
                title: "required ops command verbs exist in command surface registry",
                kind: TestKind::Pure,
                run: test_ops_root_surface_001_required_commands_exist,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-002".to_string()),
            title: "ops root command surface no hidden commands contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.no_hidden_commands".to_string()),
                title: "listed commands and action dispatch entries must match exactly",
                kind: TestKind::Pure,
                run: test_ops_root_surface_002_no_hidden_commands,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-003".to_string()),
            title: "ops root command surface deterministic ordering contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.surface_ordering_deterministic".to_string()),
                title: "command surface list is sorted deterministically",
                kind: TestKind::Pure,
                run: test_ops_root_surface_003_surface_ordering_deterministic,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-004".to_string()),
            title: "ops root command surface effects declaration contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.commands_declare_effects".to_string()),
                title: "mapped commands declare effects_required annotations",
                kind: TestKind::Pure,
                run: test_ops_root_surface_004_commands_declare_effects,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-005".to_string()),
            title: "ops root command surface pillar grouping contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.commands_grouped_by_pillar".to_string()),
                title: "ops command actions use approved pillar-style domain groups",
                kind: TestKind::Pure,
                run: test_ops_root_surface_005_commands_grouped_by_pillar,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-006".to_string()),
            title: "ops root command surface ad-hoc group guard contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.forbid_adhoc_command_groups".to_string()),
                title: "ops command actions must not use ad-hoc misc/util group names",
                kind: TestKind::Pure,
                run: test_ops_root_surface_006_forbid_adhoc_command_groups,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-007".to_string()),
            title: "ops root command surface purpose contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.command_purpose_defined".to_string()),
                title: "each command action defines a stable purpose string",
                kind: TestKind::Pure,
                run: test_ops_root_surface_007_command_purpose_defined,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-008".to_string()),
            title: "ops root command surface json output contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.command_supports_json".to_string()),
                title: "each command action declares json output support",
                kind: TestKind::Pure,
                run: test_ops_root_surface_008_command_supports_json,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-009".to_string()),
            title: "ops root command surface dry-run policy contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.command_dry_run_policy".to_string()),
                title: "each command action declares dry-run policy where applicable",
                kind: TestKind::Pure,
                run: test_ops_root_surface_009_command_dry_run_policy,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-SURFACE-010".to_string()),
            title: "ops root command surface artifacts policy contract",
            tests: vec![TestCase {
                id: TestId("ops.root_surface.artifacts_root_policy".to_string()),
                title: "each command action declares artifacts root write policy",
                kind: TestKind::Pure,
                run: test_ops_root_surface_010_artifacts_root_policy,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-011".to_string()),
            title: "ops markdown allowlist contract",
            tests: vec![TestCase {
                id: TestId("ops.root.markdown_allowlist_only".to_string()),
                title: "ops markdown files are restricted to explicit allowlist paths",
                kind: TestKind::Pure,
                run: test_ops_root_011_markdown_allowlist_only,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-012".to_string()),
            title: "ops pillar readme cardinality contract",
            tests: vec![TestCase {
                id: TestId("ops.root.single_readme_per_pillar".to_string()),
                title: "each non-root pillar has exactly one README.md at pillar root",
                kind: TestKind::Pure,
                run: test_ops_root_012_single_readme_per_pillar,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-013".to_string()),
            title: "ops markdown allowlist inventory contract",
            tests: vec![TestCase {
                id: TestId("ops.root.markdown_allowlist_file_valid".to_string()),
                title: "markdown allowlist inventory file exists and is non-empty",
                kind: TestKind::Pure,
                run: test_ops_root_013_markdown_allowlist_file_valid,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-014".to_string()),
            title: "ops procedure text contract",
            tests: vec![TestCase {
                id: TestId("ops.root.no_procedure_docs_in_ops".to_string()),
                title: "procedure-like language in ops markdown requires OPS contract references",
                kind: TestKind::Pure,
                run: test_ops_root_014_no_procedure_docs_in_ops,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-015".to_string()),
            title: "ops pillar markdown minimalism contract",
            tests: vec![TestCase {
                id: TestId("ops.root.no_extra_pillar_markdown".to_string()),
                title: "ops pillar markdown surface is restricted to allowlisted files",
                kind: TestKind::Pure,
                run: test_ops_root_015_no_extra_pillar_markdown,
            }],
        },
        Contract {
            id: ContractId("OPS-ROOT-016".to_string()),
            title: "ops deleted markdown denylist contract",
            tests: vec![TestCase {
                id: TestId("ops.root.deleted_markdown_denylist".to_string()),
                title: "historically deleted markdown paths must not be reintroduced",
                kind: TestKind::Pure,
                run: test_ops_root_016_deleted_markdown_denylist,
            }],
        },
    ]
}
