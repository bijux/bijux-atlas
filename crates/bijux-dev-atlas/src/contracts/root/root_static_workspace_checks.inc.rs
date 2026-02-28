fn test_root_039_workspace_members_match(ctx: &RunContext) -> TestResult {
    let cargo = match read_root_text(
        ctx,
        "Cargo.toml",
        "ROOT-039",
        "root.cargo.workspace_members_match",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let mut in_members = false;
    let mut declared = Vec::new();
    for line in cargo.lines() {
        let trimmed = line.trim();
        if !in_members && trimmed.starts_with("members = [") {
            in_members = true;
            continue;
        }
        if !in_members {
            continue;
        }
        if trimmed.starts_with(']') {
            break;
        }
        if let Some(stripped) = trimmed.strip_prefix('"').and_then(|value| value.strip_suffix("\",")) {
            declared.push(stripped.to_string());
        }
    }
    let mut actual = Vec::new();
    let crates_dir = ctx.repo_root.join("crates");
    let entries = match std::fs::read_dir(&crates_dir) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-039".to_string(),
                test_id: "root.cargo.workspace_members_match".to_string(),
                file: Some("crates".to_string()),
                line: None,
                message: format!("read crates/ failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("Cargo.toml").is_file() {
            actual.push(format!("crates/{}", entry.file_name().to_string_lossy()));
        }
    }
    declared.sort();
    actual.sort();
    if declared == actual {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "ROOT-039".to_string(),
            test_id: "root.cargo.workspace_members_match".to_string(),
            file: Some("Cargo.toml".to_string()),
            line: None,
            message: format!("workspace members drift: declared={declared:?}, actual={actual:?}"),
            evidence: None,
        }])
    }
}

fn test_root_040_crate_naming(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let crates_dir = ctx.repo_root.join("crates");
    let entries = match std::fs::read_dir(&crates_dir) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-040".to_string(),
                test_id: "root.cargo.crate_naming".to_string(),
                file: Some("crates".to_string()),
                line: None,
                message: format!("read crates/ failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let dir = entry.file_name().to_string_lossy().to_string();
        let cargo_path = entry.path().join("Cargo.toml");
        if !cargo_path.is_file() {
            continue;
        }
        if !dir.starts_with("bijux-") {
            push_root_violation(
                &mut violations,
                "ROOT-040",
                "root.cargo.crate_naming",
                Some(format!("crates/{dir}")),
                "crate directory names must start with `bijux-`",
            );
        }
        let rel = format!("crates/{dir}/Cargo.toml");
        let contents = match read_root_text(ctx, &rel, "ROOT-040", "root.cargo.crate_naming") {
            Ok(contents) => contents,
            Err(result) => return result,
        };
        let package_name = contents
            .lines()
            .find_map(|line| line.trim().strip_prefix("name = "))
            .map(|value| value.trim().trim_matches('"').to_string());
        if package_name.as_deref() != Some(&dir) {
            push_root_violation(
                &mut violations,
                "ROOT-040",
                "root.cargo.crate_naming",
                Some(rel),
                format!("package name {:?} must match directory name {dir}", package_name),
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
