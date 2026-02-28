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
        "policy",
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
