fn test_ops_schema_006_id_and_naming_consistency(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-006";
    let test_id = "ops.schema.id_and_naming_consistency";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".schema.json") {
            continue;
        }
        let Some(schema) = read_json(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "schema file must be valid json",
                Some(rel),
            ));
            continue;
        };
        let id = schema.get("$id").and_then(|v| v.as_str());
        let Some(id) = id else {
            violations.push(violation(
                contract_id,
                test_id,
                "schema must declare non-empty $id",
                Some(rel),
            ));
            continue;
        };
        if id.trim().is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "schema $id must not be blank",
                Some(rel),
            ));
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !id.ends_with(file_name) {
            violations.push(violation(
                contract_id,
                test_id,
                "schema $id should end with schema file name",
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
fn test_ops_schema_007_examples_validate_required_fields(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-007";
    let test_id = "ops.schema.examples_validate_required_fields";
    let lock_path = ctx.repo_root.join("ops/schema/generated/compatibility-lock.json");
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must be parseable",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let Some(targets) = lock.get("targets").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must contain targets array",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for target in targets {
        let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(example_path) = target.get("example_path").and_then(|v| v.as_str()) else {
            violations.push(violation(
                contract_id,
                test_id,
                "compatibility lock target must declare example_path",
                Some(schema_path.to_string()),
            ));
            continue;
        };
        let example_abs = ctx.repo_root.join(example_path);
        let example = if example_path.ends_with(".yaml") || example_path.ends_with(".yml") {
            fs::read_to_string(&example_abs)
                .ok()
                .and_then(|raw| serde_yaml::from_str::<serde_json::Value>(&raw).ok())
        } else {
            read_json(&example_abs)
        };
        let Some(example) = example else {
            violations.push(violation(
                contract_id,
                test_id,
                "example fixture must be parseable json or yaml",
                Some(example_path.to_string()),
            ));
            continue;
        };
        let required_fields = target
            .get("required_fields")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for field in required_fields {
            let Some(name) = field.as_str() else {
                continue;
            };
            if example.get(name).is_none() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "example fixture is missing required field",
                    Some(example_path.to_string()),
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
fn test_ops_schema_008_forbid_duplicate_schema_intent(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-008";
    let test_id = "ops.schema.forbid_duplicate_intent";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    files.sort();
    let mut ids: BTreeMap<String, String> = BTreeMap::new();
    let mut titles: BTreeMap<String, String> = BTreeMap::new();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".schema.json") {
            continue;
        }
        let Some(schema) = read_json(&path) else {
            continue;
        };
        if let Some(id) = schema.get("$id").and_then(|v| v.as_str()) {
            if let Some(existing) = ids.insert(id.to_string(), rel.clone()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "duplicate schema $id detected",
                    Some(format!("{existing} | {rel}")),
                ));
            }
        }
        if let Some(title) = schema.get("title").and_then(|v| v.as_str()) {
            let normalized = title.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                continue;
            }
            if let Some(existing) = titles.insert(normalized, rel.clone()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "duplicate schema title detected",
                    Some(format!("{existing} | {rel}")),
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
fn test_ops_schema_009_canonical_json_formatting(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-009";
    let test_id = "ops.schema.canonical_json_formatting";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema/generated"), &mut files);
    files.sort();
    let mut violations = Vec::new();
    fn sort_json(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut sorted = serde_json::Map::new();
                let mut keys = map.keys().cloned().collect::<Vec<_>>();
                keys.sort();
                for key in keys {
                    if let Some(v) = map.get(&key) {
                        sorted.insert(key, sort_json(v));
                    }
                }
                serde_json::Value::Object(sorted)
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(sort_json).collect())
            }
            other => other.clone(),
        }
    }
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".json") {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) else {
            violations.push(violation(
                contract_id,
                test_id,
                "generated json must be parseable",
                Some(rel),
            ));
            continue;
        };
        let canonical = sort_json(&parsed);
        let expected = match serde_json::to_string_pretty(&canonical) {
            Ok(v) => format!("{v}\n"),
            Err(_) => continue,
        };
        if raw != expected {
            violations.push(violation(
                contract_id,
                test_id,
                "generated json must use canonical pretty formatting with trailing newline",
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
fn test_ops_schema_010_example_coverage(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-010";
    let test_id = "ops.schema.example_coverage";
    let lock_path = ctx.repo_root.join("ops/schema/generated/compatibility-lock.json");
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must be parseable",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let Some(targets) = lock.get("targets").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must contain targets array",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for target in targets {
        let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(example_path) = target.get("example_path").and_then(|v| v.as_str()) else {
            violations.push(violation(
                contract_id,
                test_id,
                "schema target must define example_path for CI coverage",
                Some(schema_path.to_string()),
            ));
            continue;
        };
        if !ctx.repo_root.join(example_path).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "example_path referenced by compatibility lock does not exist",
                Some(example_path.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

