pub(super) fn checks_ops_domain_contract_coverage(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let contract_rel = Path::new("ops/CONTRACT.md");
    let contract_text = fs::read_to_string(ctx.repo_root.join(contract_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();

    for required in ["## Machine Authorities", "## Evidence", "## Minimal Release Surface"] {
        if !contract_text.contains(required) {
            violations.push(violation(
                "OPS_CONTRACT_ROOT_SECTION_MISSING",
                format!("ops contract is missing required section `{required}`"),
                "keep ops/CONTRACT.md as the single human contract surface for ops",
                Some(contract_rel),
            ));
        }
    }

    let registry_rel = Path::new("ops/inventory/contracts.json");
    let registry_text = fs::read_to_string(ctx.repo_root.join(registry_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let registry_json: serde_json::Value = serde_json::from_str(&registry_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contract_ids = registry_json
        .get("contract_ids")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if contract_ids.is_empty() {
        violations.push(violation(
            "OPS_CONTRACT_REGISTRY_EMPTY",
            "ops contract registry has no contract ids".to_string(),
            "populate ops/inventory/contracts.json with the live contract id set",
            Some(registry_rel),
        ));
    }

    let snapshot_rel = Path::new("ops/_generated.example/contracts-registry-snapshot.json");
    let snapshot_text = fs::read_to_string(ctx.repo_root.join(snapshot_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let snapshot_json: serde_json::Value = serde_json::from_str(&snapshot_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let snapshot_ids = snapshot_json
        .get("contracts")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|value| value.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if snapshot_ids.is_empty() {
        violations.push(violation(
            "OPS_CONTRACT_SNAPSHOT_EMPTY",
            "contracts registry snapshot has no contract entries".to_string(),
            "refresh ops/_generated.example/contracts-registry-snapshot.json from the live registry",
            Some(snapshot_rel),
        ));
        return Ok(violations);
    }

    let missing = contract_ids
        .difference(&snapshot_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        violations.push(violation(
            "OPS_CONTRACT_SNAPSHOT_MISSING_IDS",
            format!(
                "contracts registry snapshot is missing live contract ids: {}",
                missing.join(", ")
            ),
            "refresh ops/_generated.example/contracts-registry-snapshot.json after contract registry changes",
            Some(snapshot_rel),
        ));
    }

    let stale = snapshot_ids
        .difference(&contract_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !stale.is_empty() {
        violations.push(violation(
            "OPS_CONTRACT_SNAPSHOT_STALE_IDS",
            format!(
                "contracts registry snapshot contains removed contract ids: {}",
                stale.join(", ")
            ),
            "drop removed contract ids from ops/_generated.example/contracts-registry-snapshot.json",
            Some(snapshot_rel),
        ));
    }

    Ok(violations)
}
