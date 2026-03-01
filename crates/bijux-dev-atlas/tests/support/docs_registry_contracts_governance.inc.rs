#[test]
#[ignore = "legacy docs registry contract pending rewrite"]
fn docs_growth_budget_and_removal_policy_are_committed() {
    let root = repo_root();
    let policy = load_json(&root.join("docs/_internal/governance/metadata/growth-budget.json"));
    let max = policy["max_markdown_files"]
        .as_u64()
        .expect("max_markdown_files") as usize;
    let count = markdown_files(&root.join("docs")).len();
    assert!(
        count <= max,
        "docs markdown count {} exceeds budget {}",
        count,
        max
    );
    let removal = read(&root.join("docs/_internal/governance/docs-removal-policy.md"));
    for required in [
        "Deleting docs is allowed",
        "Adding new stable docs requires explicit justification",
        "growth budget contract",
    ] {
        assert!(
            removal.contains(required),
            "docs removal policy missing `{required}`"
        );
    }
}

#[test]
#[ignore = "legacy docs registry contract pending rewrite"]
fn runbook_and_decision_templates_are_canonical_and_enforced() {
    let root = repo_root();
    let runbook_template = read(&root.join("docs/operations/runbook-template.md"));
    for heading in [
        "## Symptoms",
        "## Metrics",
        "## Commands",
        "## Expected outputs",
        "## Mitigations",
        "## Rollback",
        "## Postmortem checklist",
    ] {
        assert!(
            runbook_template.contains(heading),
            "runbook template missing `{heading}`"
        );
    }
    let decision_template = read(&root.join("docs/_internal/governance/decision-template.md"));
    for heading in ["## Context", "## Decision", "## Consequences"] {
        assert!(
            decision_template.contains(heading),
            "decision template missing `{heading}`"
        );
    }

    let mut runbook_violations = Vec::new();
    for path in markdown_files(&root.join("docs/operations/runbooks")) {
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        if rel.ends_with("INDEX.md") {
            continue;
        }
        let text = read(&path);
        for heading in [
            "## Symptoms",
            "## Metrics",
            "## Commands",
            "## Expected outputs",
            "## Mitigations",
            "## Rollback",
            "## Postmortem checklist",
        ] {
            if !text.contains(heading) {
                runbook_violations.push(format!("{rel} missing `{heading}`"));
            }
        }
    }
    assert!(
        runbook_violations.is_empty(),
        "runbooks must follow the canonical structure:\n{}",
        runbook_violations.join("\n")
    );

    let mut adr_violations = Vec::new();
    for path in markdown_files(&root.join("docs/_internal/governance/adrs")) {
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        if rel.ends_with("INDEX.md") {
            continue;
        }
        let text = read(&path);
        for heading in ["Context:", "Decision:", "Consequences:"] {
            if !text.contains(heading) {
                adr_violations.push(format!("{rel} missing `{heading}`"));
            }
        }
    }
    assert!(
        adr_violations.is_empty(),
        "ADRs must follow the canonical decision structure:\n{}",
        adr_violations.join("\n")
    );
}

#[test]
fn docs_links_and_images_follow_governance_rules() {
    let root = repo_root();
    let allowlist = load_json(&root.join("docs/operations/external-link-allowlist.json"));
    let allowed_http = allowlist["entries"]
        .as_array()
        .expect("allowlist entries")
        .iter()
        .filter_map(|row| row["url"].as_str())
        .collect::<BTreeSet<_>>();
    let html_re = regex::Regex::new(
        r"(?i)</?(?:div|span|p|img|table|thead|tbody|tr|td|th|br|details|summary|kbd|sub|sup|a|code|pre|strong|em|ul|ol|li|blockquote|h[1-6])\b[^>]*>",
    )
    .expect("html regex");
    let image_budget = 1_500_000u64;
    let mut violations = Vec::new();

    for path in markdown_files(&root.join("docs")) {
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        let text = read(&path);
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            let code_span_count = trimmed.matches('`').count();
            if code_span_count == 0 && html_re.is_match(trimmed) {
                violations.push(format!(
                    "{rel}:{} raw HTML is forbidden unless explicitly allowlisted",
                    idx + 1
                ));
            }
        }

        for target in markdown_links(&text) {
            if target.starts_with("mailto:") || target.starts_with('#') {
                continue;
            }
            if target.starts_with("http://") {
                if !allowed_http.contains(target.as_str()) {
                    violations.push(format!("{rel} uses non-HTTPS link `{target}` outside allowlist"));
                }
                continue;
            }
            if target.starts_with("https://") {
                continue;
            }
        }

        for image in markdown_images(&text) {
            if image.starts_with("http://") || image.starts_with("https://") {
                violations.push(format!("{rel} image must be repo-local: `{image}`"));
                continue;
            }
            let image_path = path
                .parent()
                .unwrap_or(root.join("docs").as_path())
                .join(image.split('#').next().unwrap_or(&image));
            if !image_path.exists() {
                violations.push(format!("{rel} image target missing: `{image}`"));
                continue;
            }
            let size = fs::metadata(&image_path)
                .unwrap_or_else(|err| panic!("failed to stat {}: {err}", image_path.display()))
                .len();
            if size > image_budget {
                let image_rel = image_path
                    .strip_prefix(&root)
                    .unwrap_or(&image_path)
                    .display()
                    .to_string();
                violations.push(format!(
                    "{rel} image `{image_rel}` exceeds {} bytes",
                    image_budget
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "docs markdown must keep raw html out and links/images valid:\n{}",
        violations.join("\n")
    );
}

#[test]
#[ignore = "legacy docs registry contract pending rewrite"]
fn governance_docs_keep_tagged_code_blocks() {
    let root = repo_root();
    let mut violations = Vec::new();
    for rel in [
        "docs/_internal/governance/docs-removal-policy.md",
        "docs/operations/runbook-template.md",
        "docs/architecture/decision-template.md",
    ] {
        let text = read(&root.join(rel));
        for line in bare_fence_lines(&text) {
            violations.push(format!(
                "{rel}:{line} fenced code block must declare a language"
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "new governance docs must keep tagged code fences:\n{}",
        violations.join("\n")
    );
}

#[test]
fn docs_sections_requiring_indexes_have_canonical_index_pages() {
    let root = repo_root();
    let sections = section_manifest(&root);
    let section_map = sections["sections"].as_object().expect("sections object");
    let mut violations = Vec::new();
    for (section, policy) in section_map {
        if policy["requires_index"].as_bool() != Some(true) {
            continue;
        }
        let index_path = root.join("docs").join(section).join("index.md");
        if !index_path.exists() {
            violations.push(format!("docs/{section}/index.md"));
        }
    }
    assert!(
        violations.is_empty(),
        "docs sections requiring indexes must keep canonical index.md pages:\n{}",
        violations.join("\n")
    );
}

#[test]
#[ignore = "legacy docs registry contract pending rewrite"]
fn docs_top_level_surface_matches_section_owner_and_audience_registries() {
    let root = repo_root();
    let sections = section_manifest(&root);
    let section_map = sections["sections"].as_object().expect("sections object");
    let owners_json = load_json(&root.join("docs/_internal/registry/owners.json"));
    let owners = owners_json["section_owners"]
        .as_object()
        .expect("section owners");
    let audiences_json = load_json(&root.join("docs/_internal/governance/metadata/audiences.json"));
    let audiences = audiences_json["section_defaults"]
        .as_object()
        .expect("section defaults");

    let mut dirs = BTreeSet::new();
    for entry in fs::read_dir(root.join("docs")).expect("read docs root") {
        let entry = entry.expect("docs entry");
        if entry.path().is_dir() {
            dirs.insert(entry.file_name().to_string_lossy().to_string());
        }
    }

    let allowed = section_map.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        dirs, allowed,
        "docs top-level directories must match docs/_internal/registry/sections.json exactly"
    );
    for dir in &dirs {
        assert!(owners.contains_key(dir), "docs/_internal/registry/owners.json missing section owner for `{dir}`");
        assert!(
            audiences.contains_key(dir),
            "docs/_internal/governance/metadata/audiences.json missing section default for `{dir}`"
        );
    }
}

#[test]
#[ignore = "legacy docs registry contract pending rewrite"]
fn docs_generated_artifacts_are_covered_by_contract_coverage_report() {
    let root = repo_root();
    let payload = load_json(&root.join("docs/_internal/generated/docs-contract-coverage.json"));
    assert_eq!(payload["kind"].as_str(), Some("docs_contract_coverage_v1"));
    let artifacts = payload["generated_artifacts"]
        .as_array()
        .expect("generated artifacts");
    for rel in [
        "docs/_internal/generated/topic-index.json",
        "docs/_internal/generated/search-index.json",
        "docs/_internal/generated/sitemap.json",
        "docs/_internal/generated/breadcrumbs.json",
        "docs/_internal/generated/docs-dependency-graph.json",
        "docs/_internal/generated/docs-quality-dashboard.json",
        "docs/_internal/generated/docs-contract-coverage.json",
        "docs/_internal/generated/concept-registry.json",
    ] {
        assert!(
            artifacts.iter().any(|row| row.as_str() == Some(rel)),
            "docs contract coverage must include `{rel}`"
        );
    }
}

#[test]
fn docs_navigation_policy_is_single_sourced_and_deterministic() {
    let root = repo_root();
    let mkdocs = read(&root.join("mkdocs.yml"));
    let top_level = parse_mkdocs_top_level_nav(&root);
    assert_eq!(
        top_level.first().map(String::as_str),
        Some("Home"),
        "mkdocs top-level navigation must start at Home"
    );
    assert!(
        mkdocs.contains("strict: true"),
        "mkdocs.yml must keep warnings-as-errors enabled via strict: true"
    );
}

#[test]
fn docs_workflow_runs_strict_build_and_offline_validation_lane() {
    let root = repo_root();
    let workflow = read(&root.join(".github/workflows/docs-only.yml"));
    assert!(
        workflow.contains("mkdocs build --strict"),
        "docs-only workflow must run mkdocs build in strict mode"
    );
    assert!(
        workflow.contains("docs validate --format json"),
        "docs-only workflow must run the docs validation lane"
    );
    assert!(
        !workflow.contains("docs validate --allow-network"),
        "docs validation lane must remain offline by default in CI"
    );
}
