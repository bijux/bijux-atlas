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
    let contract_id = "OPS-DATASET-003";
    let test_id = "ops.dataset.no_fixture_drift_without_promotion_record";
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
    let contract_id = "OPS-DATASET-004";
    let test_id = "ops.dataset.release_diff_fixtures_deterministic";
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
