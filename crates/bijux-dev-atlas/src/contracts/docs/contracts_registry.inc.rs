const DOCS_ALLOWED_ROOT_DIRS: [&str; 10] = [
    "_assets",
    "_drafts",
    "_internal",
    "api",
    "architecture",
    "control-plane",
    "development",
    "operations",
    "product",
    "reference",
];

const DOCS_ALLOWED_ROOT_DIRS_TAIL: [&str; 0] = [];

const DOCS_ALLOWED_ROOT_MARKDOWN: [&str; 6] = [
    "glossary.md",
    "index.md",
    "start-here.md",
    "site-map.md",
    "what-to-read-next.md",
    "README.md",
];

const DOCS_ALLOWED_ROOT_FILES: [&str; 4] = ["owners.json", "registry.json", "sections.json", "redirects.json"];

const DOCS_MAX_DEPTH: usize = 4;
const DOCS_MAX_SIBLINGS: usize = 49;

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
        Contract {
            id: ContractId("DOC-011".to_string()),
            title: "docs section index links resolve",
            tests: vec![TestCase {
                id: TestId("docs.links.section_indexes_resolve".to_string()),
                title: "links in top-level section index pages resolve to real files",
                kind: TestKind::Pure,
                run: test_docs_011_section_index_links_resolve,
            }],
        },
        Contract {
            id: ContractId("DOC-012".to_string()),
            title: "docs root entrypoint links resolve",
            tests: vec![TestCase {
                id: TestId("docs.links.root_entrypoints_resolve".to_string()),
                title: "links in docs root entrypoint pages resolve to real files",
                kind: TestKind::Pure,
                run: test_docs_012_root_entrypoint_links_resolve,
            }],
        },
        Contract {
            id: ContractId("DOC-013".to_string()),
            title: "docs entrypoint pages declare owner metadata",
            tests: vec![TestCase {
                id: TestId("docs.metadata.entrypoint_owner".to_string()),
                title: "docs entrypoint pages include a non-empty owner field",
                kind: TestKind::Pure,
                run: test_docs_013_entrypoint_owner,
            }],
        },
        Contract {
            id: ContractId("DOC-014".to_string()),
            title: "docs entrypoint page stability values stay normalized",
            tests: vec![TestCase {
                id: TestId("docs.metadata.entrypoint_stability".to_string()),
                title: "docs entrypoint pages use only approved stability values when declared",
                kind: TestKind::Pure,
                run: test_docs_014_entrypoint_stability,
            }],
        },
        Contract {
            id: ContractId("DOC-015".to_string()),
            title: "deprecated docs entrypoints name a replacement path",
            tests: vec![TestCase {
                id: TestId("docs.metadata.deprecated_replacement".to_string()),
                title: "deprecated docs entrypoint pages include replacement guidance",
                kind: TestKind::Pure,
                run: test_docs_015_deprecated_replacement,
            }],
        },
        Contract {
            id: ContractId("DOC-016".to_string()),
            title: "docs section entrypoint owners align with the owner registry",
            tests: vec![TestCase {
                id: TestId("docs.metadata.section_owner_alignment".to_string()),
                title: "required section index pages use the owner declared in docs/owners.json",
                kind: TestKind::Pure,
                run: test_docs_016_section_owner_alignment,
            }],
        },
        Contract {
            id: ContractId("DOC-017".to_string()),
            title: "docs section manifest declares root entrypoint coverage",
            tests: vec![TestCase {
                id: TestId("docs.sections.root_entrypoint_flags".to_string()),
                title: "indexed docs sections declare whether they must appear in docs/index.md",
                kind: TestKind::Pure,
                run: test_docs_017_root_entrypoint_flags,
            }],
        },
        Contract {
            id: ContractId("DOC-018".to_string()),
            title: "docs root entrypoint links every declared root section",
            tests: vec![TestCase {
                id: TestId("docs.index.root_section_coverage".to_string()),
                title: "docs/index.md links every section marked for root entrypoint coverage",
                kind: TestKind::Pure,
                run: test_docs_018_root_section_coverage,
            }],
        },
        Contract {
            id: ContractId("DOC-019".to_string()),
            title: "docs entrypoint pages stay within the word budget",
            tests: vec![TestCase {
                id: TestId("docs.quality.entrypoint_word_budget".to_string()),
                title: "docs entrypoint pages stay within the approved word budget",
                kind: TestKind::Pure,
                run: test_docs_019_entrypoint_word_budget,
            }],
        },
        Contract {
            id: ContractId("DOC-020".to_string()),
            title: "stable docs entrypoint pages avoid placeholder markers",
            tests: vec![TestCase {
                id: TestId("docs.quality.no_placeholders".to_string()),
                title: "stable docs entrypoint pages do not contain TODO-style placeholders",
                kind: TestKind::Pure,
                run: test_docs_020_no_placeholders,
            }],
        },
        Contract {
            id: ContractId("DOC-021".to_string()),
            title: "docs entrypoint pages avoid raw tabs",
            tests: vec![TestCase {
                id: TestId("docs.format.no_tabs".to_string()),
                title: "docs entrypoint pages do not contain raw tab characters",
                kind: TestKind::Pure,
                run: test_docs_021_no_tabs,
            }],
        },
        Contract {
            id: ContractId("DOC-022".to_string()),
            title: "docs entrypoint pages avoid trailing whitespace",
            tests: vec![TestCase {
                id: TestId("docs.format.no_trailing_whitespace".to_string()),
                title: "docs entrypoint pages do not contain trailing whitespace",
                kind: TestKind::Pure,
                run: test_docs_022_no_trailing_whitespace,
            }],
        },
        Contract {
            id: ContractId("DOC-023".to_string()),
            title: "docs entrypoint pages keep a single top-level heading",
            tests: vec![TestCase {
                id: TestId("docs.structure.single_h1".to_string()),
                title: "docs entrypoint pages contain exactly one H1 heading",
                kind: TestKind::Pure,
                run: test_docs_023_single_h1,
            }],
        },
        Contract {
            id: ContractId("DOC-024".to_string()),
            title: "docs entrypoint pages avoid absolute local file links",
            tests: vec![TestCase {
                id: TestId("docs.links.no_absolute_local_paths".to_string()),
                title: "docs entrypoint pages do not link to absolute local file paths",
                kind: TestKind::Pure,
                run: test_docs_024_no_absolute_local_paths,
            }],
        },
        Contract {
            id: ContractId("DOC-025".to_string()),
            title: "docs entrypoint pages avoid raw http links",
            tests: vec![TestCase {
                id: TestId("docs.links.no_raw_http".to_string()),
                title: "docs entrypoint pages do not use raw http links",
                kind: TestKind::Pure,
                run: test_docs_025_no_raw_http_links,
            }],
        },
        Contract {
            id: ContractId("DOC-026".to_string()),
            title: "docs root entrypoint avoids duplicate section index links",
            tests: vec![TestCase {
                id: TestId("docs.index.unique_section_links".to_string()),
                title: "docs/index.md links each section index at most once",
                kind: TestKind::Pure,
                run: test_docs_026_unique_section_links,
            }],
        },
        Contract {
            id: ContractId("DOC-027".to_string()),
            title: "docs section indexes link at least one local page",
            tests: vec![TestCase {
                id: TestId("docs.index.section_indexes_list_local_pages".to_string()),
                title: "required section INDEX.md pages link at least one local markdown page",
                kind: TestKind::Pure,
                run: test_docs_027_section_indexes_list_local_pages,
            }],
        },
        Contract {
            id: ContractId("DOC-028".to_string()),
            title: "docs section indexes avoid duplicate local page links",
            tests: vec![TestCase {
                id: TestId("docs.index.section_indexes_unique_local_pages".to_string()),
                title: "top-level section INDEX.md pages link each direct local markdown page at most once",
                kind: TestKind::Pure,
                run: test_docs_028_section_indexes_unique_local_pages,
            }],
        },
        Contract {
            id: ContractId("DOC-029".to_string()),
            title: "docs root entrypoint pages avoid duplicate local page links",
            tests: vec![TestCase {
                id: TestId("docs.index.root_entrypoints_unique_local_pages".to_string()),
                title: "docs root entrypoint pages link each local markdown page at most once",
                kind: TestKind::Pure,
                run: test_docs_029_root_entrypoints_unique_local_pages,
            }],
        },
        Contract {
            id: ContractId("DOC-030".to_string()),
            title: "docs index correctness report stays derivable",
            tests: vec![TestCase {
                id: TestId("docs.index.report_generated".to_string()),
                title: "docs contracts can render a deterministic index correctness report artifact",
                kind: TestKind::Pure,
                run: test_docs_030_index_report_generated,
            }],
        },
        Contract {
            id: ContractId("DOC-031".to_string()),
            title: "docs root keeps a single canonical entrypoint",
            tests: vec![TestCase {
                id: TestId("docs.index.single_entrypoint".to_string()),
                title: "docs root keeps docs/index.md and docs/index.md content-identical as one logical entrypoint",
                kind: TestKind::Pure,
                run: test_docs_031_single_entrypoint,
            }],
        },
        Contract {
            id: ContractId("DOC-032".to_string()),
            title: "docs broken links report is generated deterministically",
            tests: vec![TestCase {
                id: TestId("docs.reports.broken_links_generated".to_string()),
                title: "docs contracts write the broken links artifact report",
                kind: TestKind::Pure,
                run: test_docs_032_broken_links_report_generated,
            }],
        },
        Contract {
            id: ContractId("DOC-033".to_string()),
            title: "docs orphan pages report is generated deterministically",
            tests: vec![TestCase {
                id: TestId("docs.reports.orphans_generated".to_string()),
                title: "docs contracts write the orphan pages artifact report",
                kind: TestKind::Pure,
                run: test_docs_033_orphans_report_generated,
            }],
        },
        Contract {
            id: ContractId("DOC-034".to_string()),
            title: "docs metadata coverage report is generated deterministically",
            tests: vec![TestCase {
                id: TestId("docs.reports.metadata_coverage_generated".to_string()),
                title: "docs contracts write the metadata coverage artifact report",
                kind: TestKind::Pure,
                run: test_docs_034_metadata_coverage_report_generated,
            }],
        },
        Contract {
            id: ContractId("DOC-035".to_string()),
            title: "docs duplication report is generated deterministically",
            tests: vec![TestCase {
                id: TestId("docs.reports.duplication_generated".to_string()),
                title: "docs contracts write the duplication artifact report",
                kind: TestKind::Pure,
                run: test_docs_035_duplication_report_generated,
            }],
        },
        Contract {
            id: ContractId("DOC-036".to_string()),
            title: "docs contract coverage report is generated deterministically",
            tests: vec![TestCase {
                id: TestId("docs.reports.coverage_generated".to_string()),
                title: "docs contracts write the coverage artifact report",
                kind: TestKind::Pure,
                run: test_docs_036_coverage_report_generated,
            }],
        },
        Contract {
            id: ContractId("DOC-037".to_string()),
            title: "spine pages require frontmatter",
            tests: vec![TestCase {
                id: TestId("docs.metadata.frontmatter_required".to_string()),
                title: "spine pages declare required YAML frontmatter keys",
                kind: TestKind::Pure,
                run: test_docs_037_spine_frontmatter_required,
            }],
        },
        Contract {
            id: ContractId("DOC-038".to_string()),
            title: "spine page frontmatter values stay normalized",
            tests: vec![TestCase {
                id: TestId("docs.metadata.frontmatter_values".to_string()),
                title: "spine page frontmatter uses normalized audience, type, and stability values",
                kind: TestKind::Pure,
                run: test_docs_038_spine_frontmatter_values,
            }],
        },
        Contract {
            id: ContractId("DOC-039".to_string()),
            title: "stable spine pages keep complete metadata",
            tests: vec![TestCase {
                id: TestId("docs.metadata.stable_spine_requirements".to_string()),
                title: "stable spine pages include owner, last reviewed, tags, and related links",
                kind: TestKind::Pure,
                run: test_docs_039_stable_spine_metadata,
            }],
        },
        Contract {
            id: ContractId("DOC-040".to_string()),
            title: "reference spine pages declare sources",
            tests: vec![TestCase {
                id: TestId("docs.metadata.reference_sources".to_string()),
                title: "reference spine pages include source links in frontmatter",
                kind: TestKind::Pure,
                run: test_docs_040_reference_spine_sources,
            }],
        },
        Contract {
            id: ContractId("DOC-041".to_string()),
            title: "internal docs keep frontmatter boundary",
            tests: vec![TestCase {
                id: TestId("docs.metadata.internal_boundary".to_string()),
                title: "internal docs declare internal frontmatter and avoid user audience",
                kind: TestKind::Pure,
                run: test_docs_041_internal_frontmatter_boundary,
            }],
        },
        Contract {
            id: ContractId("DOC-042".to_string()),
            title: "stable spine pages keep review dates normalized",
            tests: vec![TestCase {
                id: TestId("docs.metadata.review_freshness".to_string()),
                title: "stable spine pages use ISO review dates",
                kind: TestKind::Pure,
                run: test_docs_042_stable_review_freshness,
            }],
        },
        Contract {
            id: ContractId("DOC-043".to_string()),
            title: "how-to spine pages declare verification",
            tests: vec![TestCase {
                id: TestId("docs.metadata.how_to_verification".to_string()),
                title: "how-to spine pages set verification in frontmatter",
                kind: TestKind::Pure,
                run: test_docs_043_how_to_verification_flag,
            }],
        },
        Contract {
            id: ContractId("DOC-044".to_string()),
            title: "docs frontmatter schema stays present",
            tests: vec![TestCase {
                id: TestId("docs.metadata.frontmatter_schema".to_string()),
                title: "frontmatter schema exists and requires the shared keys",
                kind: TestKind::Pure,
                run: test_docs_044_frontmatter_schema_exists,
            }],
        },
        Contract {
            id: ContractId("DOC-045".to_string()),
            title: "reader utility pages keep shared metadata",
            tests: vec![TestCase {
                id: TestId("docs.metadata.reader_utility_frontmatter".to_string()),
                title: "site map and reading-track pages keep the required shared frontmatter",
                kind: TestKind::Pure,
                run: test_docs_045_reader_utility_frontmatter,
            }],
        },
        Contract {
            id: ContractId("DOC-046".to_string()),
            title: "reader utility pages do not link into internal docs",
            tests: vec![TestCase {
                id: TestId("docs.links.reader_utility_no_internal_links".to_string()),
                title: "site map and reading-track pages stay on the published reader surface",
                kind: TestKind::Pure,
                run: test_docs_046_reader_utility_no_internal_links,
            }],
        },
        Contract {
            id: ContractId("DOC-047".to_string()),
            title: "reader spine pages do not link into internal docs",
            tests: vec![TestCase {
                id: TestId("docs.links.reader_spine_no_internal_links".to_string()),
                title: "reader spine pages stay on the published surface",
                kind: TestKind::Pure,
                run: test_docs_047_reader_spine_no_internal_links,
            }],
        },
    ])
}
