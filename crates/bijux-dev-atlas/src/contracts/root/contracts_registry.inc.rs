const ROOT_ALLOWED_VISIBLE: [&str; 16] = [
    ".cargo",
    ".dockerignore",
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
    Ok(vec![Contract {
        id: ContractId("ROOT-001".to_string()),
        title: "repo root matches the sealed surface",
        tests: vec![TestCase {
            id: TestId("root.surface.allowlist".to_string()),
            title: "root files and directories stay within the declared allowlist",
            kind: TestKind::Pure,
            run: test_root_001_surface_allowlist,
        }],
    }])
}
