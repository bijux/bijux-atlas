fn checks_ops_schema_presence(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let required = [
        "ops/schema/README.md",
        "ops/schema/inventory/gates.schema.json",
        "ops/schema/inventory/pin-freeze.schema.json",
        "ops/schema/inventory/pins.schema.json",
        "ops/schema/inventory/toolchain.schema.json",
        "ops/schema/env/overlay.schema.json",
        "ops/schema/datasets/manifest.schema.json",
        "ops/schema/e2e/expectations.schema.json",
        "ops/schema/e2e/coverage-matrix.schema.json",
        "ops/schema/datasets/dataset-index.schema.json",
        "ops/schema/datasets/dataset-lineage.schema.json",
        "ops/schema/datasets/fixture-inventory.schema.json",
        "ops/schema/datasets/promotion-rules.schema.json",
        "ops/schema/datasets/qc-metadata.schema.json",
        "ops/schema/datasets/rollback-policy.schema.json",
        "ops/schema/load/deterministic-seed-policy.schema.json",
        "ops/schema/load/k6-suite.schema.json",
        "ops/schema/load/perf-baseline.schema.json",
        "ops/schema/load/thresholds.schema.json",
        "ops/schema/configs/public-surface.schema.json",
        "ops/schema/meta/ownership.schema.json",
        "ops/schema/meta/namespaces.schema.json",
        "ops/schema/meta/pins.schema.json",
        "ops/schema/meta/required-files-contract.schema.json",
        "ops/schema/meta/inventory-index.schema.json",
        "ops/schema/meta/ops-index.schema.json",
        "ops/schema/meta/scorecard.schema.json",
        "ops/schema/report/unified.schema.json",
        "ops/schema/report/readiness-score.schema.json",
        "ops/schema/report/evidence-levels.schema.json",
        "ops/schema/stack/artifact-metadata.schema.json",
        "ops/schema/stack/dependency-graph.schema.json",
        "ops/schema/stack/profile-manifest.schema.json",
        "ops/schema/generated/schema-index.json",
        "ops/schema/generated/schema-index.md",
        "ops/schema/generated/compatibility-lock.json",
    ];
    let mut violations = Vec::new();
    for path in required {
        let rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_SCHEMA_REQUIRED_FILE_MISSING",
                format!("missing required schema file `{path}`"),
                "restore required schema file under ops/schema",
                Some(rel),
            ));
        }
    }

    let embedded_schema_allowlist = BTreeSet::from([
        "ops/k8s/charts/bijux-atlas/values.schema.json".to_string(),
        "ops/observe/drills/result.schema.json".to_string(),
        "ops/observe/pack/compose.schema.json".to_string(),
        "ops/inventory/policies/dev-atlas-policy.schema.json".to_string(),
    ]);
    for file in walk_files(&ctx.repo_root.join("ops"))
        .into_iter()
        .filter_map(|path| path.strip_prefix(ctx.repo_root).ok().map(PathBuf::from))
        .filter(|rel| rel.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .filter(|rel| {
            rel.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".schema.json"))
        })
    {
        let rel_str = file.display().to_string();
        if !file.starts_with("ops/schema") && !embedded_schema_allowlist.contains(&rel_str) {
            violations.push(violation(
                "OPS_SCHEMA_OUTSIDE_CANONICAL_TREE",
                format!(
                    "schema file must live under ops/schema unless allowlisted tool-native schema: `{}`",
                    file.display()
                ),
                "move schema into ops/schema or explicitly allowlist tool-native embedded schema",
                Some(&file),
            ));
        }
    }

    let schema_contracts = [
        "ops/schema/inventory/gates.schema.json",
        "ops/schema/inventory/pin-freeze.schema.json",
        "ops/schema/inventory/pins.schema.json",
        "ops/schema/inventory/toolchain.schema.json",
        "ops/schema/datasets/manifest.schema.json",
        "ops/schema/datasets/fixture-inventory.schema.json",
        "ops/schema/e2e/expectations.schema.json",
        "ops/schema/datasets/promotion-rules.schema.json",
        "ops/schema/load/k6-suite.schema.json",
        "ops/schema/load/perf-baseline.schema.json",
        "ops/schema/load/thresholds.schema.json",
        "ops/schema/configs/public-surface.schema.json",
        "ops/schema/meta/namespaces.schema.json",
        "ops/schema/meta/pins.schema.json",
        "ops/schema/meta/required-files-contract.schema.json",
        "ops/schema/meta/inventory-index.schema.json",
        "ops/schema/meta/ops-index.schema.json",
        "ops/schema/meta/scorecard.schema.json",
        "ops/schema/report/readiness-score.schema.json",
        "ops/schema/report/evidence-levels.schema.json",
        "ops/schema/load/deterministic-seed-policy.schema.json",
        "ops/schema/report/unified.schema.json",
    ];
    for path in schema_contracts {
        let rel = Path::new(path);
        let full = ctx.repo_root.join(rel);
        let Ok(text) = fs::read_to_string(&full) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            violations.push(violation(
                "OPS_SCHEMA_INVALID_JSON",
                format!("schema is not valid JSON: `{path}`"),
                "fix JSON syntax in schema file",
                Some(rel),
            ));
            continue;
        };
        let required_schema_version = value
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .any(|item| item.as_str() == Some("schema_version"))
            })
            .unwrap_or(false);
        if !required_schema_version {
            violations.push(violation(
                "OPS_SCHEMA_VERSION_FIELD_MISSING",
                format!("schema `{path}` must require `schema_version`"),
                "add `schema_version` to schema required fields",
                Some(rel),
            ));
        }
        let has_schema_version_property = value
            .get("properties")
            .and_then(|v| v.get("schema_version"))
            .is_some();
        if !has_schema_version_property {
            violations.push(violation(
                "OPS_SCHEMA_VERSION_PROPERTY_MISSING",
                format!("schema `{path}` must define `properties.schema_version`"),
                "add schema_version property definition",
                Some(rel),
            ));
        }

        if let Some(name) = rel.file_name().and_then(|n| n.to_str()) {
            let stem = name.trim_end_matches(".schema.json");
            let naming_valid = !stem.is_empty()
                && stem
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                && !stem.starts_with('-')
                && !stem.ends_with('-');
            if !naming_valid {
                violations.push(violation(
                    "OPS_SCHEMA_FILENAME_INVALID",
                    format!(
                        "schema filename must use lowercase kebab-case and `.schema.json` suffix: `{}`",
                        rel.display()
                    ),
                    "rename schema file to lowercase kebab-case naming",
                    Some(rel),
                ));
            }
        }
    }

    let mut actual_schema_files = walk_files(&ctx.repo_root.join("ops/schema"))
        .into_iter()
        .filter_map(|path| {
            let rel = path.strip_prefix(ctx.repo_root).ok()?.to_path_buf();
            if rel.starts_with("ops/schema/generated") {
                return None;
            }
            if rel
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "json")
                && rel
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".schema.json"))
            {
                return Some(rel.display().to_string());
            }
            None
        })
        .collect::<Vec<_>>();
    actual_schema_files.sort();

    let index_rel = Path::new("ops/schema/generated/schema-index.json");
    let index_path = ctx.repo_root.join(index_rel);
    if let Ok(text) = fs::read_to_string(&index_path) {
        if let Ok(index_json) = serde_json::from_str::<serde_json::Value>(&text) {
            let expected_files = index_json
                .get("files")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(ToString::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if expected_files != actual_schema_files {
                violations.push(violation(
                    "OPS_SCHEMA_DRIFT_DETECTED",
                    format!(
                        "schema index drift detected: expected={expected_files:?} actual={actual_schema_files:?}"
                    ),
                    "regenerate ops/schema/generated/schema-index.json and schema-index.md",
                    Some(index_rel),
                ));
            }

            let index_md_rel = Path::new("ops/schema/generated/schema-index.md");
            let index_md_text = fs::read_to_string(ctx.repo_root.join(index_md_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let mut markdown_paths = Vec::new();
            for line in index_md_text.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("| `ops/schema/") {
                    continue;
                }
                let Some(path) = trimmed
                    .strip_prefix("| `")
                    .and_then(|v| v.split("` |").next())
                    .map(ToString::to_string)
                else {
                    continue;
                };
                markdown_paths.push(path);
            }
            if markdown_paths != expected_files {
                violations.push(violation(
                    "OPS_SCHEMA_INDEX_MARKDOWN_DRIFT",
                    "ops/schema/generated/schema-index.md does not match schema-index.json entries"
                        .to_string(),
                    "regenerate schema-index.md from schema-index.json",
                    Some(index_md_rel),
                ));
            }
            for path in &markdown_paths {
                let rel = Path::new(path);
                if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
                    violations.push(violation(
                        "OPS_SCHEMA_INDEX_MARKDOWN_BROKEN_LINK",
                        format!("schema-index.md references missing schema `{path}`"),
                        "remove broken markdown entry or restore missing schema file",
                        Some(index_md_rel),
                    ));
                }
            }
        }
    }

    let compatibility_rel = Path::new("ops/schema/generated/compatibility-lock.json");
    let compatibility_path = ctx.repo_root.join(compatibility_rel);
    if let Ok(text) = fs::read_to_string(&compatibility_path) {
        if let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(targets) = lock_json.get("targets").and_then(|v| v.as_array()) {
                let mut previous = String::new();
                for target in targets {
                    let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str())
                    else {
                        continue;
                    };
                    if previous.as_str() > schema_path {
                        violations.push(violation(
                            "OPS_SCHEMA_COMPATIBILITY_LOCK_ORDER_INVALID",
                            "compatibility-lock targets must be sorted by schema_path".to_string(),
                            "sort targets lexicographically by schema_path",
                            Some(compatibility_rel),
                        ));
                    }
                    previous = schema_path.to_string();
                    if !actual_schema_files.contains(&schema_path.to_string()) {
                        violations.push(violation(
                            "OPS_SCHEMA_COMPATIBILITY_LOCK_STALE_PATH",
                            format!(
                                "compatibility-lock references schema not present in schema index: `{schema_path}`"
                            ),
                            "remove stale compatibility-lock target or restore schema file",
                            Some(compatibility_rel),
                        ));
                    }
                }
                for target in targets {
                    let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str())
                    else {
                        continue;
                    };
                    let rel = Path::new(schema_path);
                    let full = ctx.repo_root.join(rel);
                    let Ok(schema_text) = fs::read_to_string(&full) else {
                        continue;
                    };
                    let Ok(schema_json) = serde_json::from_str::<serde_json::Value>(&schema_text)
                    else {
                        continue;
                    };
                    let required_set = schema_json
                        .get("required")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str().map(ToString::to_string))
                                .collect::<BTreeSet<_>>()
                        })
                        .unwrap_or_default();
                    if let Some(locked) = target.get("required_fields").and_then(|v| v.as_array()) {
                        for field in locked.iter().filter_map(|item| item.as_str()) {
                            if !required_set.contains(field) {
                                violations.push(violation(
                                    "OPS_SCHEMA_BREAKING_CHANGE_DETECTED",
                                    format!(
                                        "schema `{schema_path}` removed locked required field `{field}`"
                                    ),
                                    "restore required field or update compatibility lock with explicit breaking-change process",
                                    Some(rel),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    let schema_readme_rel = Path::new("ops/schema/README.md");
    let schema_readme = fs::read_to_string(ctx.repo_root.join(schema_readme_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for required_link in [
        "ops/schema/VERSIONING_POLICY.md",
        "ops/schema/BUDGET_POLICY.md",
        "ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md",
        "ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md",
    ] {
        if !schema_readme.contains(required_link) {
            violations.push(violation(
                "OPS_SCHEMA_GOVERNANCE_LINK_MISSING",
                format!("ops/schema/README.md must link `{required_link}`"),
                "add required governance policy links to ops/schema/README.md",
                Some(schema_readme_rel),
            ));
        }
    }
    let ops_index_rel = Path::new("ops/INDEX.md");
    let ops_index_text = fs::read_to_string(ctx.repo_root.join(ops_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !ops_index_text.contains("ops/schema/VERSIONING_POLICY.md") {
        violations.push(violation(
            "OPS_SCHEMA_VERSIONING_POLICY_NOT_LINKED",
            "ops/INDEX.md must link ops/schema/VERSIONING_POLICY.md".to_string(),
            "add schema versioning policy link to ops/INDEX.md",
            Some(ops_index_rel),
        ));
    }

    const SCHEMA_BUDGET_CAP: usize = 90;
    if actual_schema_files.len() > SCHEMA_BUDGET_CAP {
        let exceptions_rel = Path::new("ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md");
        let exceptions_text = fs::read_to_string(ctx.repo_root.join(exceptions_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if !exceptions_text.contains("- ") {
            violations.push(violation(
                "OPS_SCHEMA_BUDGET_CAP_EXCEEDED",
                format!(
                    "schema count {} exceeds budget cap {} without approved exceptions",
                    actual_schema_files.len(),
                    SCHEMA_BUDGET_CAP
                ),
                "document approved exceptions in ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md",
                Some(exceptions_rel),
            ));
        }
    }

    let allowlist_rel = Path::new("ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md");
    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for line in allowlist_text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- ") {
            continue;
        }
        let Some(path) = trimmed.split('`').nth(1) else {
            violations.push(violation(
                "OPS_SCHEMA_REFERENCE_ALLOWLIST_LINE_INVALID",
                format!("allowlist entry must contain backtick path and reason: `{trimmed}`"),
                "format entries as `- `ops/schema/...`: reason`",
                Some(allowlist_rel),
            ));
            continue;
        };
        if !path.starts_with("ops/schema/") {
            violations.push(violation(
                "OPS_SCHEMA_REFERENCE_ALLOWLIST_PATH_INVALID",
                format!("allowlist schema path must start with ops/schema/: `{path}`"),
                "keep schema allowlist scoped to ops/schema/**",
                Some(allowlist_rel),
            ));
        }
        if !trimmed.contains(":") {
            violations.push(violation(
                "OPS_SCHEMA_REFERENCE_ALLOWLIST_REASON_MISSING",
                format!("allowlist entry is missing rationale: `{trimmed}`"),
                "append a rationale after `:` for every allowlist entry",
                Some(allowlist_rel),
            ));
        }
    }
    let allowed_unreferenced = allowlist_text
        .lines()
        .filter_map(|line| line.split('`').nth(1))
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    for schema_path in &actual_schema_files {
        let references = walk_files(&ctx.repo_root.join("ops"))
            .into_iter()
            .filter_map(|path| path.strip_prefix(ctx.repo_root).ok().map(PathBuf::from))
            .filter(|rel| !rel.starts_with("ops/schema"))
            .filter(|rel| rel.extension().and_then(|ext| ext.to_str()) != Some("lock"))
            .filter_map(|rel| {
                fs::read_to_string(ctx.repo_root.join(&rel))
                    .ok()
                    .map(|text| (rel, text))
            })
            .any(|(_rel, text)| text.contains(schema_path));
        if !references && !allowed_unreferenced.contains(schema_path) {
            violations.push(violation(
                "OPS_SCHEMA_UNREFERENCED",
                format!("schema `{schema_path}` is unreferenced outside ops/schema"),
                "reference schema from contract/config or add a reason in SCHEMA_REFERENCE_ALLOWLIST.md",
                Some(Path::new(schema_path)),
            ));
        }
    }

    let required_files_schema_rel =
        Path::new("ops/schema/meta/required-files-contract.schema.json");
    let required_files_schema_text =
        fs::read_to_string(ctx.repo_root.join(required_files_schema_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
    let required_files_schema_json: serde_json::Value =
        serde_json::from_str(&required_files_schema_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
    let required_fields = required_files_schema_json
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    for key in [
        "required_files",
        "required_dirs",
        "forbidden_patterns",
        "notes",
    ] {
        if !required_fields.contains(key) {
            violations.push(violation(
                "OPS_REQUIRED_FILES_SCHEMA_KEY_MISSING",
                format!("required-files schema must require key `{key}`"),
                "keep required-files meta schema aligned with REQUIRED_FILES contract format",
                Some(required_files_schema_rel),
            ));
        }
    }
    for required_doc in walk_files(&ctx.repo_root.join("ops"))
        .into_iter()
        .filter_map(|path| path.strip_prefix(ctx.repo_root).ok().map(PathBuf::from))
        .filter(|rel| rel.file_name().and_then(|n| n.to_str()) == Some("REQUIRED_FILES.md"))
    {
        let text = fs::read_to_string(ctx.repo_root.join(&required_doc))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let Some(yaml_block) = extract_required_files_yaml_block(&text) else {
            continue;
        };
        let yaml_value: serde_yaml::Value =
            serde_yaml::from_str(&yaml_block).map_err(|err| CheckError::Failed(err.to_string()))?;
        let json_value =
            serde_json::to_value(yaml_value).map_err(|err| CheckError::Failed(err.to_string()))?;
        let json_obj = json_value.as_object().cloned().unwrap_or_default();
        for key in [
            "required_files",
            "required_dirs",
            "forbidden_patterns",
            "notes",
        ] {
            if !json_obj.contains_key(key) {
                violations.push(violation(
                    "OPS_REQUIRED_FILES_CONTRACT_KEY_MISSING",
                    format!(
                        "required files contract `{}` is missing key `{key}`",
                        required_doc.display()
                    ),
                    "include all canonical REQUIRED_FILES keys",
                    Some(&required_doc),
                ));
            }
        }
    }

    for schema_rel in &actual_schema_files {
        let schema_path = Path::new(schema_rel);
        let schema_text = fs::read_to_string(ctx.repo_root.join(schema_path))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let schema_json: serde_json::Value = serde_json::from_str(&schema_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let mut strings = Vec::new();
        collect_json_string_values(&schema_json, &mut strings);
        for value in strings {
            if !(value.contains(".schema.json") || value.starts_with("ops/")) {
                continue;
            }
            if !(value.starts_with("ops/schema/") || value.starts_with("ops/")) {
                continue;
            }
            let ref_path = Path::new(&value);
            if ref_path.starts_with("ops/") && !ctx.adapters.fs.exists(ctx.repo_root, ref_path) {
                violations.push(violation(
                    "OPS_SCHEMA_REFERENCE_PATH_MISSING",
                    format!(
                        "schema `{}` references missing path `{}`",
                        schema_path.display(),
                        ref_path.display()
                    ),
                    "fix schema reference path or restore the referenced file",
                    Some(schema_path),
                ));
            }
        }
    }

    Ok(violations)
}

