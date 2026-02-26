// SPDX-License-Identifier: Apache-2.0

pub(super) fn checks_ops_minimalism_and_deletion_safety(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    validate_directory_necessity_contract(ctx, &mut violations)?;
    validate_minimal_release_surface_contract(ctx, &mut violations)?;
    validate_load_scenario_retention(ctx, &mut violations)?;
    validate_inventory_drill_usage_contract(ctx, &mut violations)?;
    Ok(violations)
}

fn validate_directory_necessity_contract(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let rel = Path::new("ops/DIRECTORY_NECESSITY.md");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
    for required in [
        "- Owner:",
        "- Purpose:",
        "- Consumers:",
        "## Canonical Directories",
        "## Deletion Safety Notes",
        "`ops/datasets`",
        "`ops/e2e`",
        "`ops/env`",
        "`ops/inventory`",
        "`ops/k8s`",
        "`ops/load`",
        "`ops/observe`",
        "`ops/report`",
        "`ops/schema`",
        "`ops/stack`",
    ] {
        if !text.contains(required) {
            violations.push(violation(
                "OPS_DIRECTORY_NECESSITY_CONTRACT_INCOMPLETE",
                format!(
                    "directory necessity contract `{}` is missing `{required}`",
                    rel.display()
                ),
                "complete the directory necessity contract with all canonical directory declarations",
                Some(rel),
            ));
        }
    }
    Ok(())
}

fn validate_minimal_release_surface_contract(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let rel = Path::new("ops/MINIMAL_RELEASE_SURFACE.md");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
    for required in [
        "- Owner:",
        "- Purpose:",
        "- Consumers:",
        "## Minimal Release Surface",
        "## Deletion Impact Rules",
        "ops/inventory/contracts-map.json",
        "ops/inventory/authority-index.json",
        "ops/load/suites/suites.json",
        "ops/observe/drills/drills.json",
        "ops/report/generated/readiness-score.json",
    ] {
        if !text.contains(required) {
            violations.push(violation(
                "OPS_MINIMAL_RELEASE_SURFACE_INCOMPLETE",
                format!(
                    "minimal release surface contract `{}` is missing `{required}`",
                    rel.display()
                ),
                "declare the minimal release surface paths and deletion impact rules",
                Some(rel),
            ));
        }
    }
    Ok(())
}

fn validate_load_scenario_retention(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let suites_rel = Path::new("ops/load/suites/suites.json");
    let suites_text = fs::read_to_string(ctx.repo_root.join(suites_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", suites_rel.display())))?;
    let suites_json: serde_json::Value = serde_json::from_str(&suites_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", suites_rel.display())))?;
    let used_scenarios = suites_json
        .get("suites")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            entry.get("scenario").and_then(|v| v.as_str()).map(|name| {
                if name.starts_with("ops/load/scenarios/") {
                    name.to_string()
                } else {
                    format!("ops/load/scenarios/{name}")
                }
            })
        })
        .collect::<BTreeSet<_>>();

    let scenarios_dir = ctx.repo_root.join("ops/load/scenarios");
    let mut scenario_files = BTreeSet::new();
    if scenarios_dir.exists() {
        for file in walk_files(&scenarios_dir) {
            if file.extension().and_then(|v| v.to_str()) != Some("json") {
                continue;
            }
            let rel = file
                .strip_prefix(ctx.repo_root)
                .unwrap_or(file.as_path())
                .display()
                .to_string();
            scenario_files.insert(rel);
        }
    }

    let retention_rel = Path::new("ops/load/SCENARIO_RETENTION.md");
    let retention_text = fs::read_to_string(ctx.repo_root.join(retention_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", retention_rel.display())))?;
    if !retention_text.contains("## Unreferenced Scenario Retention") {
        violations.push(violation(
            "OPS_LOAD_SCENARIO_RETENTION_SECTION_MISSING",
            format!(
                "load scenario retention contract `{}` must include `## Unreferenced Scenario Retention`",
                retention_rel.display()
            ),
            "add the unreferenced scenario retention section with path + reason bullets",
            Some(retention_rel),
        ));
    }

    let retained_paths = retention_text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let path = trimmed
                .strip_prefix("- `")
                .and_then(|rest| rest.split_once("`"))
                .map(|(path, _)| path)?;
            if path.starts_with("ops/load/scenarios/") {
                Some(path.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    let unused = scenario_files
        .difference(&used_scenarios)
        .cloned()
        .collect::<BTreeSet<_>>();
    let undeclared_unused = unused
        .difference(&retained_paths)
        .cloned()
        .collect::<Vec<_>>();
    if !undeclared_unused.is_empty() {
        violations.push(violation(
            "OPS_LOAD_SCENARIO_UNUSED_UNDECLARED",
            format!(
                "unreferenced load scenarios are missing retention declarations: {}",
                undeclared_unused.join(", ")
            ),
            "delete unused scenarios or add them to ops/load/SCENARIO_RETENTION.md with explicit reasons",
            Some(retention_rel),
        ));
    }
    let stale_retained = retained_paths
        .difference(&unused)
        .cloned()
        .collect::<Vec<_>>();
    if !stale_retained.is_empty() {
        violations.push(violation(
            "OPS_LOAD_SCENARIO_RETENTION_STALE",
            format!(
                "retention declarations are stale because scenarios are now referenced or missing: {}",
                stale_retained.join(", ")
            ),
            "remove stale retention declarations after wiring scenarios into suites or deleting them",
            Some(retention_rel),
        ));
    }
    Ok(())
}

fn validate_inventory_drill_usage_contract(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let drills_rel = Path::new("ops/inventory/drills.json");
    let drills_text = fs::read_to_string(ctx.repo_root.join(drills_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", drills_rel.display())))?;
    let drills_json: serde_json::Value = serde_json::from_str(&drills_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", drills_rel.display())))?;
    let drill_ids = drills_json
        .get("drills")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();

    let links_rel = Path::new("ops/inventory/drill-contract-links.json");
    let links_text = fs::read_to_string(ctx.repo_root.join(links_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", links_rel.display())))?;
    let links_json: serde_json::Value = serde_json::from_str(&links_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", links_rel.display())))?;
    let linked_ids = links_json
        .get("links")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.get("drill_id").and_then(|x| x.as_str()).map(ToString::to_string))
        .collect::<BTreeSet<_>>();

    let unlinked = drill_ids
        .difference(&linked_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !unlinked.is_empty() {
        violations.push(violation(
            "OPS_INVENTORY_DRILL_UNUSED",
            format!(
                "inventory drills are not linked by drill-contract-links.json: {}",
                unlinked.join(", ")
            ),
            "link every inventory drill id in ops/inventory/drill-contract-links.json or remove it from ops/inventory/drills.json",
            Some(links_rel),
        ));
    }
    Ok(())
}
