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

fn test_root_016_surface_manifest_complete(ctx: &RunContext) -> TestResult {
    let payload = match root_surface_manifest(ctx, "ROOT-016", "root.surface.manifest_complete") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let entries = match payload["entries"].as_object() {
        Some(entries) => entries,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-016".to_string(),
                test_id: "root.surface.manifest_complete".to_string(),
                file: Some("root-surface.json".to_string()),
                line: None,
                message: "`entries` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let expected = ROOT_ALLOWED_VISIBLE
        .iter()
        .chain(ROOT_ALLOWED_VISIBLE_TAIL.iter())
        .map(|value| (*value).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let manifest_entries = entries.keys().cloned().collect::<std::collections::BTreeSet<_>>();
    let mut violations = Vec::new();
    for name in expected.difference(&manifest_entries) {
        push_root_violation(
            &mut violations,
            "ROOT-016",
            "root.surface.manifest_complete",
            Some(name.clone()),
            "sealed root entry is missing from root-surface.json",
        );
    }
    for name in manifest_entries.difference(&expected) {
        push_root_violation(
            &mut violations,
            "ROOT-016",
            "root.surface.manifest_complete",
            Some(name.clone()),
            "root-surface.json references an undeclared root entry",
        );
    }
    for name in manifest_entries {
        let path = ctx.repo_root.join(&name);
        if !path.exists() {
            push_root_violation(
                &mut violations,
                "ROOT-016",
                "root.surface.manifest_complete",
                Some(name),
                "root-surface.json references a missing repo root entry",
            );
            continue;
        }
        let declared_kind = entries
            .get(&name)
            .and_then(|value| value.get("kind"))
            .and_then(|value| value.as_str());
        let actual_kind = if path.is_symlink() {
            Some("symlink")
        } else if path.is_dir() {
            Some("dir")
        } else if path.is_file() {
            Some("file")
        } else {
            None
        };
        if declared_kind != actual_kind {
            push_root_violation(
                &mut violations,
                "ROOT-016",
                "root.surface.manifest_complete",
                Some(name),
                format!(
                    "root-surface.json kind drift: declared {:?}, actual {:?}",
                    declared_kind, actual_kind
                ),
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

fn test_root_017_no_binary_artifacts(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-017".to_string(),
                test_id: "root.surface.no_binary_artifacts".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let extension = match path.extension().and_then(|value| value.to_str()) {
            Some(extension) => extension,
            None => continue,
        };
        if ROOT_FORBIDDEN_BINARY_EXTENSIONS
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        {
            push_root_violation(
                &mut violations,
                "ROOT-017",
                "root.surface.no_binary_artifacts",
                Some(name),
                format!("root binary artifact extension is forbidden: .{extension}"),
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

fn test_root_018_no_env_files(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-018".to_string(),
                test_id: "root.surface.no_env_files".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".env" || (name.starts_with(".env.") && name.len() > 5) {
            push_root_violation(
                &mut violations,
                "ROOT-018",
                "root.surface.no_env_files",
                Some(name),
                "committed root .env files are forbidden",
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

fn test_root_019_directory_budget(ctx: &RunContext) -> TestResult {
    let expected = ROOT_ALLOWED_VISIBLE
        .iter()
        .chain(ROOT_ALLOWED_VISIBLE_TAIL.iter())
        .filter(|name| ctx.repo_root.join(name).is_dir())
        .map(|name| (*name).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut visible_directories = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-019".to_string(),
                test_id: "root.surface.directory_budget".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if ROOT_IGNORED_LOCAL.iter().any(|ignored| *ignored == name) {
            continue;
        }
        visible_directories.push(name);
    }
    visible_directories.sort();
    let mut violations = Vec::new();
    if visible_directories.len() > ROOT_DIRECTORY_BUDGET {
        push_root_violation(
            &mut violations,
            "ROOT-019",
            "root.surface.directory_budget",
            None,
            format!(
                "repo root directory budget exceeded: {} > {} ({})",
                visible_directories.len(),
                ROOT_DIRECTORY_BUDGET,
                visible_directories.join(", ")
            ),
        );
    }
    let actual = visible_directories.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    for name in actual.difference(&expected) {
        push_root_violation(
            &mut violations,
            "ROOT-019",
            "root.surface.directory_budget",
            Some(name.clone()),
            "unexpected top-level directory is outside the approved root surface",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_020_single_segment_entries(ctx: &RunContext) -> TestResult {
    let payload = match root_surface_manifest(ctx, "ROOT-020", "root.surface.single_segment_entries") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let entries = match payload["entries"].as_object() {
        Some(entries) => entries,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-020".to_string(),
                test_id: "root.surface.single_segment_entries".to_string(),
                file: Some("root-surface.json".to_string()),
                line: None,
                message: "`entries` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for name in entries.keys() {
        if name.contains('/') || name.contains('\\') {
            push_root_violation(
                &mut violations,
                "ROOT-020",
                "root.surface.single_segment_entries",
                Some(name.clone()),
                "root manifest entries must be single-segment repo root names",
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

fn test_root_027_manifest_ssot_roots(ctx: &RunContext) -> TestResult {
    let payload = match root_surface_manifest(ctx, "ROOT-027", "root.surface.ssot_roots") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let ssot_roots = match payload["ssot_roots"].as_array() {
        Some(values) => values
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-027".to_string(),
                test_id: "root.surface.ssot_roots".to_string(),
                file: Some("root-surface.json".to_string()),
                line: None,
                message: "`ssot_roots` array is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for required in ["configs", "ops"] {
        if !ssot_roots.contains(required) {
            push_root_violation(
                &mut violations,
                "ROOT-027",
                "root.surface.ssot_roots",
                Some("root-surface.json".to_string()),
                format!("missing ssot root declaration: {required}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_028_manifest_docs_governed(ctx: &RunContext) -> TestResult {
    let payload = match root_surface_manifest(ctx, "ROOT-028", "root.surface.docs_governed") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let ssot_roots = match payload["ssot_roots"].as_array() {
        Some(values) => values
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-028".to_string(),
                test_id: "root.surface.docs_governed".to_string(),
                file: Some("root-surface.json".to_string()),
                line: None,
                message: "`ssot_roots` array is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    if !ctx.repo_root.join("docs").is_dir() {
        push_root_violation(
            &mut violations,
            "ROOT-028",
            "root.surface.docs_governed",
            Some("docs".to_string()),
            "docs/ must exist at the repo root",
        );
    }
    if !ssot_roots.contains("docs") {
        push_root_violation(
            &mut violations,
            "ROOT-028",
            "root.surface.docs_governed",
            Some("root-surface.json".to_string()),
            "docs must be declared as a governed root in ssot_roots",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_029_no_nested_git(ctx: &RunContext) -> TestResult {
    fn collect(dir: &std::path::Path, root: &std::path::Path, violations: &mut Vec<Violation>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if path == root.join(".git") || name == ".idea" || name == "artifacts" || name == "target" {
                continue;
            }
            if name == ".git" {
                let rel = path
                    .strip_prefix(root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                violations.push(Violation {
                    contract_id: "ROOT-029".to_string(),
                    test_id: "root.surface.no_nested_git".to_string(),
                    file: Some(rel),
                    line: None,
                    message: "nested git repository is forbidden".to_string(),
                    evidence: None,
                });
                continue;
            }
            collect(&path, root, violations);
        }
    }

    let mut violations = Vec::new();
    collect(&ctx.repo_root, &ctx.repo_root, &mut violations);
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_030_no_vendor_blobs(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-030".to_string(),
                test_id: "root.surface.no_vendor_blobs".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_ascii_lowercase();
        if lower == "vendor" || lower.starts_with("vendor-") || lower.ends_with("-vendor") {
            push_root_violation(
                &mut violations,
                "ROOT-030",
                "root.surface.no_vendor_blobs",
                Some(name),
                "vendor directories and blobs are forbidden at the repo root",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_031_root_file_size_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-031".to_string(),
                test_id: "root.surface.root_file_size_budget".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                push_root_violation(
                    &mut violations,
                    "ROOT-031",
                    "root.surface.root_file_size_budget",
                    Some(name),
                    format!("read metadata failed: {err}"),
                );
                continue;
            }
        };
        if metadata.len() > ROOT_FILE_SIZE_BUDGET_BYTES {
            push_root_violation(
                &mut violations,
                "ROOT-031",
                "root.surface.root_file_size_budget",
                Some(name),
                format!(
                    "root file exceeds size budget: {} > {} bytes",
                    metadata.len(),
                    ROOT_FILE_SIZE_BUDGET_BYTES
                ),
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

fn test_root_032_no_nested_toolchain_pins(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for root in ["configs", "ops"] {
        let path = ctx.repo_root.join(root);
        if !path.is_dir() {
            continue;
        }
        let mut found = Vec::new();
        collect_named_files(&path, "rust-toolchain.toml", &mut found);
        collect_named_files(&path, "rust-toolchain", &mut found);
        for item in found {
            let rel = item
                .strip_prefix(&ctx.repo_root)
                .map(|value| value.display().to_string())
                .unwrap_or_else(|_| item.display().to_string());
            push_root_violation(
                &mut violations,
                "ROOT-032",
                "root.surface.no_nested_toolchain_pins",
                Some(rel),
                "nested toolchain authority is forbidden under configs/ or ops/",
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

fn test_root_021_editorconfig_exists(ctx: &RunContext) -> TestResult {
    if ctx.repo_root.join(".editorconfig").is_file() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-021".to_string(),
            test_id: "root.editorconfig.exists".to_string(),
            file: Some(".editorconfig".to_string()),
            line: None,
            message: ".editorconfig must exist at the repo root".to_string(),
            evidence: None,
        }])
    }
}

fn test_root_022_license_single_authority(ctx: &RunContext) -> TestResult {
    let cargo = match read_root_text(ctx, "Cargo.toml", "ROOT-022", "root.license.single_authority") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let readme = match read_root_text(ctx, "README.md", "ROOT-022", "root.license.single_authority") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    if cargo.contains("MIT") || cargo.contains("GPL") {
        push_root_violation(
            &mut violations,
            "ROOT-022",
            "root.license.single_authority",
            Some("Cargo.toml".to_string()),
            "workspace metadata must not declare a conflicting license family",
        );
    }
    if readme.contains("MIT") || readme.contains("GPL") {
        push_root_violation(
            &mut violations,
            "ROOT-022",
            "root.license.single_authority",
            Some("README.md".to_string()),
            "root README must not describe conflicting license families",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_023_readme_entrypoint_sections(ctx: &RunContext) -> TestResult {
    let contents = match read_root_text(ctx, "README.md", "ROOT-023", "root.readme.entrypoint_sections") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for required_heading in [
        "## Product Narrative",
        "## Quick Start",
        "## Documentation Entrypoints",
        "## Repository Surfaces",
    ] {
        if !contents.contains(required_heading) {
            push_root_violation(
                &mut violations,
                "ROOT-023",
                "root.readme.entrypoint_sections",
                Some("README.md".to_string()),
                format!("missing required README section: {required_heading}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_024_docs_no_legacy_links(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let root_docs = ["README.md", "CONTRIBUTING.md", "SECURITY.md", "CHANGELOG.md"];
    let forbidden_patterns = [("scripts/", "legacy scripts directory reference"), ("xtask", "legacy xtask reference")];
    for relative in root_docs {
        let contents = match read_root_text(ctx, relative, "ROOT-024", "root.docs.no_legacy_links") {
            Ok(contents) => contents,
            Err(result) => return result,
        };
        for (pattern, message) in forbidden_patterns {
            if contents.contains(pattern) {
                push_root_violation(
                    &mut violations,
                    "ROOT-024",
                    "root.docs.no_legacy_links",
                    Some(relative.to_string()),
                    format!("{message}: `{pattern}`"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_025_support_routing(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for forbidden in ["SUPPORT.md", "SUPPORT_MATRIX.md", "SUPPORT-MATRIX.md"] {
        if ctx.repo_root.join(forbidden).exists() {
            push_root_violation(
                &mut violations,
                "ROOT-025",
                "root.docs.support_routing",
                Some(forbidden.to_string()),
                "support routing must live under docs/ or ops/, not as a new root authority file",
            );
        }
    }
    let readme = match read_root_text(ctx, "README.md", "ROOT-025", "root.docs.support_routing") {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    if readme.to_ascii_lowercase().contains("support matrix") && !readme.contains("docs/") && !readme.contains("ops/") {
        push_root_violation(
            &mut violations,
            "ROOT-025",
            "root.docs.support_routing",
            Some("README.md".to_string()),
            "support matrix references in README.md must route into docs/ or ops/",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_026_no_duplicate_policy_dirs(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for forbidden in ["policy", "policies"] {
        if ctx.repo_root.join(forbidden).is_dir() {
            push_root_violation(
                &mut violations,
                "ROOT-026",
                "root.surface.no_duplicate_policy_dirs",
                Some(forbidden.to_string()),
                "policy directories must stay under the canonical configs/ surface",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
