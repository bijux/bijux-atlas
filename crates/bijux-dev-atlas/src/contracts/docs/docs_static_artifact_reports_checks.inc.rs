fn test_docs_032_broken_links_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_broken_links_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-032",
        "docs.reports.broken_links_generated",
        "broken-links.json",
        &payload,
    )
}

fn test_docs_033_orphans_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_orphans_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-033",
        "docs.reports.orphans_generated",
        "orphans.json",
        &payload,
    )
}

fn test_docs_034_metadata_coverage_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_metadata_coverage_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-034",
        "docs.reports.metadata_coverage_generated",
        "metadata-coverage.json",
        &payload,
    )
}

fn test_docs_035_duplication_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_duplication_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-035",
        "docs.reports.duplication_generated",
        "duplication-report.json",
        &payload,
    )
}

fn test_docs_036_coverage_report_generated(ctx: &RunContext) -> TestResult {
    let rows = match contracts(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-036".to_string(),
                test_id: "docs.reports.coverage_generated".to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("load docs contracts failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let static_count = rows
        .iter()
        .filter(|row| row.tests.iter().all(|case| case.kind == TestKind::Pure))
        .count();
    let effect_count = rows.len().saturating_sub(static_count);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "docs_contract_coverage",
        "group": "docs",
        "contracts_total": rows.len(),
        "contracts_static": static_count,
        "contracts_effect": effect_count,
        "tests_total": rows.iter().map(|row| row.tests.len()).sum::<usize>(),
        "generated_by": "bijux dev atlas contracts docs"
    });
    write_docs_report_artifact(
        ctx,
        "DOC-036",
        "docs.reports.coverage_generated",
        "coverage-report.json",
        &payload,
    )
}

fn test_docs_058_generated_docs_live_under_internal(ctx: &RunContext) -> TestResult {
    let generated_dirs = docs_relative_directories_named(ctx, "_generated");
    let violations = generated_dirs
        .into_iter()
        .filter(|path| path != "docs/_internal/generated")
        .map(|path| Violation {
            contract_id: "DOC-058".to_string(),
            test_id: "docs.artifacts.generated_under_internal_only".to_string(),
            file: Some(path),
            line: None,
            message: "generated docs directories must live under docs/_internal/generated only"
                .to_string(),
            evidence: None,
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_059_dashboard_links_required_artifacts(ctx: &RunContext) -> TestResult {
    let relative = "docs/_internal/governance/docs-dashboard.md";
    let contents = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-059".to_string(),
                test_id: "docs.artifacts.dashboard_links_required_outputs".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let required_targets = [
        "../generated/governance-audit/index.md",
        "../generated/governance-audit/docs-broken-links.csv",
        "../generated/governance-audit/docs-dead-end-pages.txt",
        "../generated/governance-audit/docs-top-delete-pages.md",
        "../generated/governance-audit/docs-top-merge-clusters.md",
        "../generated/governance-audit/docs-uppercase-index-pages.txt",
        "../generated/governance-audit/docs-inventory.csv",
        "../generated/docs-quality-dashboard.json",
        "../generated/docs-dependency-graph.json",
        "../generated/docs-contract-coverage.json",
        "../generated/sitemap.json",
        "../generated/search-index.json",
    ];
    let links = markdown_links_for_reports(&contents);
    let mut violations = Vec::new();
    for target in required_targets {
        if !links.iter().any(|link| link == target) {
            violations.push(Violation {
                contract_id: "DOC-059".to_string(),
                test_id: "docs.artifacts.dashboard_links_required_outputs".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("Docs Dashboard must link `{target}`"),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_060_redirects_target_real_pages(ctx: &RunContext) -> TestResult {
    let relative = "docs/redirects.json";
    let contents = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let redirects =
        match serde_json::from_str::<std::collections::BTreeMap<String, String>>(&contents) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("invalid json: {err}"),
                evidence: None,
            }]);
        }
    };
    let legacy_policy = ctx
        .repo_root
        .join("docs/_internal/governance/redirect-legacy-policy.json");
    let internal_target_policy = ctx
        .repo_root
        .join("docs/_internal/governance/redirect-internal-target-policy.json");
    let legacy_policy: serde_json::Value = match std::fs::read_to_string(&legacy_policy)
        .ok()
        .and_then(|value| serde_json::from_str(&value).ok())
    {
        Some(value) => value,
        None => serde_json::json!({ "allowedPrefixes": [], "allowedPaths": [] }),
    };
    let internal_target_policy: serde_json::Value =
        match std::fs::read_to_string(&internal_target_policy)
            .ok()
            .and_then(|value| serde_json::from_str(&value).ok())
        {
            Some(value) => value,
            None => serde_json::json!({ "allowedPrefixes": [] }),
        };
    let legacy_prefixes = legacy_policy["allowedPrefixes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let legacy_paths = legacy_policy["allowedPaths"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let internal_target_prefixes = internal_target_policy["allowedPrefixes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let mut violations = Vec::new();
    let mut duplicate_keys = std::collections::BTreeSet::new();
    let mut seen_keys = std::collections::BTreeSet::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("\"docs/") {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once("\": ") {
            let key = key.trim_matches('"');
            if !seen_keys.insert(key.to_string()) {
                duplicate_keys.insert(key.to_string());
            }
        }
    }
    for key in duplicate_keys {
        violations.push(Violation {
            contract_id: "DOC-060".to_string(),
            test_id: "docs.artifacts.redirects_integrity".to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: format!("redirect source `{key}` may not appear more than once"),
            evidence: None,
        });
    }
    for (source, target) in redirects {
        if !source.starts_with("docs/") {
            violations.push(Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("redirect source `{source}` must stay under docs/"),
                evidence: None,
            });
        }
        if !target.starts_with("docs/") {
            violations.push(Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("redirect target `{target}` must stay under docs/"),
                evidence: None,
            });
        }
        if !source.ends_with(".md") || !target.ends_with(".md") {
            violations.push(Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("redirect `{source}` -> `{target}` must stay markdown-to-markdown"),
                evidence: None,
            });
        }
        if source == target {
            violations.push(Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("redirect `{source}` may not point to itself"),
                evidence: None,
            });
        }
        if !ctx.repo_root.join(&source).exists()
            && !legacy_paths.iter().any(|path| path == &source)
            && !legacy_prefixes.iter().any(|prefix| source.starts_with(prefix))
        {
            violations.push(Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("redirect source `{source}` must exist now or be declared legacy"),
                evidence: None,
            });
        }
        if !ctx.repo_root.join(&target).exists() {
            violations.push(Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("redirect target `{target}` does not exist"),
                evidence: None,
            });
        }
        if target.starts_with("docs/_internal/")
            && !internal_target_prefixes
                .iter()
                .any(|prefix| target.starts_with(prefix))
        {
            violations.push(Violation {
                contract_id: "DOC-060".to_string(),
                test_id: "docs.artifacts.redirects_integrity".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("redirect target `{target}` may not point into docs/_internal without an explicit allowlist"),
                evidence: None,
            });
        }
    }
    let redirect_map = match serde_json::from_str::<std::collections::BTreeMap<String, String>>(&contents) {
        Ok(value) => value,
        Err(_) => std::collections::BTreeMap::new(),
    };
    for source in redirect_map.keys() {
        let mut seen = std::collections::BTreeSet::new();
        let mut current = source.as_str();
        while let Some(next) = redirect_map.get(current) {
            if !seen.insert(current.to_string()) {
                violations.push(Violation {
                    contract_id: "DOC-060".to_string(),
                    test_id: "docs.artifacts.redirects_integrity".to_string(),
                    file: Some(relative.to_string()),
                    line: None,
                    message: format!("redirect chain from `{source}` contains a loop"),
                    evidence: None,
                });
                break;
            }
            current = next;
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.message.cmp(&b.message));
        TestResult::Fail(violations)
    }
}

fn test_docs_061_registry_schema_validation(ctx: &RunContext) -> TestResult {
    validate_docs_registry_schema(ctx)
}

fn test_docs_062_sections_schema_validation(ctx: &RunContext) -> TestResult {
    validate_docs_sections_schema(ctx)
}

fn test_docs_065_repo_map_curated_sources_exist(ctx: &RunContext) -> TestResult {
    let relative = "docs/reference/repo-map.md";
    let contents = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-065".to_string(),
                test_id: "docs.reference.repo_map_curated".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    let source_of_truth = "docs/_internal/generated/docs-inventory.md";
    if !contents.contains(&format!("- Source-of-truth: `{source_of_truth}`")) {
        violations.push(Violation {
            contract_id: "DOC-065".to_string(),
            test_id: "docs.reference.repo_map_curated".to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: "repo map must declare the canonical generated docs inventory as its source of truth".to_string(),
            evidence: None,
        });
    }
    if !contents.contains("[Repository Layout](../development/repo-layout.md)") {
        violations.push(Violation {
            contract_id: "DOC-065".to_string(),
            test_id: "docs.reference.repo_map_curated".to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: "repo map must link the curated repository layout guide".to_string(),
            evidence: None,
        });
    }
    for linked in [source_of_truth, "docs/development/repo-layout.md"] {
        if !ctx.repo_root.join(linked).exists() {
            violations.push(Violation {
                contract_id: "DOC-065".to_string(),
                test_id: "docs.reference.repo_map_curated".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("repo map linked surface `{linked}` does not exist"),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_066_verification_markers_are_canonical(ctx: &RunContext) -> TestResult {
    let regex = regex::Regex::new(r"^- Last verified against: `(?:main|v[^`@]+)@[0-9a-f]{40}`$").unwrap();
    let mut violations = Vec::new();
    for path in docs_markdown_paths_under(&ctx.repo_root.join("docs")) {
        let relative = path.strip_prefix(&ctx.repo_root).unwrap_or(&path);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        for (idx, line) in text.lines().enumerate() {
            if line.contains("Last verified against:") && !regex.is_match(line.trim()) {
                violations.push(Violation {
                    contract_id: "DOC-066".to_string(),
                    test_id: "docs.metadata.canonical_last_verified".to_string(),
                    file: Some(relative.display().to_string()),
                    line: Some(idx + 1),
                    message: "Last verified against must use canonical explicit ref plus full SHA form".to_string(),
                    evidence: Some(line.trim().to_string()),
                });
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_067_generated_markdown_has_required_header(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let generated_root = ctx.repo_root.join("docs/_internal/generated");
    for path in docs_markdown_paths_under(&generated_root) {
        let relative = path.strip_prefix(&ctx.repo_root).unwrap_or(&path);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        if !text.contains("Generated by:") || !text.contains("Do not edit by hand:") {
            violations.push(Violation {
                contract_id: "DOC-067".to_string(),
                test_id: "docs.generated.generator_headers".to_string(),
                file: Some(relative.display().to_string()),
                line: None,
                message: "generated markdown must include both Generated by and Do not edit by hand markers".to_string(),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_068_authored_markdown_not_minified_without_exemption(ctx: &RunContext) -> TestResult {
    let policy_relative = "docs/_internal/governance/authored-minified-pages.json";
    let policy_text = match std::fs::read_to_string(ctx.repo_root.join(policy_relative)) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-068".to_string(),
                test_id: "docs.reviewability.authored_markdown_not_minified".to_string(),
                file: Some(policy_relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let policy: serde_json::Value = match serde_json::from_str(&policy_text) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-068".to_string(),
                test_id: "docs.reviewability.authored_markdown_not_minified".to_string(),
                file: Some(policy_relative.to_string()),
                line: None,
                message: format!("invalid json: {err}"),
                evidence: None,
            }]);
        }
    };
    let max_non_empty_lines = policy["maxNonEmptyLines"].as_u64().unwrap_or(2) as usize;
    let exemptions = policy["exemptions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value["path"].as_str().map(|s| s.to_string()))
        .collect::<std::collections::BTreeSet<_>>();
    let mut violations = Vec::new();
    for exempt in &exemptions {
        if !ctx.repo_root.join(exempt).exists() {
            violations.push(Violation {
                contract_id: "DOC-068".to_string(),
                test_id: "docs.reviewability.authored_markdown_not_minified".to_string(),
                file: Some(policy_relative.to_string()),
                line: None,
                message: format!("reviewability exemption `{exempt}` does not exist"),
                evidence: None,
            });
        }
    }
    for path in docs_markdown_paths_under(&ctx.repo_root.join("docs")) {
        let relative = path.strip_prefix(&ctx.repo_root).unwrap_or(&path);
        let relative_str = relative.display().to_string();
        if relative_str.starts_with("docs/_internal/generated/") {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        let non_empty = text.lines().filter(|line| !line.trim().is_empty()).count();
        if non_empty <= max_non_empty_lines && !exemptions.contains(&relative_str) {
            violations.push(Violation {
                contract_id: "DOC-068".to_string(),
                test_id: "docs.reviewability.authored_markdown_not_minified".to_string(),
                file: Some(relative_str),
                line: None,
                message: format!(
                    "authored markdown with {non_empty} non-empty lines must be expanded or explicitly exempted"
                ),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn docs_markdown_paths_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    fn visit(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, out);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    visit(root, &mut out);
    out.sort();
    out
}
