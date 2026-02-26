fn validate_inventory_owners_registry_and_policy_contracts(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
    owners_rel: &Path,
    registry_rel: &Path,
    tools_rel: &Path,
    policy_rel: &Path,
    policy_schema_rel: &Path,
) -> Result<(), CheckError> {
    let owners_text = fs::read_to_string(ctx.repo_root.join(owners_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let owners_json: serde_json::Value =
        serde_json::from_str(&owners_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let owner_areas = owners_json
        .get("areas")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let required_owner_prefixes = [
        "ops/datasets",
        "ops/e2e",
        "ops/env",
        "ops/inventory",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/schema",
        "ops/stack",
    ];
    for prefix in required_owner_prefixes {
        if !owner_areas.contains_key(prefix) {
            violations.push(violation(
                "OPS_OWNER_DOMAIN_MAPPING_MISSING",
                format!("owners.json is missing required area mapping `{prefix}`"),
                "add owner mapping for each top-level ops domain",
                Some(owners_rel),
            ));
        }
    }
    let owner_prefixes = owner_areas.keys().cloned().collect::<Vec<_>>();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let rel_str = rel.display().to_string();
        if rel_str.starts_with("ops/_generated/") {
            continue;
        }
        let has_owner = owner_prefixes.iter().any(|prefix| {
            rel_str == *prefix
                || rel_str
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        });
        if !has_owner {
            violations.push(violation(
                "OPS_OWNER_ASSIGNMENT_MISSING",
                format!("ops file has no owner mapping in owners.json: `{rel_str}`"),
                "add an owners.json area mapping that covers this file path",
                Some(owners_rel),
            ));
        }
    }

    let registry_text = fs::read_to_string(ctx.repo_root.join(registry_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let tools_text = fs::read_to_string(ctx.repo_root.join(tools_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if registry_text.contains("[[tools]]") {
        violations.push(violation(
            "OPS_INVENTORY_REGISTRY_TOOLS_SURFACE_COLLISION",
            "ops/inventory/registry.toml must not define [[tools]] entries".to_string(),
            "keep registry.toml for checks/actions and tools.toml for tool probes",
            Some(registry_rel),
        ));
    }
    if tools_text.contains("[[checks]]") || tools_text.contains("[[actions]]") {
        violations.push(violation(
            "OPS_INVENTORY_TOOLS_REGISTRY_SURFACE_COLLISION",
            "ops/inventory/tools.toml must not define [[checks]] or [[actions]] entries"
                .to_string(),
            "keep tools.toml limited to [[tools]] entries",
            Some(tools_rel),
        ));
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, policy_schema_rel) {
        violations.push(violation(
            "OPS_INVENTORY_POLICY_SCHEMA_MISSING",
            format!("missing policy schema `{}`", policy_schema_rel.display()),
            "restore dev-atlas policy schema file",
            Some(policy_schema_rel),
        ));
    }
    let policy_text = fs::read_to_string(ctx.repo_root.join(policy_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let policy_json: serde_json::Value =
        serde_json::from_str(&policy_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if policy_json.get("schema_version").is_none() || policy_json.get("mode").is_none() {
        violations.push(violation(
            "OPS_INVENTORY_POLICY_REQUIRED_KEYS_MISSING",
            "dev-atlas policy is missing required top-level keys".to_string(),
            "ensure dev-atlas policy includes at least schema_version and mode",
            Some(policy_rel),
        ));
    }

    Ok(())
}
