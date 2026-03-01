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
    let redirects = match serde_json::from_str::<std::collections::BTreeMap<String, String>>(&contents)
    {
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
    let mut violations = Vec::new();
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
