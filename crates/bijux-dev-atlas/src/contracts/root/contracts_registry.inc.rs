const ROOT_ALLOWED_VISIBLE: [&str; 17] = [
    ".cargo",
    ".dockerignore",
    ".editorconfig",
    ".github",
    ".gitignore",
    "CHANGELOG.md",
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
            id: ContractId("ROOT-021".to_string()),
            title: "editorconfig exists for shared formatting contracts",
            tests: vec![TestCase {
                id: TestId("root.editorconfig.exists".to_string()),
                title: ".editorconfig stays present at the repo root",
                kind: TestKind::Pure,
                run: test_root_021_editorconfig_exists,
            }],
        },
    ])
}
