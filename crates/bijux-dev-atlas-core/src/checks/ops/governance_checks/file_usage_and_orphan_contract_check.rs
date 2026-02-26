pub(super) fn check_ops_file_usage_and_orphan_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let ledger_rel = Path::new("ops/_generated.example/ops-ledger.json");
    let ledger_text = fs::read_to_string(ctx.repo_root.join(ledger_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let ledger_json: serde_json::Value =
        serde_json::from_str(&ledger_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if ledger_json.get("schema_version").and_then(|v| v.as_i64()) != Some(1)
        || ledger_json.get("generated_by").is_none()
    {
        violations.push(violation(
            "OPS_LEDGER_METADATA_INVALID",
            "ops-ledger.json must include schema_version=1 and generated_by".to_string(),
            "regenerate ops/_generated.example/ops-ledger.json with required metadata",
            Some(ledger_rel),
        ));
    }
    let ledger_entries = ledger_json
        .get("entries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let ledger_paths = ledger_entries
        .iter()
        .filter_map(|entry| entry.get("path").and_then(|v| v.as_str()))
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    let usage_report_rel = Path::new("ops/_generated.example/file-usage-report.json");
    let usage_report_text = fs::read_to_string(ctx.repo_root.join(usage_report_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let usage_report_json: serde_json::Value = serde_json::from_str(&usage_report_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if usage_report_json.get("schema_version").and_then(|v| v.as_i64()) != Some(1)
        || usage_report_json.get("generated_by").is_none()
    {
        violations.push(violation(
            "OPS_FILE_USAGE_REPORT_METADATA_MISSING",
            "ops/_generated.example/file-usage-report.json must include schema_version=1 and generated_by"
                .to_string(),
            "add schema_version and generated_by to file usage report",
            Some(usage_report_rel),
        ));
    }
    let orphan_allowlist_prefixes = usage_report_json
        .get("orphan_allowlist_prefixes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for prefix in &orphan_allowlist_prefixes {
        let is_fixture_prefix = prefix.contains("/fixtures/");
        let is_curated_example_prefix = prefix.contains("/examples/");
        if !is_fixture_prefix && !is_curated_example_prefix {
            violations.push(violation(
                "OPS_FILE_USAGE_ALLOWLIST_SCOPE_INVALID",
                format!(
                    "orphan allowlist prefix `{prefix}` is outside fixture/example scope"
                ),
                "limit orphan allowlist prefixes to fixture payloads and curated examples",
                Some(usage_report_rel),
            ));
        }
    }

    let contracts_map_rel = Path::new("ops/inventory/contracts-map.json");
    let contracts_map_text = fs::read_to_string(ctx.repo_root.join(contracts_map_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_map_json: serde_json::Value = serde_json::from_str(&contracts_map_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let registry_inputs = contracts_map_json
        .get("items")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("path").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let authority_index_rel = Path::new("ops/inventory/authority-index.json");
    let authority_index_text = fs::read_to_string(ctx.repo_root.join(authority_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let authority_index_json: serde_json::Value = serde_json::from_str(&authority_index_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let authority_index_paths = authority_index_json
        .get("authoritative_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| entry.get("path").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let authority_ignored_paths = authority_index_json
        .get("ignored_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let authority_ignored_prefixes = authority_index_json
        .get("ignored_prefixes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let schema_index_rel = Path::new("ops/schema/generated/schema-index.json");
    let schema_index_text = fs::read_to_string(ctx.repo_root.join(schema_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let schema_index_json: serde_json::Value = serde_json::from_str(&schema_index_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let schema_files = schema_index_json
        .get("files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let mut required_file_refs = BTreeSet::new();
    for req in walk_files(&ctx.repo_root.join("ops")) {
        let rel = req.strip_prefix(ctx.repo_root).unwrap_or(req.as_path());
        if rel.file_name().and_then(|name| name.to_str()) != Some("REQUIRED_FILES.md") {
            continue;
        }
        let content = fs::read_to_string(&req).map_err(|err| CheckError::Failed(err.to_string()))?;
        let parsed = parse_required_files_markdown_yaml(&content, rel)?;
        for required in parsed.required_files {
            required_file_refs.insert(required.display().to_string());
        }
    }

    let mut docs_refs = BTreeSet::new();
    for root in ["docs", "ops"] {
        for doc in walk_files(&ctx.repo_root.join(root)) {
            if doc.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let text = fs::read_to_string(&doc).map_err(|err| CheckError::Failed(err.to_string()))?;
            docs_refs.extend(extract_ops_data_paths(&text));
        }
    }

    let mut computed_orphans = Vec::new();
    let mut ops_all_files = BTreeSet::new();
    let mut registry_count_by_domain = BTreeMap::<String, usize>::new();
    let mut generated_count_by_domain = BTreeMap::<String, usize>::new();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let rel_str = rel.display().to_string();
        ops_all_files.insert(rel_str.clone());
        let Some(ext) = rel.extension().and_then(|v| v.to_str()) else {
            continue;
        };
        if !matches!(ext, "json" | "yaml" | "yml" | "toml") {
            continue;
        }
        let domain = rel
            .components()
            .nth(1)
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("root")
            .to_string();
        let is_schema = rel_str.starts_with("ops/schema/");
        let is_generated =
            rel_str.contains("/generated/") || rel_str.starts_with("ops/_generated.example/");
        let is_registry_input = registry_inputs.contains(&rel_str)
            || rel_str.starts_with("ops/inventory/contracts/")
            || rel_str.starts_with("ops/inventory/policies/")
            || rel_str.starts_with("ops/k8s/charts/")
            || rel_str.starts_with("ops/k8s/values/")
            || rel_str.starts_with("ops/observe/pack/")
            || rel_str.starts_with("ops/observe/alerts/")
            || rel_str.starts_with("ops/observe/rules/")
            || rel_str.starts_with("ops/observe/dashboards/")
            || rel_str.starts_with("ops/load/compose/")
            || rel_str.starts_with("ops/load/baselines/")
            || rel_str.starts_with("ops/load/thresholds/")
            || rel_str.starts_with("ops/e2e/manifests/")
            || rel_str.starts_with("ops/stack/")
            || rel_str.contains("/contracts/")
            || rel_str.contains("/scenarios/")
            || rel_str.contains("/suites/");
        let is_docs_ref = docs_refs.contains(&rel_str);
        let is_required_ref = required_file_refs.contains(&rel_str);
        let is_schema_ref = schema_files.contains(&rel_str);
        let is_fixture_or_test = rel_str.contains("/fixtures/")
            || rel_str.contains("/tests/")
            || rel_str.contains("/goldens/")
            || rel_str.contains("/realdata/");
        if is_generated {
            *generated_count_by_domain.entry(domain).or_insert(0) += 1;
        } else {
            *registry_count_by_domain.entry(domain).or_insert(0) += 1;
        }
        if !(is_schema
            || is_generated
            || is_registry_input
            || is_docs_ref
            || is_required_ref
            || is_schema_ref
            || is_fixture_or_test)
        {
            computed_orphans.push(rel_str);
        }
    }

    let effective_orphans = computed_orphans
        .iter()
        .filter(|path| {
            !orphan_allowlist_prefixes
                .iter()
                .any(|prefix| path.starts_with(prefix))
        })
        .cloned()
        .collect::<Vec<_>>();

    if !effective_orphans.is_empty() {
        violations.push(violation(
            "OPS_DATA_FILE_ORPHAN_FOUND",
            format!(
                "orphan ops data artifacts detected: {}",
                effective_orphans.join(", ")
            ),
            "remove orphan data files or classify them through contracts-map, schema-index, docs, and REQUIRED_FILES",
            Some(Path::new("ops")),
        ));
    }

    let ledger_missing = ops_all_files
        .iter()
        .filter(|path| !ledger_paths.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    if !ledger_missing.is_empty() {
        violations.push(violation(
            "OPS_LEDGER_FILE_MISSING",
            format!(
                "ops ledger is missing {} file entries",
                ledger_missing.len()
            ),
            "regenerate ops-ledger.json and include every ops file",
            Some(ledger_rel),
        ));
    }
    for entry in &ledger_entries {
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        if path.is_empty() || !ctx.adapters.fs.exists(ctx.repo_root, Path::new(path)) {
            violations.push(violation(
                "OPS_LEDGER_REFERENCE_MISSING",
                format!("ops ledger references missing path `{path}`"),
                "remove stale ledger entries or restore referenced files",
                Some(ledger_rel),
            ));
            continue;
        }
        let reasons = entry
            .get("reasons")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if reasons.is_empty() {
            violations.push(violation(
                "OPS_LEDGER_REASON_MISSING",
                format!("ops ledger entry has empty reasons: `{path}`"),
                "add at least one necessity reason per ledger entry",
                Some(ledger_rel),
            ));
        }
        if let Some(schema_ref) = entry.get("schema_ref").and_then(|v| v.as_str()) {
            if !schema_ref.is_empty() && !ctx.adapters.fs.exists(ctx.repo_root, Path::new(schema_ref)) {
                violations.push(violation(
                    "OPS_LEDGER_SCHEMA_REFERENCE_MISSING",
                    format!("ledger schema reference is missing: `{schema_ref}`"),
                    "fix or remove stale schema_ref values in ops ledger",
                    Some(ledger_rel),
                ));
            }
        }
        for required_by in entry
            .get("required_by")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|v| v.as_str())
        {
            if !required_by.is_empty() && !ctx.adapters.fs.exists(ctx.repo_root, Path::new(required_by)) {
                violations.push(violation(
                    "OPS_LEDGER_REQUIRED_BY_MISSING",
                    format!("ledger required_by reference is missing: `{required_by}`"),
                    "fix required_by references in ops ledger",
                    Some(ledger_rel),
                ));
            }
        }
        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or_default();
        if entry_type == "authored" && path.contains("/generated/") {
            violations.push(violation(
                "OPS_LEDGER_AUTHORED_IN_GENERATED_PATH",
                format!("authored ledger entry lives under generated path: `{path}`"),
                "mark generated-path entries as generated or move authored files out of generated directories",
                Some(ledger_rel),
            ));
        }
        if entry_type == "generated"
            && !path.contains("/generated/")
            && !path.starts_with("ops/_generated.example/")
        {
            violations.push(violation(
                "OPS_LEDGER_GENERATED_OUTSIDE_GENERATED_PATH",
                format!("generated ledger entry is outside generated paths: `{path}`"),
                "move generated artifacts under ops/**/generated or ops/_generated.example",
                Some(ledger_rel),
            ));
        }
        let covered_by_authority_index = authority_index_paths.contains(path)
            || registry_inputs.contains(path)
            || schema_files.contains(path)
            || path.contains("/generated/")
            || path.starts_with("ops/_generated.example/")
            || authority_ignored_paths.contains(path)
            || authority_ignored_prefixes
                .iter()
                .any(|prefix| path.starts_with(prefix));
        if !covered_by_authority_index {
            violations.push(violation(
                "OPS_AUTHORITY_INDEX_COVERAGE_MISSING",
                format!(
                    "ops file `{path}` is not covered by authority-index, contracts-map, generated/schema classification, or explicit ignore rule"
                ),
                "register the file in authority-index/contracts-map or add an explicit ignore rule in ops/inventory/authority-index.json",
                Some(authority_index_rel),
            ));
        }
    }

    let allowlist_rel = Path::new("ops/_generated.example/ALLOWLIST.json");
    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_json: serde_json::Value = serde_json::from_str(&allowlist_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_set = allowlist_json
        .get("allowed_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    for entry in &ledger_entries {
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or_default();
        if entry_type == "curated_evidence" && !allowlist_set.contains(path) {
            violations.push(violation(
                "OPS_LEDGER_CURATED_NOT_ALLOWLISTED",
                format!("curated evidence path is not in allowlist: `{path}`"),
                "add curated evidence paths to ops/_generated.example/ALLOWLIST.json",
                Some(allowlist_rel),
            ));
        }
    }

    let binary_allowed = [".json", ".yaml", ".yml", ".toml", ".md", ".txt", ".lock", ".js"];
    for path in &ops_all_files {
        if path.contains("/assets/") {
            continue;
        }
        let ext = Path::new(path)
            .extension()
            .and_then(|v| v.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        if !binary_allowed.iter().any(|allowed| *allowed == ext) && !path.ends_with(".gitkeep") {
            violations.push(violation(
                "OPS_BINARY_OUTSIDE_ASSETS_FORBIDDEN",
                format!("potential binary or unsupported file outside assets: `{path}`"),
                "keep non-text artifacts only under ops/**/assets and declare them in fixture contracts",
                Some(Path::new(path)),
            ));
        }
    }

    if let Some(orphan_arr) = usage_report_json.get("orphans").and_then(|v| v.as_array()) {
        let report_orphans = orphan_arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let computed_orphan_set = effective_orphans.into_iter().collect::<BTreeSet<_>>();
        if report_orphans != computed_orphan_set {
            violations.push(violation(
                "OPS_FILE_USAGE_REPORT_ORPHAN_MISMATCH",
                "ops/_generated.example/file-usage-report.json orphan list is stale".to_string(),
                "regenerate and commit file-usage-report.json after updating ops artifacts",
                Some(usage_report_rel),
            ));
        }
    }

    let registry_budget = BTreeMap::from([
        ("inventory".to_string(), 35usize),
        ("load".to_string(), 70usize),
        ("observe".to_string(), 50usize),
        ("k8s".to_string(), 45usize),
        ("datasets".to_string(), 30usize),
        ("e2e".to_string(), 20usize),
        ("stack".to_string(), 20usize),
        ("env".to_string(), 10usize),
        ("report".to_string(), 10usize),
        ("schema".to_string(), 120usize),
    ]);
    for (domain, count) in registry_count_by_domain {
        if let Some(max) = registry_budget.get(&domain) {
            if count > *max {
                violations.push(violation(
                    "OPS_REGISTRY_FILE_BUDGET_EXCEEDED",
                    format!(
                        "registry/config file budget exceeded for `{domain}`: {count} > {max}"
                    ),
                    "consolidate or remove registry/config files before adding new ones",
                    Some(Path::new("ops")),
                ));
            }
        }
    }
    let generated_budget = BTreeMap::from([
        ("_generated.example".to_string(), 20usize),
        ("stack".to_string(), 10usize),
        ("report".to_string(), 10usize),
        ("k8s".to_string(), 10usize),
        ("datasets".to_string(), 10usize),
        ("load".to_string(), 10usize),
        ("schema".to_string(), 10usize),
        ("e2e".to_string(), 10usize),
        ("observe".to_string(), 10usize),
    ]);
    for (domain, count) in generated_count_by_domain {
        if let Some(max) = generated_budget.get(&domain) {
            if count > *max {
                violations.push(violation(
                    "OPS_GENERATED_FILE_BUDGET_EXCEEDED",
                    format!("generated file budget exceeded for `{domain}`: {count} > {max}"),
                    "consolidate generated outputs and avoid adding redundant generated artifacts",
                    Some(Path::new("ops")),
                ));
            }
        }
    }

    Ok(violations)
}
