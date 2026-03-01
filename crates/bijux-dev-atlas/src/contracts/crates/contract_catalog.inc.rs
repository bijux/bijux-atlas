pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("CRATES-001".to_string()),
            title: "crate roots include required README and CONTRACT files",
            tests: vec![TestCase {
                id: TestId("crates.docs.root_markdown_contract".to_string()),
                title: "each crate root contains README.md and CONTRACT.md",
                kind: TestKind::Pure,
                run: test_crates_001_each_crate_has_readme_and_contract,
            }],
        },
        Contract {
            id: ContractId("CRATES-002".to_string()),
            title: "crate root markdown files follow strict allowlist",
            tests: vec![TestCase {
                id: TestId("crates.docs.root_markdown_allowlist".to_string()),
                title: "crate roots only contain README.md and CONTRACT.md markdown files",
                kind: TestKind::Pure,
                run: test_crates_002_root_markdown_allowlist,
            }],
        },
        Contract {
            id: ContractId("CRATES-003".to_string()),
            title: "crate docs directory respects markdown file budget",
            tests: vec![TestCase {
                id: TestId("crates.docs.docs_file_budget".to_string()),
                title: "crate docs directories contain at most 15 markdown files",
                kind: TestKind::Pure,
                run: test_crates_003_docs_file_budget,
            }],
        },
        Contract {
            id: ContractId("CRATES-004".to_string()),
            title: "crate docs markdown filenames use lowercase kebab-case",
            tests: vec![TestCase {
                id: TestId("crates.docs.kebab_case_filenames".to_string()),
                title: "crate docs markdown filenames are lowercase kebab-case",
                kind: TestKind::Pure,
                run: test_crates_004_docs_kebab_case_filenames,
            }],
        },
        Contract {
            id: ContractId("CRATES-005".to_string()),
            title: "crate docs relative markdown links resolve",
            tests: vec![TestCase {
                id: TestId("crates.docs.relative_links_resolve".to_string()),
                title: "crate docs links to local relative targets must resolve",
                kind: TestKind::Pure,
                run: test_crates_005_docs_relative_links_resolve,
            }],
        },
        Contract {
            id: ContractId("CRATES-006".to_string()),
            title: "crate readme sections include purpose usage and docs location",
            tests: vec![TestCase {
                id: TestId("crates.docs.readme_required_sections".to_string()),
                title: "crate README includes Purpose, How to use, and Where docs live sections",
                kind: TestKind::Pure,
                run: test_crates_006_readme_has_required_sections,
            }],
        },
        Contract {
            id: ContractId("CRATES-007".to_string()),
            title: "crate contract sections are complete and explicit",
            tests: vec![TestCase {
                id: TestId("crates.contract.required_sections".to_string()),
                title: "crate CONTRACT includes required sections for interface and policy guarantees",
                kind: TestKind::Pure,
                run: test_crates_007_contract_required_sections,
            }],
        },
        Contract {
            id: ContractId("CRATES-008".to_string()),
            title: "crate contract links resolve to existing files",
            tests: vec![TestCase {
                id: TestId("crates.contract.links_resolve".to_string()),
                title: "crate CONTRACT relative links resolve",
                kind: TestKind::Pure,
                run: test_crates_008_contract_links_resolve,
            }],
        },
        Contract {
            id: ContractId("CRATES-009".to_string()),
            title: "crate readme links contract and stays within docs link budget",
            tests: vec![TestCase {
                id: TestId("crates.docs.readme_links_contract".to_string()),
                title: "crate README links CONTRACT and limits docs links",
                kind: TestKind::Pure,
                run: test_crates_009_readme_links_contract_and_budget,
            }],
        },
        Contract {
            id: ContractId("CRATES-010".to_string()),
            title: "crate docs index usage follows explicit allowlist",
            tests: vec![TestCase {
                id: TestId("crates.docs.index_allowlist".to_string()),
                title: "crate docs/index.md appears only for allowlisted crates",
                kind: TestKind::Pure,
                run: test_crates_010_docs_index_allowlist,
            }],
        },
        Contract {
            id: ContractId("CRATES-011".to_string()),
            title: "crate docs avoid forbidden paths and governance/procedure leakage",
            tests: vec![TestCase {
                id: TestId("crates.docs.forbidden_paths".to_string()),
                title: "crate docs avoid forbidden path references and non-crate procedure content",
                kind: TestKind::Pure,
                run: test_crates_011_docs_forbidden_paths_and_terms,
            }],
        },
        Contract {
            id: ContractId("CRATES-012".to_string()),
            title: "crate docs code fences are valid and typed",
            tests: vec![TestCase {
                id: TestId("crates.docs.code_fence_language".to_string()),
                title: "crate docs code fences are balanced and specify language",
                kind: TestKind::Pure,
                run: test_crates_012_docs_code_fence_integrity,
            }],
        },
        Contract {
            id: ContractId("CRATES-013".to_string()),
            title: "crate docs disallow secondary readme without allowlist",
            tests: vec![TestCase {
                id: TestId("crates.docs.docs_readme_forbidden".to_string()),
                title: "crate docs/README.md only allowed for allowlisted crates",
                kind: TestKind::Pure,
                run: test_crates_013_docs_readme_file_forbidden,
            }],
        },
        Contract {
            id: ContractId("CRATES-014".to_string()),
            title: "crate docs files remain under size budget",
            tests: vec![TestCase {
                id: TestId("crates.docs.file_size_limit".to_string()),
                title: "crate docs markdown files remain under line budget",
                kind: TestKind::Pure,
                run: test_crates_014_docs_file_size_limit,
            }],
        },
        Contract {
            id: ContractId("CRATES-015".to_string()),
            title: "published crate docs require allowlist and metadata contract",
            tests: vec![TestCase {
                id: TestId("crates.docs.publish_allowlist".to_string()),
                title: "published crate docs are allowlisted and include ownership metadata and title prefix",
                kind: TestKind::Pure,
                run: test_crates_015_published_docs_contracts,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "CRATES-001" => "Ensures every crate root has canonical documentation entrypoints: README.md and CONTRACT.md.".to_string(),
        "CRATES-002" => "Ensures crate root markdown files are limited to README.md and CONTRACT.md.".to_string(),
        "CRATES-003" => "Ensures each crate docs/ directory stays within the markdown file budget (max 15).".to_string(),
        "CRATES-004" => "Ensures crate docs markdown filenames use lowercase kebab-case.".to_string(),
        "CRATES-005" => "Ensures relative links in crate docs markdown resolve to existing files.".to_string(),
        "CRATES-006" => "Ensures crate README.md has Purpose, How to use, and Where docs live sections.".to_string(),
        "CRATES-007" => "Ensures each crate CONTRACT.md includes required sections for inputs, outputs, invariants, and policy expectations.".to_string(),
        "CRATES-008" => "Ensures every relative link in each crate CONTRACT.md resolves.".to_string(),
        "CRATES-009" => "Ensures each crate README links CONTRACT.md and keeps docs link sprawl under budget.".to_string(),
        "CRATES-010" => "Ensures crate docs/index.md exists only for crates listed in the explicit index allowlist.".to_string(),
        "CRATES-011" => "Ensures crate docs avoid forbidden internal/generated/artifacts references and policy/procedure leakage.".to_string(),
        "CRATES-012" => "Ensures crate docs code fences are balanced and language-tagged.".to_string(),
        "CRATES-013" => "Ensures docs/README.md in crate docs is forbidden unless explicitly allowlisted.".to_string(),
        "CRATES-014" => "Ensures crate docs files stay under a per-file size budget to avoid mega-doc drift.".to_string(),
        "CRATES-015" => "Ensures published crate docs are explicitly allowlisted and include owner/review metadata plus crate-prefixed titles.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts crates`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts crates --mode static"
}
