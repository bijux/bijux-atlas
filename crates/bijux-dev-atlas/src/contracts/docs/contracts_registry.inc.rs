const DOCS_ALLOWED_ROOT_DIRS: [&str; 22] = [
    "_assets",
    "_drafts",
    "_generated",
    "_lint",
    "_nav",
    "_style",
    "adrs",
    "api",
    "architecture",
    "contracts",
    "control-plane",
    "dev",
    "development",
    "docs",
    "engineering",
    "examples",
    "governance",
    "metadata",
    "operations",
    "policies",
    "product",
    "quickstart",
];

const DOCS_ALLOWED_ROOT_DIRS_TAIL: [&str; 6] = [
    "reference",
    "release",
    "root",
    "science",
    "security",
    "start",
];

const DOCS_ALLOWED_ROOT_MARKDOWN: [&str; 9] = [
    "AUTHORS.md",
    "CONTRACT.md",
    "OWNERS.md",
    "PROJECT_DESCRIPTION_SNIPPET.md",
    "PROJECT_IDENTITY.md",
    "START_HERE.md",
    "STYLE.md",
    "index.md",
    "taxonomy-map.md",
];

const DOCS_ALLOWED_ROOT_FILES: [&str; 3] = ["owners.json", "registry.json", "sections.json"];

const DOCS_MAX_DEPTH: usize = 4;
const DOCS_MAX_SIBLINGS: usize = 48;

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("DOC-001".to_string()),
            title: "docs top-level sections stay curated",
            tests: vec![TestCase {
                id: TestId("docs.surface.allowed_root_dirs".to_string()),
                title: "docs top-level directories stay in the allowlist",
                kind: TestKind::Pure,
                run: test_docs_001_allowed_root_dirs,
            }],
        },
        Contract {
            id: ContractId("DOC-002".to_string()),
            title: "docs root markdown stays on the curated surface",
            tests: vec![TestCase {
                id: TestId("docs.surface.allowed_root_markdown".to_string()),
                title: "docs root markdown files stay in the allowlist",
                kind: TestKind::Pure,
                run: test_docs_002_allowed_root_markdown,
            }],
        },
        Contract {
            id: ContractId("DOC-003".to_string()),
            title: "docs paths stay within the depth budget",
            tests: vec![TestCase {
                id: TestId("docs.structure.depth_budget".to_string()),
                title: "docs files stay within the configured depth budget",
                kind: TestKind::Pure,
                run: test_docs_003_depth_budget,
            }],
        },
        Contract {
            id: ContractId("DOC-004".to_string()),
            title: "docs directories stay within the sibling budget",
            tests: vec![TestCase {
                id: TestId("docs.structure.sibling_budget".to_string()),
                title: "docs directories stay within the configured sibling budget",
                kind: TestKind::Pure,
                run: test_docs_004_sibling_budget,
            }],
        },
        Contract {
            id: ContractId("DOC-005".to_string()),
            title: "docs filenames avoid whitespace drift",
            tests: vec![TestCase {
                id: TestId("docs.naming.no_whitespace".to_string()),
                title: "docs file and directory names avoid whitespace",
                kind: TestKind::Pure,
                run: test_docs_005_no_whitespace_names,
            }],
        },
        Contract {
            id: ContractId("DOC-006".to_string()),
            title: "docs canonical entrypoint exists",
            tests: vec![TestCase {
                id: TestId("docs.index.exists".to_string()),
                title: "docs index exists",
                kind: TestKind::Pure,
                run: test_docs_006_index_exists,
            }],
        },
        Contract {
            id: ContractId("DOC-007".to_string()),
            title: "docs root files stay on the declared non-markdown surface",
            tests: vec![TestCase {
                id: TestId("docs.surface.allowed_root_files".to_string()),
                title: "docs root non-markdown files stay in the allowlist",
                kind: TestKind::Pure,
                run: test_docs_007_allowed_root_files,
            }],
        },
        Contract {
            id: ContractId("DOC-008".to_string()),
            title: "docs top-level sections keep declared owners",
            tests: vec![TestCase {
                id: TestId("docs.owners.section_coverage".to_string()),
                title: "docs owners map covers all top-level section directories",
                kind: TestKind::Pure,
                run: test_docs_008_section_owner_coverage,
            }],
        },
        Contract {
            id: ContractId("DOC-009".to_string()),
            title: "docs section manifest stays complete",
            tests: vec![TestCase {
                id: TestId("docs.sections.manifest_complete".to_string()),
                title: "docs section manifest covers every top-level section",
                kind: TestKind::Pure,
                run: test_docs_009_section_manifest_complete,
            }],
        },
        Contract {
            id: ContractId("DOC-010".to_string()),
            title: "docs section entrypoints follow the declared manifest",
            tests: vec![TestCase {
                id: TestId("docs.sections.index_policy".to_string()),
                title: "docs section INDEX.md presence follows the manifest",
                kind: TestKind::Pure,
                run: test_docs_010_section_index_policy,
            }],
        },
    ])
}
