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
    let regex = regex::Regex::new(r"^- Last verified against: `(?:main|v[^`@]+)@[0-9a-f]{40}`$")
        .unwrap_or_else(|err| panic!("invalid last-verified regex: {err}"));
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

fn test_docs_069_high_value_pages_keep_governed_ownership_registry(ctx: &RunContext) -> TestResult {
    let relative = "docs/_internal/governance/docs-page-ownership-registry.json";
    let contents = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-069".to_string(),
                test_id: "docs.metadata.high_value_ownership_registry".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let registry = match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&contents) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-069".to_string(),
                test_id: "docs.metadata.high_value_ownership_registry".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("invalid json: {err}"),
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    for (path, metadata) in registry {
        if !ctx.repo_root.join(&path).exists() {
            violations.push(Violation {
                contract_id: "DOC-069".to_string(),
                test_id: "docs.metadata.high_value_ownership_registry".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("registry path `{path}` does not exist"),
                evidence: None,
            });
            continue;
        }
        for key in ["owner", "reviewCadence", "stability"] {
            if metadata.get(key).and_then(|value| value.as_str()).unwrap_or("").is_empty() {
                violations.push(Violation {
                    contract_id: "DOC-069".to_string(),
                    test_id: "docs.metadata.high_value_ownership_registry".to_string(),
                    file: Some(relative.to_string()),
                    line: None,
                    message: format!("registry entry `{path}` must declare non-empty `{key}`"),
                    evidence: None,
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

fn test_docs_070_docs_health_dashboard_exists_and_is_generated(ctx: &RunContext) -> TestResult {
    let relative = "docs/_internal/generated/docs-health-dashboard.md";
    let contents = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-070".to_string(),
                test_id: "docs.generated.health_dashboard_present".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    if !contents.contains("- Generated by: `bijux-dev-atlas docs health-dashboard --allow-write`") {
        violations.push(Violation {
            contract_id: "DOC-070".to_string(),
            test_id: "docs.generated.health_dashboard_present".to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: "docs health dashboard must keep the canonical generated-by line".to_string(),
            evidence: None,
        });
    }
    if !contents.contains("- Do not edit by hand:") {
        violations.push(Violation {
            contract_id: "DOC-070".to_string(),
            test_id: "docs.generated.health_dashboard_present".to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: "docs health dashboard must keep the canonical do-not-edit line".to_string(),
            evidence: None,
        });
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_071_readme_files_do_not_link_to_stub_reference_pages(ctx: &RunContext) -> TestResult {
    let policy_relative = "docs/_internal/governance/authored-minified-pages.json";
    let policy_text = match std::fs::read_to_string(ctx.repo_root.join(policy_relative)) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-071".to_string(),
                test_id: "docs.links.readme_avoids_stub_reference_pages".to_string(),
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
                contract_id: "DOC-071".to_string(),
                test_id: "docs.links.readme_avoids_stub_reference_pages".to_string(),
                file: Some(policy_relative.to_string()),
                line: None,
                message: format!("invalid json: {err}"),
                evidence: None,
            }]);
        }
    };
    let stub_targets = policy["exemptions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value["path"].as_str().map(|value| value.strip_prefix("docs/").unwrap_or(value).to_string()))
        .collect::<Vec<_>>();
    let link_re = regex::Regex::new(r"\[[^\]]+\]\(([^)]+)\)")
        .unwrap_or_else(|err| panic!("invalid markdown link regex: {err}"));
    let mut violations = Vec::new();
    for path in docs_markdown_paths_under(&ctx.repo_root) {
        let relative = path.strip_prefix(&ctx.repo_root).unwrap_or(&path);
        let rel_string = relative.display().to_string();
        if !relative
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "README.md")
            .unwrap_or(false)
        {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        for capture in link_re.captures_iter(&text) {
            let target = capture.get(1).map(|m| m.as_str()).unwrap_or_default();
            let target = target.split('#').next().unwrap_or_default();
            if stub_targets.iter().any(|stub| target.ends_with(stub)) {
                violations.push(Violation {
                    contract_id: "DOC-071".to_string(),
                    test_id: "docs.links.readme_avoids_stub_reference_pages".to_string(),
                    file: Some(rel_string.clone()),
                    line: None,
                    message: format!("README may not link directly to stub-style docs page `{target}`"),
                    evidence: None,
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

fn test_docs_072_redirect_registry_covers_redirects_and_inventory_matches(ctx: &RunContext) -> TestResult {
    let redirects = match docs_parse_redirect_mapping(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-072".to_string(),
                test_id: "docs.redirects.registry_and_inventory".to_string(),
                file: Some("docs/redirects.json".to_string()),
                line: None,
                message: err,
                evidence: None,
            }]);
        }
    };
    let registry_relative = "docs/_internal/governance/redirect-registry.json";
    let registry_text = match std::fs::read_to_string(ctx.repo_root.join(registry_relative)) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-072".to_string(),
                test_id: "docs.redirects.registry_and_inventory".to_string(),
                file: Some(registry_relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let registry: serde_json::Value = match serde_json::from_str(&registry_text) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-072".to_string(),
                test_id: "docs.redirects.registry_and_inventory".to_string(),
                file: Some(registry_relative.to_string()),
                line: None,
                message: format!("invalid json: {err}"),
                evidence: None,
            }]);
        }
    };
    let rules = registry["rules"].as_array().cloned().unwrap_or_default();
    let inventory_relative = "docs/_internal/generated/legacy-url-inventory.md";
    let inventory_text = std::fs::read_to_string(ctx.repo_root.join(inventory_relative)).unwrap_or_default();
    let mut violations = Vec::new();
    for rule in &rules {
        if rule.get("owner").and_then(|value| value.as_str()).unwrap_or("").is_empty() {
            violations.push(Violation {
                contract_id: "DOC-072".to_string(),
                test_id: "docs.redirects.registry_and_inventory".to_string(),
                file: Some(registry_relative.to_string()),
                line: None,
                message: "every redirect registry rule must declare an owner".to_string(),
                evidence: None,
            });
        }
        if rule.get("reason").and_then(|value| value.as_str()).unwrap_or("").is_empty() {
            violations.push(Violation {
                contract_id: "DOC-072".to_string(),
                test_id: "docs.redirects.registry_and_inventory".to_string(),
                file: Some(registry_relative.to_string()),
                line: None,
                message: "every redirect registry rule must declare a reason".to_string(),
                evidence: None,
            });
        }
        if rule.get("temporary").and_then(|value| value.as_bool()).unwrap_or(false)
            && rule.get("expiresOn").and_then(|value| value.as_str()).unwrap_or("").is_empty()
        {
            violations.push(Violation {
                contract_id: "DOC-072".to_string(),
                test_id: "docs.redirects.registry_and_inventory".to_string(),
                file: Some(registry_relative.to_string()),
                line: None,
                message: "temporary redirect registry rules must declare expiresOn".to_string(),
                evidence: None,
            });
        }
    }
    for source in redirects.keys() {
        let resolved = rules.iter().any(|rule| {
            rule.get("matchPath")
                .and_then(|value| value.as_str())
                .map(|value| value == source)
                .unwrap_or(false)
                || rule
                    .get("matchPrefix")
                    .and_then(|value| value.as_str())
                    .map(|value| source.starts_with(value))
                    .unwrap_or(false)
        });
        if !resolved {
            violations.push(Violation {
                contract_id: "DOC-072".to_string(),
                test_id: "docs.redirects.registry_and_inventory".to_string(),
                file: Some(registry_relative.to_string()),
                line: None,
                message: format!("redirect source `{source}` does not resolve redirect metadata"),
                evidence: None,
            });
        }
    }
    let expected_inventory = docs_render_legacy_url_inventory(&redirects, &registry);
    if inventory_text != expected_inventory {
        violations.push(Violation {
            contract_id: "DOC-072".to_string(),
            test_id: "docs.redirects.registry_and_inventory".to_string(),
            file: Some(inventory_relative.to_string()),
            line: None,
            message: "legacy URL inventory is out of sync with redirects.json and redirect-registry.json".to_string(),
            evidence: None,
        });
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_073_mkdocs_nav_references_real_files(ctx: &RunContext) -> TestResult {
    let refs = match docs_mkdocs_nav_refs(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-073".to_string(),
                test_id: "docs.navigation.mkdocs_nav_targets_exist".to_string(),
                file: Some("mkdocs.yml".to_string()),
                line: None,
                message: format!("mkdocs nav parse failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    for (_, rel) in refs {
        if !ctx.repo_root.join("docs").join(&rel).exists() {
            violations.push(Violation {
                contract_id: "DOC-073".to_string(),
                test_id: "docs.navigation.mkdocs_nav_targets_exist".to_string(),
                file: Some("mkdocs.yml".to_string()),
                line: None,
                message: format!("mkdocs nav references missing file `docs/{rel}`"),
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

fn test_docs_074_section_indexes_include_purpose_and_entrypoints(ctx: &RunContext) -> TestResult {
    let indexes = [
        "docs/product/index.md",
        "docs/architecture/index.md",
        "docs/api/index.md",
        "docs/operations/index.md",
        "docs/control-plane/index.md",
        "docs/development/index.md",
        "docs/reference/index.md",
        "docs/_internal/governance/index.md",
    ];
    let purpose_markers = ["## Purpose", "## Why you are reading this", "## Why this section exists"];
    let entrypoint_markers = ["## Entry Points", "## Entrypoints", "## What You Will Find Here", "## Curated guide map"];
    let mut violations = Vec::new();
    for relative in indexes {
        let text = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
            Ok(text) => text,
            Err(err) => {
                violations.push(Violation {
                    contract_id: "DOC-074".to_string(),
                    test_id: "docs.structure.section_indexes_have_purpose_and_entrypoints".to_string(),
                    file: Some(relative.to_string()),
                    line: None,
                    message: format!("read failed: {err}"),
                    evidence: None,
                });
                continue;
            }
        };
        if !purpose_markers.iter().any(|marker| text.contains(marker)) {
            violations.push(Violation {
                contract_id: "DOC-074".to_string(),
                test_id: "docs.structure.section_indexes_have_purpose_and_entrypoints".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: "section index must include a purpose heading".to_string(),
                evidence: None,
            });
        }
        if !entrypoint_markers.iter().any(|marker| text.contains(marker)) {
            violations.push(Violation {
                contract_id: "DOC-074".to_string(),
                test_id: "docs.structure.section_indexes_have_purpose_and_entrypoints".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: "section index must include entrypoint guidance".to_string(),
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

fn test_docs_075_operations_runbooks_include_required_sections(ctx: &RunContext) -> TestResult {
    let mut targets = docs_markdown_paths_under(&ctx.repo_root.join("docs/operations/runbooks"));
    targets.push(ctx.repo_root.join("docs/operations/runbook-template.md"));
    let mut violations = Vec::new();
    for path in targets {
        let relative = path.strip_prefix(&ctx.repo_root).unwrap_or(&path).display().to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        for heading in ["## Prereqs", "## Verify", "## Rollback"] {
            if !text.contains(heading) {
                violations.push(Violation {
                    contract_id: "DOC-075".to_string(),
                    test_id: "docs.operations.runbooks_have_required_sections".to_string(),
                    file: Some(relative.clone()),
                    line: None,
                    message: format!("runbook must include `{heading}`"),
                    evidence: None,
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

fn test_docs_076_major_docs_changes_require_explicit_owner_approval(ctx: &RunContext) -> TestResult {
    let relative = "docs/_internal/governance/docs-change-classification.md";
    let text = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-076".to_string(),
                test_id: "docs.governance.major_changes_require_owner_approval".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    if text.contains("explicit `docs-governance` owner approval before merge") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-076".to_string(),
            test_id: "docs.governance.major_changes_require_owner_approval".to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: "major changes policy must require explicit docs-governance owner approval before merge".to_string(),
            evidence: None,
        }])
    }
}

fn test_docs_077_committed_docs_artifacts_use_canonical_regeneration_flow(
    ctx: &RunContext,
) -> TestResult {
    let artifact_contract_path = "docs/_internal/governance/docs-artifact-contract.md";
    let regenerate_path = "docs/_internal/governance/regenerate-committed-artifacts.md";
    let artifact_contract = match std::fs::read_to_string(ctx.repo_root.join(artifact_contract_path)) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-077".to_string(),
                test_id: "docs.governance.committed_artifacts_use_canonical_regeneration".to_string(),
                file: Some(artifact_contract_path.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let regenerate_doc = match std::fs::read_to_string(ctx.repo_root.join(regenerate_path)) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-077".to_string(),
                test_id: "docs.governance.committed_artifacts_use_canonical_regeneration".to_string(),
                file: Some(regenerate_path.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };

    let mut violations = Vec::new();
    if !artifact_contract.contains("must be refreshed with the control-plane") {
        violations.push(Violation {
            contract_id: "DOC-077".to_string(),
            test_id: "docs.governance.committed_artifacts_use_canonical_regeneration".to_string(),
            file: Some(artifact_contract_path.to_string()),
            line: None,
            message: "docs artifact contract must require committed generated markdown to be refreshed with the control-plane".to_string(),
            evidence: None,
        });
    }
    if !regenerate_doc.contains("bijux-dev-atlas docs") {
        violations.push(Violation {
            contract_id: "DOC-077".to_string(),
            test_id: "docs.governance.committed_artifacts_use_canonical_regeneration".to_string(),
            file: Some(regenerate_path.to_string()),
            line: None,
            message: "regeneration guide must declare the canonical bijux-dev-atlas docs command flow".to_string(),
            evidence: None,
        });
    }
    if !regenerate_doc.contains("Do not edit committed generated markdown directly") {
        violations.push(Violation {
            contract_id: "DOC-077".to_string(),
            test_id: "docs.governance.committed_artifacts_use_canonical_regeneration".to_string(),
            file: Some(regenerate_path.to_string()),
            line: None,
            message: "regeneration guide must forbid manual edits to committed generated markdown".to_string(),
            evidence: None,
        });
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

fn docs_parse_redirect_mapping(
    repo_root: &std::path::Path,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let relative = "docs/redirects.json";
    let contents = std::fs::read_to_string(repo_root.join(relative))
        .map_err(|err| format!("read {relative} failed: {err}"))?;
    serde_json::from_str::<std::collections::BTreeMap<String, String>>(&contents)
        .map_err(|err| format!("parse {relative} failed: {err}"))
}

fn docs_render_legacy_url_inventory(
    redirects: &std::collections::BTreeMap<String, String>,
    registry: &serde_json::Value,
) -> String {
    fn match_rule<'a>(source: &str, rules: &'a [serde_json::Value]) -> Option<&'a serde_json::Value> {
        for rule in rules {
            if let Some(path) = rule.get("matchPath").and_then(|value| value.as_str()) {
                if source == path {
                    return Some(rule);
                }
            }
            if let Some(prefix) = rule.get("matchPrefix").and_then(|value| value.as_str()) {
                if source.starts_with(prefix) {
                    return Some(rule);
                }
            }
        }
        None
    }

    let rules = registry["rules"].as_array().cloned().unwrap_or_default();
    let mut out = String::from("# Legacy URL Inventory\n\n");
    out.push_str("- Generated by: `bijux-dev-atlas docs redirects sync --allow-write`\n");
    out.push_str("- Do not edit by hand: regenerate with the control-plane command.\n\n");
    out.push_str("| Legacy Path | Current Path | Owner | Reason | Temporary Until |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for (source, target) in redirects {
        let rule = match_rule(source, &rules).cloned().unwrap_or(serde_json::json!({}));
        let owner = rule["owner"].as_str().unwrap_or("unassigned");
        let reason = rule["reason"].as_str().unwrap_or("missing redirect metadata");
        let expiry = rule["expiresOn"].as_str().unwrap_or("none");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | `{}` |\n",
            source, target, owner, reason, expiry
        ));
    }
    out
}

fn docs_mkdocs_nav_refs(repo_root: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    fn collect_refs(node: &serde_yaml::Value, refs: &mut Vec<(String, String)>) {
        match node {
            serde_yaml::Value::Sequence(items) => {
                for item in items {
                    collect_refs(item, refs);
                }
            }
            serde_yaml::Value::Mapping(map) => {
                for (key, value) in map {
                    if let (Some(title), Some(path)) = (key.as_str(), value.as_str()) {
                        refs.push((title.to_string(), path.to_string()));
                    } else {
                        collect_refs(value, refs);
                    }
                }
            }
            _ => {}
        }
    }

    let mkdocs_path = repo_root.join("mkdocs.yml");
    let yaml_text = std::fs::read_to_string(&mkdocs_path)
        .map_err(|err| format!("read {} failed: {err}", mkdocs_path.display()))?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
        .map_err(|err| format!("parse {} failed: {err}", mkdocs_path.display()))?;
    let nav = yaml
        .get("nav")
        .ok_or_else(|| "mkdocs.yml missing `nav`".to_string())?;
    let mut refs = Vec::new();
    collect_refs(nav, &mut refs);
    refs.sort();
    Ok(refs)
}
