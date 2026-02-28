fn test_configs_032_root_json_canonical(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for file in ROOT_CANONICAL_JSON_FILES {
        let path = ctx.repo_root.join(file);
        let text = match read_text(&path) {
            Ok(text) => text,
            Err(err) => {
                violations.push(violation(
                    "CONFIGS-032",
                    "configs.json.canonical_root_surface",
                    file,
                    err,
                ));
                continue;
            }
        };
        let value = match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(value) => value,
            Err(err) => {
                violations.push(violation(
                    "CONFIGS-032",
                    "configs.json.canonical_root_surface",
                    file,
                    format!("parse failed: {err}"),
                ));
                continue;
            }
        };
        let canonical = match canonical_json_string(&value) {
            Ok(text) => text,
            Err(err) => {
                violations.push(violation(
                    "CONFIGS-032",
                    "configs.json.canonical_root_surface",
                    file,
                    err,
                ));
                continue;
            }
        };
        if text != canonical {
            violations.push(violation(
                "CONFIGS-032",
                "configs.json.canonical_root_surface",
                file,
                "json must use stable two-space pretty formatting with lexicographically sorted object keys",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_033_schema_index_matches_committed(ctx: &RunContext) -> TestResult {
    let expected = match schema_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-033",
                "configs.schema.index_committed_match",
                SCHEMAS_PATH,
                err,
            )
        }
    };
    let path = ctx
        .repo_root
        .join("configs/schema/generated/schema-index.json");
    let text = match read_text(&path) {
        Ok(text) => text,
        Err(err) => {
            return fail(
                "CONFIGS-033",
                "configs.schema.index_committed_match",
                "configs/schema/generated/schema-index.json",
                err,
            )
        }
    };
    let actual = match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-033",
                "configs.schema.index_committed_match",
                "configs/schema/generated/schema-index.json",
                format!("parse configs/schema/generated/schema-index.json failed: {err}"),
            )
        }
    };
    if actual == expected {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-033",
            "configs.schema.index_committed_match",
            "configs/schema/generated/schema-index.json",
            "committed schema index does not match the canonical schema map render",
        )
    }
}

fn test_configs_034_no_orphan_input_schemas(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-034",
                "configs.schema.no_orphan_inputs",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let payload = match schema_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-034",
                "configs.schema.no_orphan_inputs",
                SCHEMAS_PATH,
                err,
            )
        }
    };
    let governed_public_schemas = index
        .group_files
        .get("schema")
        .map(|files| files.public.clone())
        .unwrap_or_default();
    let mut violations = Vec::new();
    for schema in payload["orphan_input_schemas"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str())
    {
        if !governed_public_schemas.contains(schema) {
            continue;
        }
        violations.push(violation(
            "CONFIGS-034",
            "configs.schema.no_orphan_inputs",
            schema,
            "input schema is not referenced by any governed config mapping",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_035_schema_versioning_policy(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let policy = match read_schema_versioning_policy(&ctx.repo_root) {
        Ok(policy) => policy,
        Err(err) => {
            return fail(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                SCHEMA_VERSIONING_POLICY_PATH,
                err,
            )
        }
    };
    let governed_public_schemas = index
        .group_files
        .get("schema")
        .map(|files| {
            files
                .public
                .iter()
                .filter(|file| schema_like(file))
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut covered = BTreeSet::new();
    let mut violations = Vec::new();
    if policy.schema_version != 1 {
        violations.push(violation(
            "CONFIGS-035",
            "configs.schema.versioning_policy",
            SCHEMA_VERSIONING_POLICY_PATH,
            format!(
                "unsupported schema versioning policy schema_version {}",
                policy.schema_version
            ),
        ));
    }
    if policy.kind != "configs-schema-versioning-policy" {
        violations.push(violation(
            "CONFIGS-035",
            "configs.schema.versioning_policy",
            SCHEMA_VERSIONING_POLICY_PATH,
            format!("unexpected policy kind `{}`", policy.kind),
        ));
    }
    for rule in &policy.policies {
        if rule.versioning != "locked" {
            violations.push(violation(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                &rule.schema,
                format!("unsupported versioning policy `{}`", rule.versioning),
            ));
        }
        if rule.compatibility != "backward-compatible" {
            violations.push(violation(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                &rule.schema,
                format!("unsupported compatibility policy `{}`", rule.compatibility),
            ));
        }
        if !governed_public_schemas.contains(&rule.schema) {
            violations.push(violation(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                &rule.schema,
                "schema policy entry does not belong to the governed public schema surface",
            ));
        }
        if !ctx.repo_root.join(&rule.schema).is_file() {
            violations.push(violation(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                &rule.schema,
                "schema policy entry references a missing schema file",
            ));
        }
        if !covered.insert(rule.schema.clone()) {
            violations.push(violation(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                &rule.schema,
                "schema policy entry is duplicated",
            ));
        }
    }
    for schema in governed_public_schemas {
        if !covered.contains(&schema) {
            violations.push(violation(
                "CONFIGS-035",
                "configs.schema.versioning_policy",
                &schema,
                "governed public schema file is missing from the schema versioning policy",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_036_exclusion_governance(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-036",
                "configs.exclusions.governed_metadata",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for exclusion in &index.registry.exclusions {
        match exclusion.approved_by.as_deref() {
            Some(value) if !value.trim().is_empty() => {}
            _ => violations.push(violation(
                "CONFIGS-036",
                "configs.exclusions.governed_metadata",
                REGISTRY_PATH,
                format!(
                    "exclusion `{}` must declare a non-empty approved_by",
                    exclusion.pattern
                ),
            )),
        }
        match exclusion.expires_on.as_deref() {
            Some(value) if looks_like_iso_date(value) => {}
            Some(_) => violations.push(violation(
                "CONFIGS-036",
                "configs.exclusions.governed_metadata",
                REGISTRY_PATH,
                format!(
                    "exclusion `{}` must use YYYY-MM-DD expires_on",
                    exclusion.pattern
                ),
            )),
            None => violations.push(violation(
                "CONFIGS-036",
                "configs.exclusions.governed_metadata",
                REGISTRY_PATH,
                format!("exclusion `{}` must declare expires_on", exclusion.pattern),
            )),
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

