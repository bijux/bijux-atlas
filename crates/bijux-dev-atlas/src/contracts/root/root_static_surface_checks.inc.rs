use std::path::PathBuf;

const ROOT_FORBIDDEN_BINARY_EXTENSIONS: [&str; 12] = [
    "bin", "dmg", "exe", "gz", "iso", "jar", "o", "so", "tar", "tgz", "war", "zip",
];
const ROOT_DIRECTORY_BUDGET: usize = 8;
const ROOT_FILE_SIZE_BUDGET_BYTES: u64 = 512 * 1024;

fn test_root_001_surface_allowlist(ctx: &RunContext) -> TestResult {
    let mut allowed = ROOT_ALLOWED_VISIBLE
        .iter()
        .chain(ROOT_ALLOWED_VISIBLE_TAIL.iter())
        .map(|value| (*value).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-001".to_string(),
                test_id: "root.surface.allowlist".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if ROOT_IGNORED_LOCAL.iter().any(|ignored| *ignored == name) {
            continue;
        }
        if !allowed.remove(&name) {
            violations.push(Violation {
                contract_id: "ROOT-001".to_string(),
                test_id: "root.surface.allowlist".to_string(),
                file: Some(name),
                line: None,
                message: "unexpected repo root entry".to_string(),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn read_root_text(ctx: &RunContext, relative: &str, contract_id: &str, test_id: &str) -> Result<String, TestResult> {
    std::fs::read_to_string(ctx.repo_root.join(relative)).map_err(|err| {
        TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: format!("read failed: {err}"),
            evidence: None,
        }])
    })
}

fn push_root_violation(violations: &mut Vec<Violation>, contract_id: &str, test_id: &str, file: impl Into<Option<String>>, message: impl Into<String>) {
    violations.push(Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: file.into(),
        line: None,
        message: message.into(),
        evidence: None,
    });
}

fn test_root_002_allowed_markdown(ctx: &RunContext) -> TestResult {
    let allowed = ["README.md", "CONTRIBUTING.md", "SECURITY.md", "CHANGELOG.md"];
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-002".to_string(),
                test_id: "root.docs.allowed_markdown".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }
        if !allowed.iter().any(|candidate| *candidate == name) {
            push_root_violation(
                &mut violations,
                "ROOT-002",
                "root.docs.allowed_markdown",
                Some(name),
                "unexpected root markdown file",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_003_no_legacy_script_dirs(ctx: &RunContext) -> TestResult {
    let forbidden = ["scripts", "tools", "xtask"];
    let mut violations = Vec::new();
    for name in forbidden {
        if ctx.repo_root.join(name).exists() {
            push_root_violation(
                &mut violations,
                "ROOT-003",
                "root.surface.no_legacy_script_dirs",
                Some(name.to_string()),
                "legacy root script directory is forbidden",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_004_symlink_allowlist(ctx: &RunContext) -> TestResult {
    let allowed = ["Dockerfile"];
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-004".to_string(),
                test_id: "root.surface.symlink_allowlist".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_symlink() && !allowed.iter().any(|candidate| *candidate == name) {
            push_root_violation(
                &mut violations,
                "ROOT-004",
                "root.surface.symlink_allowlist",
                Some(name),
                "root symlink is not allowlisted",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_005_dockerfile_policy(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("Dockerfile");
    let mut violations = Vec::new();
    if !path.exists() {
        push_root_violation(
            &mut violations,
            "ROOT-005",
            "root.dockerfile.policy",
            Some("Dockerfile".to_string()),
            "root Dockerfile is required",
        );
    } else if !path.is_symlink() {
        push_root_violation(
            &mut violations,
            "ROOT-005",
            "root.dockerfile.policy",
            Some("Dockerfile".to_string()),
            "root Dockerfile must be a symlink to the canonical runtime dockerfile",
        );
    } else {
        match std::fs::read_link(&path) {
            Ok(target) => {
                let expected = PathBuf::from("docker/images/runtime/Dockerfile");
                if target != expected {
                    push_root_violation(
                        &mut violations,
                        "ROOT-005",
                        "root.dockerfile.policy",
                        Some("Dockerfile".to_string()),
                        format!("root Dockerfile must point to {}", expected.display()),
                    );
                }
            }
            Err(err) => push_root_violation(
                &mut violations,
                "ROOT-005",
                "root.dockerfile.policy",
                Some("Dockerfile".to_string()),
                format!("read symlink failed: {err}"),
            ),
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_006_makefile_thin_delegator(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(ctx, "Makefile", "ROOT-006", "root.makefile.thin_delegator") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let trimmed: Vec<&str> = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if trimmed.len() == 1 && trimmed[0] == "include make/public.mk" {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-006".to_string(),
            test_id: "root.makefile.thin_delegator".to_string(),
            file: Some("Makefile".to_string()),
            line: None,
            message: "root Makefile must contain only `include make/public.mk`".to_string(),
            evidence: None,
        }])
    }
}

fn test_root_007_workspace_lock(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    if !ctx.repo_root.join("Cargo.lock").is_file() {
        push_root_violation(
            &mut violations,
            "ROOT-007",
            "root.cargo.workspace_lock",
            Some("Cargo.lock".to_string()),
            "root Cargo.lock is required",
        );
    }
    let crates_dir = ctx.repo_root.join("crates");
    if crates_dir.is_dir() {
        let mut nested = Vec::new();
        collect_named_files(&crates_dir, "Cargo.lock", &mut nested);
        for path in nested {
            let rel = path
                .strip_prefix(&ctx.repo_root)
                .map(|value| value.display().to_string())
                .unwrap_or_else(|_| path.display().to_string());
            push_root_violation(
                &mut violations,
                "ROOT-007",
                "root.cargo.workspace_lock",
                Some(rel),
                "nested Cargo.lock is forbidden inside crates/",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn collect_named_files(dir: &std::path::Path, filename: &str, found: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_named_files(&path, filename, found);
        } else if path.file_name().and_then(|value| value.to_str()) == Some(filename) {
            found.push(path);
        }
    }
}

fn test_root_008_rust_toolchain_pinned(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(ctx, "rust-toolchain.toml", "ROOT-008", "root.rust_toolchain.pinned") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    let pinned = contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("channel = "))
        .map(|value| value.trim().trim_matches('"').to_string());
    match pinned {
        Some(channel)
            if channel.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
                && channel.split('.').count() == 3 => {}
        Some(_) => push_root_violation(
            &mut violations,
            "ROOT-008",
            "root.rust_toolchain.pinned",
            Some("rust-toolchain.toml".to_string()),
            "rust-toolchain channel must be a concrete semantic version",
        ),
        None => push_root_violation(
            &mut violations,
            "ROOT-008",
            "root.rust_toolchain.pinned",
            Some("rust-toolchain.toml".to_string()),
            "rust-toolchain channel is missing",
        ),
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_009_cargo_config_no_network_defaults(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(
        ctx,
        ".cargo/config.toml",
        "ROOT-009",
        "root.cargo_config.no_network_defaults",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let forbidden_snippets = [
        "git-fetch-with-cli = true",
        "git-fetch-with-cli=true",
        "registries.crates-io.protocol",
        "net.",
    ];
    let mut violations = Vec::new();
    for snippet in forbidden_snippets {
        if contents.contains(snippet) {
            push_root_violation(
                &mut violations,
                "ROOT-009",
                "root.cargo_config.no_network_defaults",
                Some(".cargo/config.toml".to_string()),
                format!("forbidden cargo network setting: {snippet}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_010_license_approved(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(ctx, "LICENSE", "ROOT-010", "root.license.approved") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    if contents.contains("Apache License") && contents.contains("Version 2.0, January 2004") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-010".to_string(),
            test_id: "root.license.approved".to_string(),
            file: Some("LICENSE".to_string()),
            line: None,
            message: "LICENSE must match the approved Apache-2.0 text".to_string(),
            evidence: None,
        }])
    }
}

fn test_root_011_security_report_path(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(ctx, "SECURITY.md", "ROOT-011", "root.security.report_path") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let required = ["Reporting a Vulnerability", "private security advisory", "Triage and Fix"];
    let mut missing = Vec::new();
    for needle in required {
        if !contents.to_lowercase().contains(&needle.to_lowercase()) {
            missing.push(needle);
        }
    }
    if missing.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-011".to_string(),
            test_id: "root.security.report_path".to_string(),
            file: Some("SECURITY.md".to_string()),
            line: None,
            message: format!("SECURITY.md is missing required guidance: {}", missing.join(", ")),
            evidence: None,
        }])
    }
}

fn test_root_012_contributing_control_plane(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(
        ctx,
        "CONTRIBUTING.md",
        "ROOT-012",
        "root.contributing.control_plane",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    if contents.contains("bijux dev atlas") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-012".to_string(),
            test_id: "root.contributing.control_plane".to_string(),
            file: Some("CONTRIBUTING.md".to_string()),
            line: None,
            message: "CONTRIBUTING.md must name `bijux dev atlas` as the canonical control plane".to_string(),
            evidence: None,
        }])
    }
}

fn test_root_013_changelog_version_header(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(
        ctx,
        "CHANGELOG.md",
        "ROOT-013",
        "root.changelog.version_header",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let has_version_header = contents
        .lines()
        .any(|line| line.trim().starts_with("## v"));
    if has_version_header {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-013".to_string(),
            test_id: "root.changelog.version_header".to_string(),
            file: Some("CHANGELOG.md".to_string()),
            line: None,
            message: "CHANGELOG.md must contain a `## v...` release header".to_string(),
            evidence: None,
        }])
    }
}

fn test_root_014_gitignore_tracked_contract_outputs(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(
        ctx,
        ".gitignore",
        "ROOT-014",
        "root.gitignore.tracked_contract_outputs",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let forbidden = ["/artifacts/contracts/", "artifacts/contracts/"];
    let mut violations = Vec::new();
    for pattern in forbidden {
        if contents.lines().any(|line| line.trim() == pattern) {
            push_root_violation(
                &mut violations,
                "ROOT-014",
                "root.gitignore.tracked_contract_outputs",
                Some(".gitignore".to_string()),
                format!("tracked contracts output pattern is forbidden: {pattern}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_015_no_duplicate_toolchain_authority(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for duplicate in ["rust-toolchain", "rust-toolchain.yaml", "rust-toolchain.json"] {
        if ctx.repo_root.join(duplicate).exists() {
            push_root_violation(
                &mut violations,
                "ROOT-015",
                "root.surface.no_duplicate_toolchain_authority",
                Some(duplicate.to_string()),
                "only rust-toolchain.toml may define root toolchain authority",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn root_surface_manifest(ctx: &RunContext, contract_id: &str, test_id: &str) -> Result<serde_json::Value, TestResult> {
    let contents = match read_root_text(ctx, "root-surface.json", contract_id, test_id) {
        Ok(contents) => contents,
        Err(result) => return Err(result),
    };
    serde_json::from_str(&contents).map_err(|err| {
        TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("root-surface.json".to_string()),
            line: None,
            message: format!("invalid json: {err}"),
            evidence: None,
        }])
    })
}

