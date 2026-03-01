const ROOT_ALLOWED_VISIBLE: [&str; 19] = [
    ".cargo",
    ".dockerignore",
    ".editorconfig",
    ".github",
    ".gitignore",
    "CHANGELOG.md",
    "CONTRACT.md",
    "CONTRIBUTING.md",
    "Cargo.lock",
    "Cargo.toml",
    "Dockerfile",
    "LICENSE",
    "Makefile",
    "README.md",
    "SECURITY.md",
    "configs",
    "crates",
    "docker",
    "root-surface.json",
];

const ROOT_ALLOWED_VISIBLE_TAIL: [&str; 5] = ["docs", "make", "mkdocs.yml", "ops", "rust-toolchain.toml"];

const ROOT_IGNORED_LOCAL: [&str; 3] = [".git", ".idea", "artifacts"];

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("ROOT-001".to_string()),
            title: "repo root matches the sealed surface",
            tests: vec![TestCase {
                id: TestId("root.surface.allowlist".to_string()),
                title: "root files and directories stay within the declared allowlist",
                kind: TestKind::Pure,
                run: test_root_001_surface_allowlist,
            }],
        },
        Contract {
            id: ContractId("ROOT-002".to_string()),
            title: "repo root markdown stays within the documented surface",
            tests: vec![TestCase {
                id: TestId("root.docs.allowed_markdown".to_string()),
                title: "root markdown files stay within the allowlist",
                kind: TestKind::Pure,
                run: test_root_002_allowed_markdown,
            }],
        },
        Contract {
            id: ContractId("ROOT-003".to_string()),
            title: "repo root forbids legacy script directories",
            tests: vec![TestCase {
                id: TestId("root.surface.no_legacy_script_dirs".to_string()),
                title: "legacy script directories stay absent",
                kind: TestKind::Pure,
                run: test_root_003_no_legacy_script_dirs,
            }],
        },
        Contract {
            id: ContractId("ROOT-004".to_string()),
            title: "repo root symlinks stay explicitly allowlisted",
            tests: vec![TestCase {
                id: TestId("root.surface.symlink_allowlist".to_string()),
                title: "root symlinks stay within the allowlist",
                kind: TestKind::Pure,
                run: test_root_004_symlink_allowlist,
            }],
        },
        Contract {
            id: ContractId("ROOT-005".to_string()),
            title: "root Dockerfile points at the canonical runtime dockerfile",
            tests: vec![TestCase {
                id: TestId("root.dockerfile.policy".to_string()),
                title: "root Dockerfile matches the canonical location policy",
                kind: TestKind::Pure,
                run: test_root_005_dockerfile_policy,
            }],
        },
        Contract {
            id: ContractId("ROOT-006".to_string()),
            title: "root Makefile stays a thin delegator",
            tests: vec![TestCase {
                id: TestId("root.makefile.thin_delegator".to_string()),
                title: "root Makefile contains only the canonical include",
                kind: TestKind::Pure,
                run: test_root_006_makefile_thin_delegator,
            }],
        },
        Contract {
            id: ContractId("ROOT-007".to_string()),
            title: "workspace lockfile stays rooted at the repo root",
            tests: vec![TestCase {
                id: TestId("root.cargo.workspace_lock".to_string()),
                title: "workspace uses a single root Cargo.lock",
                kind: TestKind::Pure,
                run: test_root_007_workspace_lock,
            }],
        },
        Contract {
            id: ContractId("ROOT-008".to_string()),
            title: "rust toolchain stays pinned at the repo root",
            tests: vec![TestCase {
                id: TestId("root.rust_toolchain.pinned".to_string()),
                title: "rust-toolchain.toml exists and pins a concrete channel",
                kind: TestKind::Pure,
                run: test_root_008_rust_toolchain_pinned,
            }],
        },
        Contract {
            id: ContractId("ROOT-009".to_string()),
            title: "cargo config avoids implicit network defaults",
            tests: vec![TestCase {
                id: TestId("root.cargo_config.no_network_defaults".to_string()),
                title: ".cargo/config.toml avoids network-fetch defaults",
                kind: TestKind::Pure,
                run: test_root_009_cargo_config_no_network_defaults,
            }],
        },
        Contract {
            id: ContractId("ROOT-010".to_string()),
            title: "license stays on the approved SPDX track",
            tests: vec![TestCase {
                id: TestId("root.license.approved".to_string()),
                title: "LICENSE matches the approved license family",
                kind: TestKind::Pure,
                run: test_root_010_license_approved,
            }],
        },
        Contract {
            id: ContractId("ROOT-011".to_string()),
            title: "security policy keeps a reporting path",
            tests: vec![TestCase {
                id: TestId("root.security.report_path".to_string()),
                title: "SECURITY.md includes reporting and disclosure guidance",
                kind: TestKind::Pure,
                run: test_root_011_security_report_path,
            }],
        },
        Contract {
            id: ContractId("ROOT-012".to_string()),
            title: "contributing guide names the canonical control plane",
            tests: vec![TestCase {
                id: TestId("root.contributing.control_plane".to_string()),
                title: "CONTRIBUTING.md points contributors to bijux dev atlas",
                kind: TestKind::Pure,
                run: test_root_012_contributing_control_plane,
            }],
        },
        Contract {
            id: ContractId("ROOT-013".to_string()),
            title: "changelog keeps a versioned release header",
            tests: vec![TestCase {
                id: TestId("root.changelog.version_header".to_string()),
                title: "CHANGELOG.md starts with a version header",
                kind: TestKind::Pure,
                run: test_root_013_changelog_version_header,
            }],
        },
        Contract {
            id: ContractId("ROOT-014".to_string()),
            title: "gitignore preserves tracked contract outputs",
            tests: vec![TestCase {
                id: TestId("root.gitignore.tracked_contract_outputs".to_string()),
                title: ".gitignore does not hide tracked contract outputs",
                kind: TestKind::Pure,
                run: test_root_014_gitignore_tracked_contract_outputs,
            }],
        },
        Contract {
            id: ContractId("ROOT-015".to_string()),
            title: "repo root forbids duplicate toolchain authority files",
            tests: vec![TestCase {
                id: TestId("root.surface.no_duplicate_toolchain_authority".to_string()),
                title: "root toolchain authority stays singular and canonical",
                kind: TestKind::Pure,
                run: test_root_015_no_duplicate_toolchain_authority,
            }],
        },
        Contract {
            id: ContractId("ROOT-016".to_string()),
            title: "repo root keeps a machine-readable surface manifest",
            tests: vec![TestCase {
                id: TestId("root.surface.manifest_complete".to_string()),
                title: "root surface manifest matches the sealed root surface",
                kind: TestKind::Pure,
                run: test_root_016_surface_manifest_complete,
            }],
        },
        Contract {
            id: ContractId("ROOT-017".to_string()),
            title: "repo root forbids undeclared binary-like artifacts",
            tests: vec![TestCase {
                id: TestId("root.surface.no_binary_artifacts".to_string()),
                title: "root files may not use binary artifact extensions",
                kind: TestKind::Pure,
                run: test_root_017_no_binary_artifacts,
            }],
        },
        Contract {
            id: ContractId("ROOT-018".to_string()),
            title: "repo root forbids committed environment files",
            tests: vec![TestCase {
                id: TestId("root.surface.no_env_files".to_string()),
                title: "root .env files stay absent",
                kind: TestKind::Pure,
                run: test_root_018_no_env_files,
            }],
        },
        Contract {
            id: ContractId("ROOT-019".to_string()),
            title: "repo root keeps a bounded top-level directory surface",
            tests: vec![TestCase {
                id: TestId("root.surface.directory_budget".to_string()),
                title: "root directory count stays within the approved budget",
                kind: TestKind::Pure,
                run: test_root_019_directory_budget,
            }],
        },
        Contract {
            id: ContractId("ROOT-020".to_string()),
            title: "repo root manifest keeps single-segment entry paths",
            tests: vec![TestCase {
                id: TestId("root.surface.single_segment_entries".to_string()),
                title: "root surface manifest entries stay single-segment",
                kind: TestKind::Pure,
                run: test_root_020_single_segment_entries,
            }],
        },
        Contract {
            id: ContractId("ROOT-027".to_string()),
            title: "root surface manifest declares the configs and ops ssot roots",
            tests: vec![TestCase {
                id: TestId("root.surface.ssot_roots".to_string()),
                title: "root surface manifest names configs and ops as ssot roots",
                kind: TestKind::Pure,
                run: test_root_027_manifest_ssot_roots,
            }],
        },
        Contract {
            id: ContractId("ROOT-028".to_string()),
            title: "root surface manifest keeps docs under contract governance",
            tests: vec![TestCase {
                id: TestId("root.surface.docs_governed".to_string()),
                title: "root surface manifest names docs as a governed root",
                kind: TestKind::Pure,
                run: test_root_028_manifest_docs_governed,
            }],
        },
        Contract {
            id: ContractId("ROOT-029".to_string()),
            title: "repo tree forbids nested git repositories",
            tests: vec![TestCase {
                id: TestId("root.surface.no_nested_git".to_string()),
                title: "no nested .git directories exist under the repo tree",
                kind: TestKind::Pure,
                run: test_root_029_no_nested_git,
            }],
        },
        Contract {
            id: ContractId("ROOT-030".to_string()),
            title: "repo root forbids vendor directories and blobs",
            tests: vec![TestCase {
                id: TestId("root.surface.no_vendor_blobs".to_string()),
                title: "vendor directories do not appear at the repo root",
                kind: TestKind::Pure,
                run: test_root_030_no_vendor_blobs,
            }],
        },
        Contract {
            id: ContractId("ROOT-031".to_string()),
            title: "repo root forbids oversized root files",
            tests: vec![TestCase {
                id: TestId("root.surface.root_file_size_budget".to_string()),
                title: "root files stay under the approved size budget",
                kind: TestKind::Pure,
                run: test_root_031_root_file_size_budget,
            }],
        },
        Contract {
            id: ContractId("ROOT-032".to_string()),
            title: "configs and ops do not duplicate rust toolchain pins",
            tests: vec![TestCase {
                id: TestId("root.surface.no_nested_toolchain_pins".to_string()),
                title: "toolchain authority does not reappear under configs or ops",
                kind: TestKind::Pure,
                run: test_root_032_no_nested_toolchain_pins,
            }],
        },
        Contract {
            id: ContractId("ROOT-033".to_string()),
            title: "release process authority stays inside docs or ops",
            tests: vec![TestCase {
                id: TestId("root.docs.release_process_location".to_string()),
                title: "release process files do not reappear at the repo root",
                kind: TestKind::Pure,
                run: test_root_033_release_process_location,
            }],
        },
        Contract {
            id: ContractId("ROOT-034".to_string()),
            title: "repo root keeps a single contracts command interface",
            tests: vec![TestCase {
                id: TestId("root.contracts.single_interface".to_string()),
                title: "root documentation references the canonical contracts entrypoint",
                kind: TestKind::Pure,
                run: test_root_034_single_contract_interface,
            }],
        },
        Contract {
            id: ContractId("ROOT-035".to_string()),
            title: "make contract wrappers delegate to the contracts runner",
            tests: vec![TestCase {
                id: TestId("root.make.contract_wrappers_delegate".to_string()),
                title: "make/checks.mk delegates to bijux dev atlas contracts make",
                kind: TestKind::Pure,
                run: test_root_035_make_contract_wrappers_delegate,
            }],
        },
        Contract {
            id: ContractId("ROOT-036".to_string()),
            title: "docker make wrappers delegate to the contracts runner",
            tests: vec![TestCase {
                id: TestId("root.make.docker_wrappers_delegate".to_string()),
                title: "make/docker.mk delegates to bijux dev atlas contracts docker",
                kind: TestKind::Pure,
                run: test_root_036_docker_wrappers_delegate,
            }],
        },
        Contract {
            id: ContractId("ROOT-037".to_string()),
            title: "repo tree forbids editor backups and platform noise",
            tests: vec![TestCase {
                id: TestId("root.surface.no_editor_backup_noise".to_string()),
                title: "no .orig, backup, or .DS_Store files exist in the repo tree",
                kind: TestKind::Pure,
                run: test_root_037_no_editor_backup_noise,
            }],
        },
        Contract {
            id: ContractId("ROOT-038".to_string()),
            title: "gitattributes line ending policy stays consistent when present",
            tests: vec![TestCase {
                id: TestId("root.gitattributes.line_endings".to_string()),
                title: ".gitattributes keeps a canonical line ending policy if present",
                kind: TestKind::Pure,
                run: test_root_038_gitattributes_line_endings,
            }],
        },
        Contract {
            id: ContractId("ROOT-039".to_string()),
            title: "workspace members match the actual crate surface",
            tests: vec![TestCase {
                id: TestId("root.cargo.workspace_members_match".to_string()),
                title: "workspace member declarations match crate directories and manifests",
                kind: TestKind::Pure,
                run: test_root_039_workspace_members_match,
            }],
        },
        Contract {
            id: ContractId("ROOT-040".to_string()),
            title: "workspace crates keep canonical naming",
            tests: vec![TestCase {
                id: TestId("root.cargo.crate_naming".to_string()),
                title: "workspace crate directory names and package names stay canonical",
                kind: TestKind::Pure,
                run: test_root_040_crate_naming,
            }],
        },
        Contract {
            id: ContractId("ROOT-021".to_string()),
            title: "editorconfig exists for shared formatting contracts",
            tests: vec![TestCase {
                id: TestId("root.editorconfig.exists".to_string()),
                title: ".editorconfig stays present at the repo root",
                kind: TestKind::Pure,
                run: test_root_021_editorconfig_exists,
            }],
        },
        Contract {
            id: ContractId("ROOT-022".to_string()),
            title: "repo root keeps a single unambiguous license authority",
            tests: vec![TestCase {
                id: TestId("root.license.single_authority".to_string()),
                title: "license metadata does not declare conflicting license families",
                kind: TestKind::Pure,
                run: test_root_022_license_single_authority,
            }],
        },
        Contract {
            id: ContractId("ROOT-023".to_string()),
            title: "root readme keeps the canonical entrypoint sections",
            tests: vec![TestCase {
                id: TestId("root.readme.entrypoint_sections".to_string()),
                title: "README.md keeps the required top-level entrypoint sections",
                kind: TestKind::Pure,
                run: test_root_023_readme_entrypoint_sections,
            }],
        },
        Contract {
            id: ContractId("ROOT-024".to_string()),
            title: "root docs avoid legacy control-plane references",
            tests: vec![TestCase {
                id: TestId("root.docs.no_legacy_links".to_string()),
                title: "root docs do not reference deleted legacy control-plane surfaces",
                kind: TestKind::Pure,
                run: test_root_024_docs_no_legacy_links,
            }],
        },
        Contract {
            id: ContractId("ROOT-025".to_string()),
            title: "repo root keeps support routing out of the root surface",
            tests: vec![TestCase {
                id: TestId("root.docs.support_routing".to_string()),
                title: "support references point into docs or ops instead of new root authority files",
                kind: TestKind::Pure,
                run: test_root_025_support_routing,
            }],
        },
        Contract {
            id: ContractId("ROOT-026".to_string()),
            title: "repo root forbids duplicate policy directories",
            tests: vec![TestCase {
                id: TestId("root.surface.no_duplicate_policy_dirs".to_string()),
                title: "policy directories stay out of the repo root surface",
                kind: TestKind::Pure,
                run: test_root_026_no_duplicate_policy_dirs,
            }],
        },
        Contract {
            id: ContractId("ROOT-041".to_string()),
            title: "top-level contract documents follow the canonical executable template",
            tests: vec![TestCase {
                id: TestId("root.contract_docs.canonical_template".to_string()),
                title: "root/docs/docker/make/ops/configs CONTRACT.md files stay canonical and registry-complete",
                kind: TestKind::Pure,
                run: test_root_041_contract_docs_canonical_template,
            }],
        },
        Contract {
            id: ContractId("ROOT-042".to_string()),
            title: "contract registries keep unique contract ids and mapped checks",
            tests: vec![TestCase {
                id: TestId("root.contracts.meta_registry_integrity".to_string()),
                title: "all contract ids are unique and every registry row maps at least one check",
                kind: TestKind::Pure,
                run: test_root_042_meta_registry_integrity,
            }],
        },
        Contract {
            id: ContractId("ROOT-043".to_string()),
            title: "contract registries keep check-to-contract mappings non-orphaned",
            tests: vec![TestCase {
                id: TestId("root.contracts.meta_test_mapping_integrity".to_string()),
                title: "all test ids map to exactly one contract across registries",
                kind: TestKind::Pure,
                run: test_root_043_meta_test_mapping_integrity,
            }],
        },
        Contract {
            id: ContractId("ROOT-044".to_string()),
            title: "contracts list and group execution order stay deterministic",
            tests: vec![TestCase {
                id: TestId("root.contracts.meta_ordering_stable".to_string()),
                title: "contracts list ordering and all-domain execution order stay stable",
                kind: TestKind::Pure,
                run: test_root_044_meta_ordering_stable,
            }],
        },
        Contract {
            id: ContractId("META-REQ-001".to_string()),
            title: "required contracts stay stable and approved",
            tests: vec![TestCase {
                id: TestId("root.required_contracts.stable_and_approved".to_string()),
                title: "required contracts manifest and artifact stay aligned unless active approval metadata exists",
                kind: TestKind::Pure,
                run: test_meta_req_001_required_contracts_stable_and_approved,
            }],
        },
        Contract {
            id: ContractId("META-REQ-002".to_string()),
            title: "required contracts cover every pillar",
            tests: vec![TestCase {
                id: TestId("root.required_contracts.cover_every_pillar".to_string()),
                title: "required contracts include root docs make configs docker and ops coverage",
                kind: TestKind::Pure,
                run: test_meta_req_002_required_contracts_cover_every_pillar,
            }],
        },
        Contract {
            id: ContractId("META-REQ-003".to_string()),
            title: "required contracts avoid placeholder stubs",
            tests: vec![TestCase {
                id: TestId("root.required_contracts.no_placeholder_stubs".to_string()),
                title: "required contracts point to concrete non-placeholder rules and tests",
                kind: TestKind::Pure,
                run: test_meta_req_003_required_contracts_no_placeholder_stubs,
            }],
        },
    ])
}
