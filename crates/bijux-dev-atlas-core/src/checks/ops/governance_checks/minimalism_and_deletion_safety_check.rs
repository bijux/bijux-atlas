// SPDX-License-Identifier: Apache-2.0

pub(super) fn checks_ops_minimalism_and_deletion_safety(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    validate_directory_necessity_contract(ctx, &mut violations)?;
    validate_minimal_release_surface_contract(ctx, &mut violations)?;
    validate_load_scenario_retention(ctx, &mut violations)?;
    validate_inventory_drill_usage_contract(ctx, &mut violations)?;
    validate_unused_schema_detection(ctx, &mut violations)?;
    validate_deletion_impact_example_report(ctx, &mut violations)?;
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

fn validate_unused_schema_detection(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let allowlist_rel = Path::new("ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md");
    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", allowlist_rel.display())))?;
    let allowlisted = allowlist_text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let path = trimmed
                .strip_prefix("- `")
                .and_then(|rest| rest.split_once("`"))
                .map(|(path, _)| path)?;
            if path.starts_with("ops/schema/") && path.ends_with(".schema.json") {
                Some(path.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    let mut all_schema_paths = BTreeSet::new();
    let schema_dir = ctx.repo_root.join("ops/schema");
    if schema_dir.exists() {
        for file in walk_files(&schema_dir) {
            if file.extension().and_then(|v| v.to_str()) != Some("json") {
                continue;
            }
            let rel = file
                .strip_prefix(ctx.repo_root)
                .unwrap_or(file.as_path())
                .display()
                .to_string();
            if rel.ends_with(".schema.json") {
                all_schema_paths.insert(rel);
            }
        }
    }

    let mut search_roots = vec![ctx.repo_root.join("ops"), ctx.repo_root.join("docs")];
    let mut referenced = BTreeSet::new();
    for root in search_roots.drain(..) {
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            if rel_str == allowlist_rel.display().to_string() {
                continue;
            }
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            for schema_path in &all_schema_paths {
                if text.contains(schema_path) {
                    referenced.insert(schema_path.clone());
                }
            }
        }
    }

    let unused = all_schema_paths
        .difference(&referenced)
        .filter(|path| !allowlisted.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    if !unused.is_empty() {
        violations.push(violation(
            "OPS_SCHEMA_UNUSED_UNALLOWLISTED",
            format!(
                "schema contracts are not referenced in ops/docs and not allowlisted: {}",
                unused.join(", ")
            ),
            "reference schemas from contracts/docs or add explicit entries to ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md",
            Some(allowlist_rel),
        ));
    }
    Ok(())
}

fn validate_deletion_impact_example_report(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let rel = Path::new("ops/_generated.example/what-breaks-if-removed-report.json");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", rel.display())))?;
    if json
        .get("generated_by")
        .and_then(|v| v.as_str())
        .is_none_or(|s| s.trim().is_empty())
    {
        violations.push(violation(
            "OPS_DELETION_IMPACT_REPORT_GENERATOR_MISSING",
            format!("deletion impact report `{}` must include non-empty `generated_by`", rel.display()),
            "add generated_by metadata to what-breaks-if-removed report example",
            Some(rel),
        ));
    }
    let targets = json.get("targets").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if targets.is_empty() {
        violations.push(violation(
            "OPS_DELETION_IMPACT_REPORT_TARGETS_EMPTY",
            format!("deletion impact report `{}` must include at least one target", rel.display()),
            "add representative deletion impact targets and consumers",
            Some(rel),
        ));
    }
    for target in targets {
        let path = target.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let consumers = target
            .get("consumers")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if path.trim().is_empty() || consumers.is_empty() {
            violations.push(violation(
                "OPS_DELETION_IMPACT_REPORT_TARGET_INVALID",
                format!(
                    "deletion impact report `{}` has target entries missing `path` or `consumers`",
                    rel.display()
                ),
                "ensure every target declares a path and at least one consumer",
                Some(rel),
            ));
            break;
        }
    }
    Ok(())
}
