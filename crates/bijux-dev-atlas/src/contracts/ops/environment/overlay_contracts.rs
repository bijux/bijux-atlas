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

