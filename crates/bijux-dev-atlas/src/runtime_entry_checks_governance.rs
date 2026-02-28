pub(crate) fn run_check_tree_budgets(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(repo_root)?;
    let mut errors = Vec::<String>::new();
    let mut deepest = Vec::<(String, usize)>::new();
    let forbidden_dir_names = ["misc", "tmp", "old", "legacy"];

    let rules = [
        ("configs", 4usize, 10usize),
        ("docs", 4usize, 10usize),
        ("ops", 5usize, usize::MAX),
    ];

    let exceptions_path = repo_root.join("configs/repo/tree-budget-exceptions.json");
    let exception_prefixes = if exceptions_path.exists() {
        let text = fs::read_to_string(&exceptions_path)
            .map_err(|e| format!("failed to read {}: {e}", exceptions_path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("failed to parse {}: {e}", exceptions_path.display()))?;
        value["allow_path_prefixes"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    for (root_name, max_depth, max_top_dirs) in rules {
        let root = repo_root.join(root_name);
        if !root.exists() {
            continue;
        }
        let top_level_dirs = fs::read_dir(&root)
            .map_err(|e| format!("failed to list {}: {e}", root.display()))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .count();
        if top_level_dirs > max_top_dirs {
            errors.push(format!(
                "TREE_BUDGET_ERROR: `{root_name}` top-level dirs {top_level_dirs} exceed budget {max_top_dirs}"
            ));
        }

        for file in walk_files_local(&root) {
            let rel = file
                .strip_prefix(&repo_root)
                .unwrap_or(&file)
                .display()
                .to_string();
            if exception_prefixes.iter().any(|prefix| rel.starts_with(prefix)) {
                continue;
            }
            let depth = file
                .strip_prefix(&root)
                .unwrap_or(&file)
                .components()
                .count();
            deepest.push((rel.clone(), depth));
            if depth > max_depth {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` depth {depth} exceeds `{root_name}` budget {max_depth}"
                ));
            }
            if (root_name == "configs" || root_name == "docs")
                && file
                    .components()
                    .any(|c| {
                        c.as_os_str()
                            .to_str()
                            .map(|name| forbidden_dir_names.contains(&name))
                            .unwrap_or(false)
                    })
            {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` uses forbidden directory name in configs/docs"
                ));
            }
        }
    }

    for root_name in ["configs", "docs"] {
        let root = repo_root.join(root_name);
        if !root.exists() {
            continue;
        }
        for dir in walk_files_local(&root)
            .into_iter()
            .filter_map(|p| p.parent().map(Path::to_path_buf))
            .collect::<std::collections::BTreeSet<_>>()
        {
            let rel_dir = dir
                .strip_prefix(&repo_root)
                .unwrap_or(&dir)
                .display()
                .to_string();
            if rel_dir.contains("/_generated") || rel_dir.contains("/_drafts") {
                continue;
            }
            let index_path = dir.join("INDEX.md");
            if !index_path.exists() {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: directory `{rel_dir}` is missing required `INDEX.md`"
                ));
            }
        }
    }

    let mut basename_paths = std::collections::BTreeMap::<String, Vec<String>>::new();
    for root_name in ["configs", "docs"] {
        for file in walk_files_local(&repo_root.join(root_name)) {
            let rel = file
                .strip_prefix(&repo_root)
                .unwrap_or(&file)
                .display()
                .to_string();
            if let Some(name) = file.file_name().and_then(|v| v.to_str()) {
                if matches!(name, "INDEX.md" | "README.md" | "OWNERS.md") {
                    continue;
                }
                basename_paths.entry(name.to_string()).or_default().push(rel);
            }
        }
    }
    for (name, paths) in basename_paths {
        if paths.len() > 1 {
            errors.push(format!(
                "TREE_BUDGET_ERROR: duplicate filename `{name}` across docs/configs: {}",
                paths.join(", ")
            ));
        }
    }

    let check_owner_coverage = |owners_path: &Path, prefix: &str| -> Result<Vec<String>, String> {
        let mut errs = Vec::<String>::new();
        if !owners_path.exists() {
            errs.push(format!(
                "TREE_BUDGET_ERROR: missing owners file `{}`",
                owners_path
                    .strip_prefix(&repo_root)
                    .unwrap_or(owners_path)
                    .display()
            ));
            return Ok(errs);
        }
        let text = fs::read_to_string(owners_path)
            .map_err(|e| format!("failed to read {}: {e}", owners_path.display()))?;
        let mut covered = std::collections::BTreeSet::<String>::new();
        for line in text.lines() {
            if let Some(idx) = line.find(&format!("`{prefix}/")) {
                let rest = &line[idx + 1..];
                if let Some(end) = rest.find('`') {
                    covered.insert(rest[..end].to_string());
                }
            }
        }
        let root = repo_root.join(prefix);
        if root.exists() {
            for entry in fs::read_dir(&root)
                .map_err(|e| format!("failed to list {}: {e}", root.display()))?
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
            {
                if let Some(name) = entry.file_name().to_str() {
                    let key = format!("{prefix}/{name}");
                    if !covered.contains(&key) {
                        errs.push(format!(
                            "TREE_BUDGET_ERROR: missing owner mapping for `{key}` in `{}`",
                            owners_path
                                .strip_prefix(&repo_root)
                                .unwrap_or(owners_path)
                                .display()
                        ));
                    }
                }
            }
        }
        Ok(errs)
    };
    errors.extend(check_owner_coverage(&repo_root.join("configs/OWNERS.md"), "configs")?);
    errors.extend(check_owner_coverage(&repo_root.join("docs/OWNERS.md"), "docs")?);

    let make_help = repo_root.join("make/help.md");
    let make_targets = repo_root.join("make/target-list.json");
    if make_help.exists() && make_targets.exists() {
        let help_text = fs::read_to_string(&make_help)
            .map_err(|e| format!("failed to read {}: {e}", make_help.display()))?;
        let targets_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&make_targets)
                .map_err(|e| format!("failed to read {}: {e}", make_targets.display()))?,
        )
        .map_err(|e| format!("failed to parse {}: {e}", make_targets.display()))?;
        for target in targets_json["public_targets"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
        {
            if !help_text.contains(&format!("- {target}:")) {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: public make target `{target}` missing from make/help.md"
                ));
            }
        }
    }

    for rel in [
        "docs/reference/commands.md",
        "docs/reference/schemas.md",
        "docs/reference/configs.md",
        "docs/reference/make-targets.md",
    ] {
        let path = repo_root.join(rel);
        if !path.exists() {
            errors.push(format!(
                "TREE_BUDGET_ERROR: missing required generated reference page `{rel}`"
            ));
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        if !text.contains("This page is generated by") {
            errors.push(format!(
                "TREE_BUDGET_ERROR: `{rel}` must declare generated artifact marker"
            ));
        }
    }

    let command_index_path = repo_root.join("docs/_generated/command-index.json");
    let known_command_ids = if command_index_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&command_index_path)
                .map_err(|e| format!("read {} failed: {e}", command_index_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", command_index_path.display()))?;
        value["commands"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| row["id"].as_str().map(str::to_string))
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        std::collections::BTreeSet::new()
    };

    let make_public_targets = if make_targets.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&make_targets)
                .map_err(|e| format!("read {} failed: {e}", make_targets.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", make_targets.display()))?;
        value["public_targets"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        std::collections::BTreeSet::new()
    };

    let schema_index_path = repo_root.join("docs/_generated/schema-index.json");
    let known_schema_paths = if schema_index_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&schema_index_path)
                .map_err(|e| format!("read {} failed: {e}", schema_index_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", schema_index_path.display()))?;
        value["schemas"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| row["path"].as_str().map(str::to_string))
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        std::collections::BTreeSet::new()
    };

    let command_ref_re =
        Regex::new(r"`bijux dev atlas ([a-z0-9_-]+(?: [a-z0-9_-]+)?)`").map_err(|e| e.to_string())?;
    let make_ref_re = Regex::new(r"`make ([a-z0-9_-]+)`").map_err(|e| e.to_string())?;
    let path_ref_re = Regex::new(r"`((?:configs|ops/schema)/[a-zA-Z0-9_./-]+)`")
        .map_err(|e| e.to_string())?;
    for doc in walk_files_local(&repo_root.join("docs"))
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
    {
        let rel = doc
            .strip_prefix(&repo_root)
            .unwrap_or(&doc)
            .display()
            .to_string();
        let text = fs::read_to_string(&doc).unwrap_or_default();
        for cap in command_ref_re.captures_iter(&text) {
            let ref_cmd = cap
                .get(1)
                .map(|m| m.as_str().replace(' ', "."))
                .unwrap_or_default();
            if !known_command_ids.is_empty() && !known_command_ids.contains(&ref_cmd) {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references unknown command `bijux dev atlas {}`",
                    cap.get(1).map(|m| m.as_str()).unwrap_or_default()
                ));
            }
        }
        for cap in make_ref_re.captures_iter(&text) {
            let ref_make = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            if !make_public_targets.is_empty() && !make_public_targets.contains(ref_make) {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references unknown make target `make {ref_make}`"
                ));
            }
        }
        for cap in path_ref_re.captures_iter(&text) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            if path.starts_with("configs/") && !repo_root.join(path).exists() {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references missing config path `{path}`"
                ));
            }
            if path.starts_with("ops/schema/")
                && !known_schema_paths.is_empty()
                && !known_schema_paths.contains(path)
            {
                errors.push(format!(
                    "TREE_BUDGET_ERROR: `{rel}` references schema path not present in schema index `{path}`"
                ));
            }
        }
    }

    deepest.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    deepest.truncate(20);

    let payload = serde_json::json!({
        "schema_version": 1,
        "text": if errors.is_empty() { "tree budgets passed" } else { "tree budgets failed" },
        "errors": errors,
        "deepest_paths": deepest
            .iter()
            .map(|(path, depth)| serde_json::json!({"path": path, "depth": depth}))
            .collect::<Vec<_>>(),
        "exceptions_file": if exceptions_path.exists() {
            serde_json::Value::String("configs/repo/tree-budget-exceptions.json".to_string())
        } else {
            serde_json::Value::Null
        }
    });

    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) { 1 } else { 0 }))
}

pub(crate) fn run_check_repo_doctor(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let (tree_rendered, tree_code) =
        run_check_tree_budgets(Some(root.clone()), FormatArg::Json, None)?;
    let tree_payload: serde_json::Value =
        serde_json::from_str(&tree_rendered).map_err(|e| format!("tree payload parse failed: {e}"))?;

    let docs_payload = docs_validate_payload(
        &docs_context(&DocsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            strict: true,
            include_drafts: false,
        })?,
        &DocsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            strict: true,
            include_drafts: false,
        },
    )?;
    let docs_code = if docs_payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
        1
    } else {
        0
    };

    let configs_payload = configs_validate_payload(
        &configs_context(&ConfigsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_write: false,
            allow_subprocess: false,
            allow_network: false,
            strict: true,
        })?,
        &ConfigsCommonArgs {
            repo_root: Some(root.clone()),
            artifacts_root: None,
            run_id: None,
            format: FormatArg::Json,
            out: None,
            allow_write: false,
            allow_subprocess: false,
            allow_network: false,
            strict: true,
        },
    )?;
    let configs_code = if configs_payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
        1
    } else {
        0
    };

    let mut docs_indexes = walk_files_local(&root.join("docs"))
        .into_iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(&root).ok()?.display().to_string();
            if rel.ends_with("/INDEX.md") || rel == "docs/INDEX.md" {
                Some(rel)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    docs_indexes.sort();
    let mut make_targets = Vec::<String>::new();
    let target_list_path = root.join("make/target-list.json");
    if target_list_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&target_list_path)
                .map_err(|e| format!("read {} failed: {e}", target_list_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", target_list_path.display()))?;
        make_targets = value["public_targets"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        make_targets.sort();
    }
    let mut config_groups = fs::read_dir(root.join("configs"))
        .map_err(|e| format!("list configs failed: {e}"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .collect::<Vec<_>>();
    config_groups.sort();

    let snapshot = serde_json::json!({
        "schema_version": 1,
        "make_public_targets": make_targets,
        "docs_indexes": docs_indexes,
        "config_groups": config_groups
    });
    let snapshot_rel = Path::new("configs/repo/surface-snapshot.json");
    let mut snapshot_drift_error = serde_json::Value::Null;
    let snapshot_path = root.join(snapshot_rel);
    if snapshot_path.exists() {
        let expected: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&snapshot_path)
                .map_err(|e| format!("read {} failed: {e}", snapshot_path.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", snapshot_path.display()))?;
        if expected != snapshot {
            snapshot_drift_error = serde_json::json!(format!(
                "REPO_SURFACE_DRIFT_ERROR: `{}` does not match current repo surface snapshot",
                snapshot_rel.display()
            ));
        }
    } else {
        snapshot_drift_error = serde_json::json!(format!(
            "REPO_SURFACE_DRIFT_ERROR: missing `{}`",
            snapshot_rel.display()
        ));
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "text": if tree_code == 0 && docs_code == 0 && configs_code == 0 && snapshot_drift_error.is_null() { "repo doctor passed" } else { "repo doctor failed" },
        "checks": {
            "tree_budgets": tree_payload,
            "docs_validate": docs_payload,
            "configs_validate": configs_payload
        },
        "surface_snapshot": snapshot,
        "surface_snapshot_contract": snapshot_rel.display().to_string(),
        "surface_snapshot_drift_error": snapshot_drift_error
    });
    let rendered = emit_payload(format, out, &payload)?;
    let code = if tree_code == 0
        && docs_code == 0
        && configs_code == 0
        && payload["surface_snapshot_drift_error"].is_null()
    {
        0
    } else {
        1
    };
    Ok((rendered, code))
}

pub(crate) fn run_check_registry_doctor(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let report = registry_doctor(&root);
    let status = if report.errors.is_empty() {
        "ok"
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "repo_root": root.display().to_string(),
        "errors": report.errors,
    });
    let rendered = match format {
        FormatArg::Text => format!(
            "status: {status}\nerrors: {}",
            payload["errors"].as_array().map_or(0, Vec::len)
        ),
        FormatArg::Json => serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&payload).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

