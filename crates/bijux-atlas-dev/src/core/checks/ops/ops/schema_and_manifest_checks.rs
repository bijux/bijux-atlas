fn checks_ops_schema_presence(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let required = [
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
        "ops/schema/inventory/public-surface.schema.json",
        "ops/schema/meta/schema-index.schema.json",
        "ops/schema/meta/compatibility-lock.schema.json",
        "ops/schema/meta/ownership.schema.json",
        "ops/schema/meta/namespaces.schema.json",
        "ops/schema/meta/pins.schema.json",
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
        "ops/audit/event.schema.json".to_string(),
        "ops/audit/report.schema.json".to_string(),
        "ops/cli/schema/command-output.schema.json".to_string(),
        "ops/k8s/charts/bijux-atlas/values.schema.json".to_string(),
        "ops/observe/drills/result.schema.json".to_string(),
        "ops/observe/pack/compose.schema.json".to_string(),
        "ops/inventory/policies/dev-atlas-policy.schema.json".to_string(),
        "ops/load/contracts/load-report.schema.json".to_string(),
        "ops/load/contracts/load-summary.schema.json".to_string(),
    ]);
    let embedded_schema_prefix_allowlist =
        [
            "ops/docker/schema/",
            "ops/observe/",
            "ops/release/",
            "ops/reproducibility/",
        ];
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
        if !file.starts_with("ops/schema")
            && !embedded_schema_allowlist.contains(&rel_str)
            && !embedded_schema_prefix_allowlist
                .iter()
                .any(|prefix| rel_str.starts_with(prefix))
        {
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
        "ops/schema/inventory/public-surface.schema.json",
        "ops/schema/meta/schema-index.schema.json",
        "ops/schema/meta/compatibility-lock.schema.json",
        "ops/schema/meta/namespaces.schema.json",
        "ops/schema/meta/pins.schema.json",
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
                    "regenerate ops/schema/generated/schema-index.json",
                    Some(index_rel),
                ));
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

    let contract_rel = Path::new("ops/CONTRACT.md");
    let contract_text = fs::read_to_string(ctx.repo_root.join(contract_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !contract_text.contains("ops/schema/generated/schema-index.json") {
        violations.push(violation(
            "OPS_SCHEMA_INDEX_NOT_LINKED_FROM_CONTRACT",
            "ops/CONTRACT.md must link ops/schema/generated/schema-index.json".to_string(),
            "link the schema index JSON from the root ops contract",
            Some(contract_rel),
        ));
    }
    let ops_index_rel = Path::new("ops/INDEX.md");
    let ops_index_text = fs::read_to_string(ctx.repo_root.join(ops_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !ops_index_text.contains("ops/schema/generated/schema-index.json") {
        violations.push(violation(
            "OPS_SCHEMA_INDEX_NOT_LINKED_FROM_INDEX",
            "ops/INDEX.md must link ops/schema/generated/schema-index.json".to_string(),
            "add schema index link to ops/INDEX.md",
            Some(ops_index_rel),
        ));
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
