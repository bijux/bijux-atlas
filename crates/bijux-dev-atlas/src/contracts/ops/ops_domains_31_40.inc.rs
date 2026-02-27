fn test_ops_root_001_allowed_surface(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-001";
    let test_id = "ops.root.allowed_surface";
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops root directory is missing",
            Some("ops".to_string()),
        )]);
    };
    let allowed_files = BTreeSet::from(["README.md", "CONTRACT.md"]);
    let allowed_dirs = BTreeSet::from([
        "_generated",
        "_generated.example",
        "datasets",
        "e2e",
        "env",
        "inventory",
        "k8s",
        "load",
        "observe",
        "report",
        "schema",
        "stack",
    ]);
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if path.is_file() && !allowed_files.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops root contains unexpected file",
                Some(format!("ops/{name}")),
            ));
        } else if path.is_dir() && !allowed_dirs.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops root contains unexpected directory",
                Some(format!("ops/{name}")),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_002_forbid_extra_root_markdown(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-002";
    let test_id = "ops.root.forbid_extra_markdown";
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = fs::read_dir(&root) else {
        return TestResult::Error("ops root missing".to_string());
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name.ends_with(".md") && name != "README.md" && name != "CONTRACT.md" {
            violations.push(violation(
                contract_id,
                test_id,
                "only README.md and CONTRACT.md are allowed markdown files at ops root",
                Some(format!("ops/{name}")),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_003_no_shell_script_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-003";
    let test_id = "ops.root.no_shell_script_files";
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.ends_with(".sh") || rel.ends_with(".bash") {
            violations.push(violation(
                contract_id,
                test_id,
                "ops tree must not contain shell script files",
                Some(rel),
            ));
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if content.starts_with("#!/bin/bash") || content.starts_with("#!/usr/bin/env bash") {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "ops file embeds bash shebang",
                    Some(rel_to_root(&path, &ctx.repo_root)),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_004_max_directory_depth(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-004";
    let test_id = "ops.root.max_directory_depth";
    let max_depth = 12usize;
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let Ok(rel) = path.strip_prefix(ops_root(&ctx.repo_root)) else {
            continue;
        };
        let depth = rel.components().count();
        if depth > max_depth {
            violations.push(violation(
                contract_id,
                test_id,
                "ops file path exceeds configured maximum directory depth",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_005_filename_policy(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-005";
    let test_id = "ops.root.filename_policy";
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let uppercase_allow =
        BTreeSet::from(["README.md", "CONTRACT.md", "Chart.yaml", "ALLOWLIST.json"]);
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name.contains(' ') {
            violations.push(violation(
                contract_id,
                test_id,
                "ops filenames must not contain spaces",
                Some(rel.clone()),
            ));
        }
        if !uppercase_allow.contains(name) && name.chars().any(|ch| ch.is_ascii_uppercase()) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops filenames must be lowercase unless allowlisted",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_006_generated_gitignore_policy(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-006";
    let test_id = "ops.root.generated_gitignore_policy";
    let gitignore_path = ctx.repo_root.join(".gitignore");
    let Ok(gitignore) = fs::read_to_string(&gitignore_path) else {
        return TestResult::Error("unable to read .gitignore".to_string());
    };
    let has_generated_ignore = gitignore.lines().any(|line| line.trim() == "ops/_generated/**");
    let has_gitkeep_exception = gitignore
        .lines()
        .any(|line| line.trim() == "!ops/_generated/.gitkeep");
    if has_generated_ignore && has_gitkeep_exception {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops/_generated must be ignored with a .gitkeep exception in .gitignore",
            Some(".gitignore".to_string()),
        )])
    }
}

fn test_ops_root_007_generated_example_secret_guard(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-007";
    let test_id = "ops.root.generated_example_secret_guard";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/_generated.example"), &mut files);
    files.sort();
    let blocked_tokens = [
        "BEGIN PRIVATE KEY",
        "AWS_SECRET_ACCESS_KEY",
        "password=",
        "token=",
        "secret=",
    ];
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if let Ok(content) = fs::read_to_string(&path) {
            if blocked_tokens.iter().any(|token| content.contains(token)) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "generated example artifact contains a blocked secret-like token",
                    Some(rel.clone()),
                ));
            }
            if rel.ends_with(".json") && serde_json::from_str::<Value>(&content).is_err() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "generated example json file is invalid",
                    Some(rel),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_008_placeholder_dirs_allowlist(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-008";
    let test_id = "ops.root.placeholder_dirs_allowlist";
    let allow_path = ctx.repo_root.join("ops/inventory/placeholder-dirs.json");
    let Some(allow) = read_json(&allow_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "placeholder-dirs allowlist must be valid json",
            Some("ops/inventory/placeholder-dirs.json".to_string()),
        )]);
    };
    let allowed: BTreeSet<String> = allow
        .get("placeholder_entries")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|v| {
                    v.get("path")
                        .and_then(|path| path.as_str().map(ToOwned::to_owned))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut violations = Vec::new();
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    for path in files {
        if path.file_name().and_then(|v| v.to_str()) != Some(".gitkeep") {
            continue;
        }
        let rel = rel_to_root(&path, &ctx.repo_root);
        let parent_rel = path
            .parent()
            .map(|p| rel_to_root(p, &ctx.repo_root))
            .unwrap_or_else(|| "ops".to_string());
        if !allowed.contains(&parent_rel) {
            violations.push(violation(
                contract_id,
                test_id,
                "placeholder directory is not listed in allowlist",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_009_inventory_coverage_for_policy_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-009";
    let test_id = "ops.root.policy_files_inventory_coverage";
    let (authoritative, contracts_map_items) = inventory_paths(&ctx.repo_root);
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let coverage_allowlist = BTreeSet::from([
        "ops/datasets/promotion-rules.json",
        "ops/datasets/qc-metadata.json",
        "ops/datasets/real-datasets.json",
        "ops/e2e/taxonomy.json",
        "ops/k8s/install-matrix.json",
        "ops/observe/alert-catalog.json",
        "ops/observe/readiness.json",
        "ops/observe/slo-definitions.json",
        "ops/observe/telemetry-drills.json",
        "ops/report/evidence-levels.json",
        "ops/stack/profiles.json",
    ]);
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        let ops_depth = path
            .strip_prefix(ops_root(&ctx.repo_root))
            .ok()
            .map(|rel| rel.components().count())
            .unwrap_or(0);
        if rel.starts_with("ops/_generated/")
            || rel.starts_with("ops/_generated.example/")
            || rel.starts_with("ops/schema/")
            || rel.ends_with(".md")
        {
            continue;
        }
        if ops_depth > 2 {
            continue;
        }
        if !(rel.ends_with(".json")
            || rel.ends_with(".yaml")
            || rel.ends_with(".yml")
            || rel.ends_with(".toml"))
        {
            continue;
        }
        if !authoritative.contains(&rel)
            && !contracts_map_items.contains(&rel)
            && !rel.starts_with("ops/inventory/")
        {
            if coverage_allowlist.contains(rel.as_str()) {
                continue;
            }
            if rel.ends_with(".json") {
                if let Some(value) = read_json(&path) {
                    if value
                        .get("$schema")
                        .and_then(|v| v.as_str())
                        .is_some_and(|schema| !schema.is_empty())
                    {
                        continue;
                    }
                }
            }
            if rel.ends_with(".toml")
                && (rel == "ops/load/load.toml" || rel == "ops/stack/stack.toml")
            {
                continue;
            }
            violations.push(violation(
                contract_id,
                test_id,
                "ops policy/config artifact must be covered by inventory registry",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_dataset_003_no_fixture_drift_without_promotion_record(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-003";
    let test_id = "ops.datasets.no_fixture_drift_without_promotion_record";
    let index_path = ctx.repo_root.join("ops/datasets/generated/dataset-index.json");
    let Some(index) = read_json(&index_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "dataset index must be valid json",
            Some("ops/datasets/generated/dataset-index.json".to_string()),
        )]);
    };

    let missing_count = index
        .get("missing_dataset_ids")
        .and_then(|v| v.as_array())
        .map_or(0, |items| items.len());
    let stale_count = index
        .get("stale_dataset_ids")
        .and_then(|v| v.as_array())
        .map_or(0, |items| items.len());

    if missing_count + stale_count == 0 {
        return TestResult::Pass;
    }

    let promotion_rules_path = ctx.repo_root.join("ops/datasets/promotion-rules.json");
    let Some(promotion_rules) = read_json(&promotion_rules_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "promotion-rules must be valid json when dataset drift exists",
            Some("ops/datasets/promotion-rules.json".to_string()),
        )]);
    };
    let has_promotion_rule = promotion_rules
        .get("rules")
        .and_then(|v| v.as_array())
        .is_some_and(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .any(|rule| rule.contains("promotion"))
        });
    if !has_promotion_rule {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "fixture drift requires explicit promotion rule coverage",
            Some("ops/datasets/promotion-rules.json".to_string()),
        )]);
    }
    TestResult::Pass
}

fn test_ops_dataset_004_release_diff_fixtures_are_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-004";
    let test_id = "ops.datasets.release_diff_fixtures_deterministic";
    let lock_path = ctx
        .repo_root
        .join("ops/datasets/fixtures/release-diff/v1/manifest.lock");
    let Ok(lock_text) = std::fs::read_to_string(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "release-diff fixture manifest.lock is required",
            Some("ops/datasets/fixtures/release-diff/v1/manifest.lock".to_string()),
        )]);
    };
    let has_sha256 = lock_text.lines().any(|line| line.starts_with("sha256="));
    let has_archive = lock_text.lines().any(|line| line.starts_with("archive="));
    let mut violations = Vec::new();
    if !has_sha256 || !has_archive {
        violations.push(violation(
            contract_id,
            test_id,
            "release-diff manifest.lock must include sha256 and archive pins",
            Some("ops/datasets/fixtures/release-diff/v1/manifest.lock".to_string()),
        ));
    }

    for rel in [
        "ops/datasets/fixtures/release-diff/v1/release-diff-queries.v1.json",
        "ops/datasets/fixtures/release-diff/v1/release-diff-responses.v1.json",
    ] {
        let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "release-diff golden files must be valid json",
                Some(rel.to_string()),
            ));
            continue;
        };
        if !value.is_array() {
            violations.push(violation(
                contract_id,
                test_id,
                "release-diff golden files must be top-level arrays",
                Some(rel.to_string()),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_001_suites_reference_real_scenarios(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-001";
    let test_id = "ops.e2e.suites_reference_real_scenarios";
    let suites_path = ctx.repo_root.join("ops/e2e/suites/suites.json");
    let Some(suites) = read_json(&suites_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e suites must be valid json",
            Some("ops/e2e/suites/suites.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let Some(items) = suites.get("suites").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e suites file must contain suites array",
            Some("ops/e2e/suites/suites.json".to_string()),
        )]);
    };
    for suite in items {
        if let Some(scenarios) = suite.get("scenarios").and_then(|v| v.as_array()) {
            for scenario in scenarios {
                let id = scenario.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let runner = scenario
                    .get("runner")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if id.is_empty() || !seen_ids.insert(id.to_string()) {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "scenario id must be non-empty and globally unique in suites.json",
                        Some("ops/e2e/suites/suites.json".to_string()),
                    ));
                }
                if runner.is_empty() {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "scenario runner must be non-empty",
                        Some("ops/e2e/suites/suites.json".to_string()),
                    ));
                }
                if runner.contains("python3 ") && !runner.contains(".py") {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "python runner command must reference a python source path",
                        Some("ops/e2e/suites/suites.json".to_string()),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_002_smoke_manifest_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-002";
    let test_id = "ops.e2e.smoke_manifest_valid";
    let manifest_rel = "ops/e2e/manifests/smoke.manifest.json";
    let Some(manifest) = read_json(&ctx.repo_root.join(manifest_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "smoke manifest must be valid json",
            Some(manifest_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if manifest
        .get("schema_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .is_empty()
    {
        violations.push(violation(
            contract_id,
            test_id,
            "smoke manifest requires schema_name",
            Some(manifest_rel.to_string()),
        ));
    }
    let queries_lock = manifest
        .get("queries_lock")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if queries_lock.is_empty() || !ctx.repo_root.join(queries_lock).exists() {
        violations.push(violation(
            contract_id,
            test_id,
            "smoke manifest queries_lock must reference existing file",
            Some(manifest_rel.to_string()),
        ));
    } else if let Ok(contents) = fs::read_to_string(ctx.repo_root.join(queries_lock)) {
        let entries = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if entries.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "smoke manifest queries_lock must contain at least one query",
                Some(queries_lock.to_string()),
            ));
        } else {
            let unique = entries.iter().cloned().collect::<BTreeSet<_>>();
            if unique.len() != entries.len() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "smoke manifest queries_lock entries must be unique",
                    Some(queries_lock.to_string()),
                ));
            }
            let mut sorted = entries.clone();
            sorted.sort();
            if entries != sorted {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "smoke manifest queries_lock entries must stay lexicographically sorted",
                    Some(queries_lock.to_string()),
                ));
            }
        }
    }
    if manifest
        .get("steps")
        .and_then(|v| v.as_array())
        .is_none_or(|steps| steps.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "smoke manifest must include at least one step",
            Some(manifest_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_003_fixtures_lock_matches_allowlist(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-003";
    let test_id = "ops.e2e.fixtures_lock_matches_allowlist";
    let lock_rel = "ops/e2e/fixtures/fixtures.lock";
    let allow_rel = "ops/e2e/fixtures/allowlist.json";
    let lock_path = ctx.repo_root.join(lock_rel);
    let allow_path = ctx.repo_root.join(allow_rel);

    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "fixtures.lock must be valid json",
            Some(lock_rel.to_string()),
        )]);
    };
    let Some(allow) = read_json(&allow_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "allowlist.json must be valid json",
            Some(allow_rel.to_string()),
        )]);
    };

    let mut violations = Vec::new();
    let expected_sha = file_sha256(&allow_path);
    let actual_sha = lock
        .get("allowlist_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if expected_sha.as_deref().is_some_and(|sha| sha != actual_sha) {
        violations.push(violation(
            contract_id,
            test_id,
            "fixtures.lock allowlist_sha256 must match allowlist.json digest",
            Some(lock_rel.to_string()),
        ));
    }

    let mut allowed_paths = BTreeSet::new();
    if let Some(paths) = allow.get("allowed_paths").and_then(|v| v.as_array()) {
        for path in paths {
            if let Some(path) = path.as_str() {
                allowed_paths.insert(path.to_string());
            }
        }
    }
    let fixtures_dir = ctx.repo_root.join("ops/e2e/fixtures");
    if let Ok(entries) = std::fs::read_dir(fixtures_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let rel = rel_to_root(&path, &ctx.repo_root);
            if !allowed_paths.contains(&rel) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "fixture file is not declared in allowlist",
                    Some(rel),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_004_realdata_snapshots_schema_valid_and_pinned(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-004";
    let test_id = "ops.e2e.realdata_snapshots_schema_valid_and_pinned";
    let snapshots_root = ctx.repo_root.join("ops/e2e/realdata/snapshots");
    let canonical_rel = "ops/e2e/realdata/canonical_queries.json";
    let mut files = Vec::new();
    walk_files(&snapshots_root, &mut files);
    files.sort();
    if files.is_empty() {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "at least one realdata snapshot is required",
            Some("ops/e2e/realdata/snapshots".to_string()),
        )]);
    }
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".json") {
            continue;
        }
        let Some(value) = read_json(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "snapshot file must be valid json",
                Some(rel),
            ));
            continue;
        };
        let generated_from = value
            .get("generated_from")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if generated_from != canonical_rel {
            violations.push(violation(
                contract_id,
                test_id,
                "snapshot generated_from must pin canonical query source",
                Some(rel.clone()),
            ));
        }
        if value.get("entries").and_then(|v| v.as_array()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "snapshot must expose entries array",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_005_taxonomy_covers_scenarios(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-005";
    let test_id = "ops.e2e.taxonomy_covers_scenarios";
    let taxonomy_rel = "ops/e2e/taxonomy.json";
    let scenarios_rel = "ops/e2e/scenarios/scenarios.json";
    let summary_rel = "ops/e2e/generated/e2e-summary.json";
    let Some(taxonomy) = read_json(&ctx.repo_root.join(taxonomy_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "taxonomy.json must be valid json",
            Some(taxonomy_rel.to_string()),
        )]);
    };
    let Some(scenarios) = read_json(&ctx.repo_root.join(scenarios_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "scenarios.json must be valid json",
            Some(scenarios_rel.to_string()),
        )]);
    };
    let Some(summary) = read_json(&ctx.repo_root.join(summary_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e summary must be valid json",
            Some(summary_rel.to_string()),
        )]);
    };

    let mut violations = Vec::new();
    if taxonomy.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "taxonomy schema_version must be 1",
            Some(taxonomy_rel.to_string()),
        ));
    }

    let mut category_ids = BTreeSet::new();
    for category in taxonomy
        .get("categories")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(id) = category.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        if !category_ids.insert(id.to_string()) {
            violations.push(violation(
                contract_id,
                test_id,
                "taxonomy categories must be unique",
                Some(taxonomy_rel.to_string()),
            ));
        }
    }
    let required_categories = BTreeSet::from([
        "smoke".to_string(),
        "kubernetes".to_string(),
        "realdata".to_string(),
        "performance".to_string(),
    ]);
    if category_ids != required_categories {
        violations.push(violation(
            contract_id,
            test_id,
            "taxonomy categories must exactly match required category set",
            Some(taxonomy_rel.to_string()),
        ));
    }

    let scenario_category = BTreeMap::from([
        ("smoke", "smoke"),
        ("k8s-suite", "kubernetes"),
        ("realdata", "realdata"),
        ("perf-e2e", "performance"),
    ]);
    let mut scenario_ids = BTreeSet::new();
    for scenario in scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(id) = scenario.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        scenario_ids.insert(id.to_string());
        let Some(category) = scenario_category.get(id) else {
            violations.push(violation(
                contract_id,
                test_id,
                "scenario id is missing category mapping",
                Some(scenarios_rel.to_string()),
            ));
            continue;
        };
        if !category_ids.contains(*category) {
            violations.push(violation(
                contract_id,
                test_id,
                "scenario references taxonomy category that does not exist",
                Some(scenarios_rel.to_string()),
            ));
        }
    }
    let summary_scenarios: BTreeSet<String> = summary
        .get("scenario_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if summary_scenarios != scenario_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "summary scenario_ids must match scenarios registry ids",
            Some(summary_rel.to_string()),
        ));
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_006_reproducibility_policy_enforced(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-006";
    let test_id = "ops.e2e.reproducibility_policy_enforced";
    let policy_rel = "ops/e2e/reproducibility-policy.json";
    let summary_rel = "ops/e2e/generated/e2e-summary.json";
    let Some(policy) = read_json(&ctx.repo_root.join(policy_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "reproducibility-policy.json must be valid json",
            Some(policy_rel.to_string()),
        )]);
    };
    let Some(summary) = read_json(&ctx.repo_root.join(summary_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e summary must be valid json",
            Some(summary_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if policy.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy schema_version must be 1",
            Some(policy_rel.to_string()),
        ));
    }
    if policy.get("ordering").and_then(|v| v.as_str()) != Some("stable") {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy ordering must be stable",
            Some(policy_rel.to_string()),
        ));
    }
    let required_checks = BTreeSet::from([
        "scenario-order-deterministic".to_string(),
        "fixture-allowlist-enforced".to_string(),
        "expectation-catalog-complete".to_string(),
    ]);
    let checks: BTreeSet<String> = policy
        .get("required_checks")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if checks != required_checks {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy required_checks must match canonical checks",
            Some(policy_rel.to_string()),
        ));
    }
    let seed_source = policy
        .get("seed_source")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if seed_source.is_empty() || !ctx.repo_root.join(seed_source).exists() {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy seed_source must reference an existing file",
            Some(policy_rel.to_string()),
        ));
    }
    let suite_ids: Vec<String> = summary
        .get("suite_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let mut sorted_suite_ids = suite_ids.clone();
    sorted_suite_ids.sort();
    if suite_ids != sorted_suite_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_ids must be deterministically sorted",
            Some(summary_rel.to_string()),
        ));
    }
    let scenario_ids: Vec<String> = summary
        .get("scenario_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let mut sorted_scenario_ids = scenario_ids.clone();
    sorted_scenario_ids.sort();
    if scenario_ids != sorted_scenario_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_ids must be deterministically sorted",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("suite_count").and_then(|v| v.as_u64()) != Some(suite_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_count must match suite_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("scenario_count").and_then(|v| v.as_u64()) != Some(scenario_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_count must match scenario_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_007_coverage_matrix_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-007";
    let test_id = "ops.e2e.coverage_matrix_deterministic";
    let matrix_rel = "ops/e2e/generated/coverage-matrix.json";
    let scenarios_rel = "ops/e2e/scenarios/scenarios.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "coverage-matrix.json must be valid json",
            Some(matrix_rel.to_string()),
        )]);
    };
    let Some(scenarios) = read_json(&ctx.repo_root.join(scenarios_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "scenarios.json must be valid json",
            Some(scenarios_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if matrix.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix schema_version must be 1",
            Some(matrix_rel.to_string()),
        ));
    }
    if matrix
        .get("missing_domains")
        .and_then(|v| v.as_array())
        .is_some_and(|v| !v.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix missing_domains must be empty",
            Some(matrix_rel.to_string()),
        ));
    }
    let mut scenario_ids_from_rows = Vec::new();
    for row in matrix
        .get("rows")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(scenario_id) = row.get("scenario_id").and_then(|v| v.as_str()) else {
            continue;
        };
        scenario_ids_from_rows.push(scenario_id.to_string());
        let covers: Vec<String> = row
            .get("covers")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let mut covers_sorted = covers.clone();
        covers_sorted.sort();
        covers_sorted.dedup();
        if covers != covers_sorted {
            violations.push(violation(
                contract_id,
                test_id,
                "coverage matrix covers entries must be unique and sorted",
                Some(matrix_rel.to_string()),
            ));
        }
    }
    let mut sorted_rows = scenario_ids_from_rows.clone();
    sorted_rows.sort();
    sorted_rows.dedup();
    if scenario_ids_from_rows != sorted_rows {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix rows must be unique and sorted by scenario_id",
            Some(matrix_rel.to_string()),
        ));
    }
    let scenario_ids: BTreeSet<String> = scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
        .collect();
    let covered_ids: BTreeSet<String> = scenario_ids_from_rows.into_iter().collect();
    if scenario_ids != covered_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix rows must match scenarios registry ids",
            Some(matrix_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_008_realdata_registry_and_snapshots_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-008";
    let test_id = "ops.e2e.realdata_registry_and_snapshots_valid";
    let scenarios_rel = "ops/e2e/realdata/scenarios.json";
    let snapshots_rel = "ops/e2e/realdata/snapshots";
    let Some(scenarios) = read_json(&ctx.repo_root.join(scenarios_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "realdata scenarios.json must be valid json",
            Some(scenarios_rel.to_string()),
        )]);
    };
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join(snapshots_rel), &mut files);
    files.sort();
    let mut violations = Vec::new();
    if scenarios.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "realdata scenarios schema_version must be 1",
            Some(scenarios_rel.to_string()),
        ));
    }
    let scenario_rows = scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if scenario_rows.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "realdata scenarios registry must include at least one scenario",
            Some(scenarios_rel.to_string()),
        ));
    }
    for row in scenario_rows {
        let id = row.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let runner = row.get("runner").and_then(|v| v.as_str()).unwrap_or_default();
        let action_id = row.get("action_id").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() || runner.is_empty() || action_id.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata scenario rows must include id runner and action_id",
                Some(scenarios_rel.to_string()),
            ));
            continue;
        }
        if !runner.ends_with(".py")
            || !runner.starts_with("crates/bijux-dev-atlas/src/bijux-dev-atlas/commands/ops/e2e/")
        {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata scenario runner must use canonical ops/e2e python path",
                Some(scenarios_rel.to_string()),
            ));
        }
    }
    if files.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "realdata snapshots directory must include at least one snapshot",
            Some(snapshots_rel.to_string()),
        ));
    }
    for path in files {
        if !path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let rel = rel_to_root(&path, &ctx.repo_root);
        let Some(snapshot) = read_json(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata snapshot must be valid json",
                Some(rel),
            ));
            continue;
        };
        if snapshot.get("entries").and_then(|v| v.as_array()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata snapshot must include entries array",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_009_no_stray_e2e_artifacts(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-009";
    let test_id = "ops.e2e.no_stray_e2e_artifacts";
    let e2e_root = ctx.repo_root.join("ops/e2e");
    let Ok(entries) = fs::read_dir(&e2e_root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops/e2e directory must exist",
            Some("ops/e2e".to_string()),
        )]);
    };
    let allowed_entries = BTreeSet::from([
        "CONTRACT.md",
        "README.md",
        "datasets",
        "expectations",
        "fixtures",
        "generated",
        "manifests",
        "realdata",
        "reproducibility-policy.json",
        "scenarios",
        "smoke",
        "suites",
        "taxonomy.json",
    ]);
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !allowed_entries.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops/e2e contains artifact outside declared surface",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_010_summary_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-010";
    let test_id = "ops.e2e.summary_schema_valid";
    let summary_rel = "ops/e2e/generated/e2e-summary.json";
    let suites_rel = "ops/e2e/suites/suites.json";
    let scenarios_rel = "ops/e2e/scenarios/scenarios.json";
    let expectations_rel = "ops/e2e/expectations/expectations.json";
    let Some(summary) = read_json(&ctx.repo_root.join(summary_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e summary must be valid json",
            Some(summary_rel.to_string()),
        )]);
    };
    let Some(suites) = read_json(&ctx.repo_root.join(suites_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e suites registry must be valid json",
            Some(suites_rel.to_string()),
        )]);
    };
    let Some(scenarios) = read_json(&ctx.repo_root.join(scenarios_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e scenarios registry must be valid json",
            Some(scenarios_rel.to_string()),
        )]);
    };
    let Some(expectations) = read_json(&ctx.repo_root.join(expectations_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e expectations registry must be valid json",
            Some(expectations_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if summary.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary schema_version must be 1",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("status").and_then(|v| v.as_str()) != Some("stable") {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary status must be stable",
            Some(summary_rel.to_string()),
        ));
    }
    let suite_ids: Vec<String> = summary
        .get("suite_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let scenario_ids: Vec<String> = summary
        .get("scenario_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let mut sorted_suite_ids = suite_ids.clone();
    sorted_suite_ids.sort();
    if suite_ids != sorted_suite_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_ids must be sorted",
            Some(summary_rel.to_string()),
        ));
    }
    let mut sorted_scenario_ids = scenario_ids.clone();
    sorted_scenario_ids.sort();
    if scenario_ids != sorted_scenario_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_ids must be sorted",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("suite_count").and_then(|v| v.as_u64()) != Some(suite_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_count must equal suite_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("scenario_count").and_then(|v| v.as_u64()) != Some(scenario_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_count must equal scenario_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    let suites_declared: BTreeSet<String> = suites
        .get("suites")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
        .collect();
    let scenarios_declared: BTreeSet<String> = scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
        .collect();
    if suites_declared != suite_ids.iter().cloned().collect() {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_ids must match suites registry",
            Some(summary_rel.to_string()),
        ));
    }
    if scenarios_declared != scenario_ids.iter().cloned().collect() {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_ids must match scenarios registry",
            Some(summary_rel.to_string()),
        ));
    }
    let expectation_ids: BTreeSet<String> = expectations
        .get("expectations")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| {
            v.get("scenario_id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    if expectation_ids != scenarios_declared {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e expectations must cover every declared public scenario exactly once",
            Some(expectations_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_001_overlays_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-001";
    let test_id = "ops.env.overlays_schema_valid";
    let overlays = [
        "ops/env/base/overlay.json",
        "ops/env/ci/overlay.json",
        "ops/env/dev/overlay.json",
        "ops/env/prod/overlay.json",
    ];
    let mut violations = Vec::new();
    for rel in overlays {
        let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay must be valid json",
                Some(rel.to_string()),
            ));
            continue;
        };
        if value.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay schema_version must be 1",
                Some(rel.to_string()),
            ));
        }
        if value.get("environment").and_then(|v| v.as_str()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay must include environment",
                Some(rel.to_string()),
            ));
        }
        if !value.get("values").is_some_and(|v| v.is_object()) {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay must include object values map",
                Some(rel.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_002_env_profiles_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-002";
    let test_id = "ops.env.profiles_complete";
    let profiles = ["base", "ci", "dev", "prod"];
    let mut violations = Vec::new();
    for profile in profiles {
        let rel = format!("ops/env/{profile}/overlay.json");
        let path = ctx.repo_root.join(&rel);
        if !path.exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "required environment overlay is missing",
                Some(rel),
            ));
            continue;
        }
        let Some(value) = read_json(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "required environment overlay must be valid json",
                Some(rel),
            ));
            continue;
        };
        if value.get("environment").and_then(|v| v.as_str()) != Some(profile) {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay environment field must match profile directory",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_003_no_unknown_keys(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-003";
    let test_id = "ops.env.no_unknown_keys";
    let overlays = [
        "ops/env/base/overlay.json",
        "ops/env/ci/overlay.json",
        "ops/env/dev/overlay.json",
        "ops/env/prod/overlay.json",
    ];
    let allowed_top = BTreeSet::from(["schema_version", "environment", "values"]);
    let allowed_values = BTreeSet::from([
        "namespace",
        "cluster_profile",
        "allow_write",
        "allow_subprocess",
        "network_mode",
    ]);
    let mut violations = Vec::new();
    for rel in overlays {
        let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
            continue;
        };
        let Some(obj) = value.as_object() else {
            continue;
        };
        for key in obj.keys() {
            if !allowed_top.contains(key.as_str()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "overlay uses unknown top-level key",
                    Some(rel.to_string()),
                ));
            }
        }
        if let Some(values) = value.get("values").and_then(|v| v.as_object()) {
            for key in values.keys() {
                if !allowed_values.contains(key.as_str()) {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "overlay values uses unknown key",
                        Some(rel.to_string()),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_004_overlay_merge_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-004";
    let test_id = "ops.env.overlay_merge_deterministic";
    let base_rel = "ops/env/base/overlay.json";
    let profiles = ["dev", "ci", "prod"];
    let Some(base_overlay) = read_json(&ctx.repo_root.join(base_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "base overlay must be valid json",
            Some(base_rel.to_string()),
        )]);
    };
    let base_values = base_overlay
        .get("values")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut violations = Vec::new();
    for profile in profiles {
        let rel = format!("ops/env/{profile}/overlay.json");
        let Some(overlay) = read_json(&ctx.repo_root.join(&rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "profile overlay must be valid json",
                Some(rel),
            ));
            continue;
        };
        let Some(values) = overlay.get("values").and_then(|v| v.as_object()) else {
            violations.push(violation(
                contract_id,
                test_id,
                "profile overlay values must be object",
                Some(rel),
            ));
            continue;
        };
        let mut merged_a = base_values.clone();
        for (k, v) in values {
            merged_a.insert(k.clone(), v.clone());
        }
        let mut merged_b = base_values.clone();
        for (k, v) in values {
            merged_b.insert(k.clone(), v.clone());
        }
        if merged_a != merged_b {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay merge with same inputs must be deterministic",
                Some(format!("ops/env/{profile}/overlay.json")),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_005_prod_forbids_dev_toggles(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-005";
    let test_id = "ops.env.prod_forbids_dev_toggles";
    let rel = "ops/env/prod/overlay.json";
    let Some(prod) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "prod overlay must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let values = prod.get("values").and_then(|v| v.as_object());
    let allow_write = values
        .and_then(|v| v.get("allow_write"))
        .and_then(|v| v.as_bool());
    let allow_subprocess = values
        .and_then(|v| v.get("allow_subprocess"))
        .and_then(|v| v.as_bool());
    let network_mode = values
        .and_then(|v| v.get("network_mode"))
        .and_then(|v| v.as_str());
    if allow_write != Some(false) {
        violations.push(violation(
            contract_id,
            test_id,
            "prod overlay must set allow_write=false",
            Some(rel.to_string()),
        ));
    }
    if allow_subprocess != Some(false) {
        violations.push(violation(
            contract_id,
            test_id,
            "prod overlay must set allow_subprocess=false",
            Some(rel.to_string()),
        ));
    }
    if network_mode != Some("restricted") {
        violations.push(violation(
            contract_id,
            test_id,
            "prod overlay must use restricted network_mode",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_006_ci_restricts_effects(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-006";
    let test_id = "ops.env.ci_restricts_effects";
    let rel = "ops/env/ci/overlay.json";
    let Some(ci) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ci overlay must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let values = ci.get("values").and_then(|v| v.as_object());
    if values
        .and_then(|v| v.get("allow_subprocess"))
        .and_then(|v| v.as_bool())
        != Some(false)
    {
        violations.push(violation(
            contract_id,
            test_id,
            "ci overlay must set allow_subprocess=false",
            Some(rel.to_string()),
        ));
    }
    if values
        .and_then(|v| v.get("network_mode"))
        .and_then(|v| v.as_str())
        != Some("restricted")
    {
        violations.push(violation(
            contract_id,
            test_id,
            "ci overlay must use restricted network_mode",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_007_base_overlay_required_defaults(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-007";
    let test_id = "ops.env.base_overlay_required_defaults";
    let rel = "ops/env/base/overlay.json";
    let Some(base) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "base overlay must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let values = base.get("values").and_then(|v| v.as_object());
    for key in [
        "namespace",
        "cluster_profile",
        "allow_write",
        "allow_subprocess",
        "network_mode",
    ] {
        if values.and_then(|v| v.get(key)).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "base overlay is missing required default key",
                Some(rel.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_008_overlay_keys_stable(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-008";
    let test_id = "ops.env.overlay_keys_stable";
    let profiles = ["base", "dev", "ci", "prod"];
    let mut violations = Vec::new();
    let mut reference_keys: Option<BTreeSet<String>> = None;
    for profile in profiles {
        let rel = format!("ops/env/{profile}/overlay.json");
        let Some(overlay) = read_json(&ctx.repo_root.join(&rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay must be valid json",
                Some(rel),
            ));
            continue;
        };
        if overlay.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
            violations.push(violation(
                contract_id,
                test_id,
                "overlay schema_version must be 1",
                Some(rel.clone()),
            ));
        }
        let keys: BTreeSet<String> = overlay
            .get("values")
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();
        if let Some(reference) = &reference_keys {
            if &keys != reference {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "overlay values keys must be stable across all profiles",
                    Some(rel),
                ));
            }
        } else {
            reference_keys = Some(keys);
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_env_009_overlays_dir_no_stray_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ENV-009";
    let test_id = "ops.env.overlays_dir_no_stray_files";
    let overlays_dir = ctx.repo_root.join("ops/env/overlays");
    let Ok(entries) = fs::read_dir(&overlays_dir) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops/env/overlays directory must exist",
            Some("ops/env/overlays".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
        if name != ".gitkeep" {
            violations.push(violation(
                contract_id,
                test_id,
                "ops/env/overlays may only contain .gitkeep",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }
    let matrix_rel = "ops/env/portability-matrix.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        violations.push(violation(
            contract_id,
            test_id,
            "environment portability matrix must exist and be valid json",
            Some(matrix_rel.to_string()),
        ));
        return TestResult::Fail(violations);
    };
    let expected_envs = BTreeSet::from([
        "base".to_string(),
        "ci".to_string(),
        "dev".to_string(),
        "prod".to_string(),
    ]);
    let envs: BTreeSet<String> = matrix
        .get("environments")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if envs != expected_envs {
        violations.push(violation(
            contract_id,
            test_id,
            "portability matrix environments must include base/ci/dev/prod",
            Some(matrix_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_k8s_001_chart_renders_static(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-001";
    let test_id = "ops.k8s.chart_renders_static";
    let chart_root = ctx.repo_root.join("ops/k8s/charts/bijux-atlas");
    let required = [
        "ops/k8s/charts/bijux-atlas/Chart.yaml",
        "ops/k8s/charts/bijux-atlas/values.yaml",
        "ops/k8s/charts/bijux-atlas/values.schema.json",
    ];
    let mut violations = Vec::new();
    for rel in required {
        if !ctx.repo_root.join(rel).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "required chart source is missing",
                Some(rel.to_string()),
            ));
        }
    }
    let templates_dir = chart_root.join("templates");
    let mut template_count = 0usize;
    if let Ok(entries) = std::fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path
                .extension()
                .and_then(|v| v.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("tpl"))
            {
                template_count += 1;
            }
        }
    }
    if template_count == 0 {
        violations.push(violation(
            contract_id,
            test_id,
            "helm chart must include at least one template file",
            Some("ops/k8s/charts/bijux-atlas/templates".to_string()),
        ));
    }

    let Some(chart) = read_yaml_value(&ctx.repo_root.join("ops/k8s/charts/bijux-atlas/Chart.yaml")) else {
        violations.push(violation(
            contract_id,
            test_id,
            "Chart.yaml must be valid yaml",
            Some("ops/k8s/charts/bijux-atlas/Chart.yaml".to_string()),
        ));
        return TestResult::Fail(violations);
    };
    let chart_name = chart.get("name").and_then(|v| v.as_str()).unwrap_or("");
    if chart_name != "bijux-atlas" {
        violations.push(violation(
            contract_id,
            test_id,
            "Chart.yaml name must be bijux-atlas",
            Some("ops/k8s/charts/bijux-atlas/Chart.yaml".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn read_yaml_value(path: &Path) -> Option<serde_yaml::Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&text).ok()
}
