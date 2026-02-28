fn read_markdown_allowlist(root: &Path) -> Result<BTreeSet<String>, String> {
    let Some(value) = read_json(&root.join("ops/inventory/markdown-allowlist.json")) else {
        return Err("ops/inventory/markdown-allowlist.json is missing or invalid".to_string());
    };
    Ok(value
        .get("allowed_markdown")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default())
}

fn read_markdown_denylist(root: &Path) -> Result<BTreeSet<String>, String> {
    let Some(value) = read_json(&root.join("ops/inventory/deleted-markdown-denylist.json")) else {
        return Err("ops/inventory/deleted-markdown-denylist.json is missing or invalid".to_string());
    };
    Ok(value
        .get("paths")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default())
}

fn test_ops_root_011_markdown_allowlist_only(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-011";
    let test_id = "ops.root.markdown_allowlist_only";
    let allowlist = match read_markdown_allowlist(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        if path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !allowlist.contains(&rel) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops markdown file is not in markdown allowlist",
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

fn test_ops_root_012_single_readme_per_pillar(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-012";
    let test_id = "ops.root.single_readme_per_pillar";
    let pillars = match read_pillars_doc(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let mut violations = Vec::new();
    for pillar in pillars.pillars {
        if pillar.id == "root-surface" {
            continue;
        }
        let dir = ctx.repo_root.join(format!("ops/{}", pillar.id));
        let readme = dir.join("README.md");
        if !readme.is_file() {
            violations.push(violation(
                contract_id,
                test_id,
                "pillar directory must have exactly one README.md at root",
                Some(format!("ops/{}/README.md", pillar.id)),
            ));
        }
        let mut pillar_files = Vec::new();
        walk_files(&dir, &mut pillar_files);
        let readme_count = pillar_files
            .iter()
            .filter(|p| p.file_name().and_then(|v| v.to_str()) == Some("README.md"))
            .count();
        if readme_count != 1 {
            violations.push(violation(
                contract_id,
                test_id,
                "pillar must contain exactly one README.md file",
                Some(format!("ops/{}", pillar.id)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_013_markdown_allowlist_file_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-013";
    let test_id = "ops.root.markdown_allowlist_file_valid";
    let allowlist = match read_markdown_allowlist(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "markdown allowlist file must exist and be valid json",
                Some("ops/inventory/markdown-allowlist.json".to_string()),
            )]);
        }
    };
    if allowlist.is_empty() {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "markdown allowlist must not be empty",
            Some("ops/inventory/markdown-allowlist.json".to_string()),
        )])
    } else {
        TestResult::Pass
    }
}

fn test_ops_root_014_no_procedure_docs_in_ops(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-014";
    let test_id = "ops.root.no_procedure_docs_in_ops";
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        if path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let rel = rel_to_root(&path, &ctx.repo_root);
        let lower = rel.to_ascii_lowercase();
        if lower.contains("workflow")
            || lower.contains("procedure")
            || lower.contains("runbook")
            || lower.contains("policy")
        {
            violations.push(violation(
                contract_id,
                test_id,
                "workflow/procedure/policy markdown artifacts must not live under ops/",
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

fn test_ops_root_015_no_extra_pillar_markdown(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-015";
    let test_id = "ops.root.no_extra_pillar_markdown";
    let allowlist = match read_markdown_allowlist(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        if path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !allowlist.contains(&rel) {
            violations.push(violation(
                contract_id,
                test_id,
                "pillar markdown not allowed by ops markdown contract",
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

fn test_ops_root_016_deleted_markdown_denylist(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-016";
    let test_id = "ops.root.deleted_markdown_denylist";
    let denylist = match read_markdown_denylist(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "deleted markdown denylist must exist and be valid json",
                Some("ops/inventory/deleted-markdown-denylist.json".to_string()),
            )]);
        }
    };
    let mut violations = Vec::new();
    for rel in denylist {
        let path = ctx.repo_root.join(&rel);
        if path.exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "historically deleted markdown path must not be reintroduced",
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

fn test_ops_schema_001_parseable_documents(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-001";
    let test_id = "ops.schema.parseable_documents";
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.starts_with("ops/k8s/charts/") && rel.contains("/templates/") {
            continue;
        }
        if rel.ends_with(".json") {
            if read_json(&path).is_none() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "json document under ops must be parseable",
                    Some(rel),
                ));
            }
            continue;
        }
        if rel.ends_with(".yaml") || rel.ends_with(".yml") {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text.contains("{{") || text.contains("{%") {
                continue;
            }
            let mut parsed_any = false;
            let mut parsed_all = true;
            for doc in serde_yaml::Deserializer::from_str(&text) {
                parsed_any = true;
                if serde_yaml::Value::deserialize(doc).is_err() {
                    parsed_all = false;
                    break;
                }
            }
            if !parsed_any || !parsed_all {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "yaml document under ops must be parseable",
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

fn test_ops_schema_002_schema_index_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-002";
    let test_id = "ops.schema.index_complete";
    let index_path = ctx.repo_root.join("ops/schema/generated/schema-index.json");
    let Some(index) = read_json(&index_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "schema index must be parseable",
            Some("ops/schema/generated/schema-index.json".to_string()),
        )]);
    };
    let mut indexed = BTreeSet::new();
    if let Some(files) = index.get("files").and_then(|v| v.as_array()) {
        for item in files {
            if let Some(path) = item.as_str() {
                indexed.insert(path.to_string());
            }
        }
    }
    let mut actual = BTreeSet::new();
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.ends_with(".schema.json") {
            actual.insert(rel);
        }
    }
    let mut violations = Vec::new();
    for path in actual.difference(&indexed) {
        violations.push(violation(
            contract_id,
            test_id,
            "schema file missing from generated schema index",
            Some(path.clone()),
        ));
    }
    for path in indexed.difference(&actual) {
        violations.push(violation(
            contract_id,
            test_id,
            "schema index references schema file that is missing on disk",
            Some(path.clone()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_schema_003_no_unversioned_schemas(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-003";
    let test_id = "ops.schema.no_unversioned";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.starts_with("ops/schema/generated/") {
            continue;
        }
        if rel == "ops/schema/report/schema.json" {
            continue;
        }
        if rel.ends_with(".json") && !rel.ends_with(".schema.json") {
            violations.push(violation(
                contract_id,
                test_id,
                "schema source files must use .schema.json naming",
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

fn test_ops_schema_004_budget_policy(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-004";
    let test_id = "ops.schema.budget_policy";
    let budgets: BTreeMap<&str, usize> = BTreeMap::from([
        ("configs", 5),
        ("datasets", 20),
        ("e2e", 12),
        ("env", 5),
        ("inventory", 31),
        ("k8s", 12),
        ("load", 15),
        ("meta", 20),
        ("observe", 15),
        ("report", 25),
        ("stack", 12),
    ]);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".schema.json") {
            continue;
        }
        let parts = rel.split('/').collect::<Vec<_>>();
        if parts.len() < 4 {
            continue;
        }
        let domain = parts[2].to_string();
        *counts.entry(domain).or_insert(0) += 1;
    }
    let mut violations = Vec::new();
    for (domain, count) in counts {
        if let Some(limit) = budgets.get(domain.as_str()) {
            if count > *limit {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "schema count exceeds per-domain budget",
                    Some(format!("ops/schema/{domain}")),
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
fn test_ops_schema_005_evolution_lock(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-005";
    let test_id = "ops.schema.evolution_lock";
    let lock_path = ctx
        .repo_root
        .join("ops/schema/generated/compatibility-lock.json");
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "schema compatibility lock must be valid json",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let targets = lock.get("targets").and_then(|v| v.as_array());
    if targets.is_none_or(|arr| arr.is_empty()) {
        violations.push(violation(
            contract_id,
            test_id,
            "compatibility lock must contain non-empty targets",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        ));
    }
    if let Some(targets) = targets {
        for target in targets {
            let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str()) else {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "compatibility lock target missing schema_path",
                    Some("ops/schema/generated/compatibility-lock.json".to_string()),
                ));
                continue;
            };
            if !ctx.repo_root.join(schema_path).exists() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "compatibility lock target references missing schema path",
                    Some(schema_path.to_string()),
                ));
            }
            let required = target.get("required_fields").and_then(|v| v.as_array());
            if required.is_none_or(|r| r.is_empty()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "compatibility lock target requires non-empty required_fields",
                    Some(schema_path.to_string()),
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

