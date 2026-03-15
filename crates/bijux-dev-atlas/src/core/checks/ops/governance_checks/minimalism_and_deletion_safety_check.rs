// SPDX-License-Identifier: Apache-2.0

pub(super) fn checks_ops_minimalism_and_deletion_safety(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    validate_root_index_matches_live_dirs(ctx, &mut violations)?;
    validate_minimal_release_surface(ctx, &mut violations)?;
    validate_load_scenario_usage(ctx, &mut violations)?;
    validate_inventory_drill_usage(ctx, &mut violations)?;
    validate_deletion_impact_example_report(ctx, &mut violations)?;
    validate_ops_surface_budgets(ctx, &mut violations)?;
    Ok(violations)
}

fn validate_root_index_matches_live_dirs(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let index_rel = Path::new("ops/INDEX.md");
    let index_text = fs::read_to_string(ctx.repo_root.join(index_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", index_rel.display())))?;

    for required in [
        "ops/inventory/",
        "ops/schema/",
        "ops/env/",
        "ops/stack/",
        "ops/k8s/",
        "ops/observe/",
        "ops/load/",
        "ops/datasets/",
        "ops/e2e/",
        "ops/report/",
    ] {
        if !index_text.contains(required) {
            violations.push(violation(
                "OPS_INDEX_CANONICAL_DIR_MISSING",
                format!("ops index is missing canonical directory `{required}`"),
                "keep ops/INDEX.md aligned with the live authored pillars",
                Some(index_rel),
            ));
        }
    }

    Ok(())
}

fn validate_minimal_release_surface(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let contract_rel = Path::new("ops/CONTRACT.md");
    let contract_text = fs::read_to_string(ctx.repo_root.join(contract_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", contract_rel.display())))?;
    for required in [
        "ops/inventory/contracts-map.json",
        "ops/inventory/authority-index.json",
        "ops/load/suites/suites.json",
        "ops/observe/drills.json",
        "ops/report/generated/readiness-score.json",
    ] {
        if !contract_text.contains(required) {
            violations.push(violation(
                "OPS_MINIMAL_RELEASE_SURFACE_INCOMPLETE",
                format!("ops contract is missing minimal release surface path `{required}`"),
                "declare the full minimal release surface in ops/CONTRACT.md",
                Some(contract_rel),
            ));
        }
    }
    Ok(())
}

fn validate_load_scenario_usage(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let suites_rel = Path::new("ops/load/suites/suites.json");
    let suites_text = fs::read_to_string(ctx.repo_root.join(suites_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", suites_rel.display())))?;
    let suites_json: serde_json::Value = serde_json::from_str(&suites_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", suites_rel.display())))?;
    let used = suites_json
        .get("suites")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|entry| entry.get("scenario").and_then(|value| value.as_str()))
                .map(|name| {
                    if name.starts_with("ops/load/scenarios/") {
                        name.to_string()
                    } else {
                        format!("ops/load/scenarios/{name}")
                    }
                })
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let mut all = BTreeSet::new();
    for file in walk_files(&ctx.repo_root.join("ops/load/scenarios")) {
        if file.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let rel = file
            .strip_prefix(ctx.repo_root)
            .unwrap_or(file.as_path())
            .display()
            .to_string();
        all.insert(rel);
    }

    let unused = all.difference(&used).cloned().collect::<Vec<_>>();
    if !unused.is_empty() {
        violations.push(violation(
            "OPS_LOAD_SCENARIO_UNUSED",
            format!("load scenarios are not referenced by suites.json: {}", unused.join(", ")),
            "delete unused scenarios or wire them into ops/load/suites/suites.json",
            Some(suites_rel),
        ));
    }
    Ok(())
}

fn validate_inventory_drill_usage(
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
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let links_rel = Path::new("ops/inventory/drill-contract-links.json");
    let links_text = fs::read_to_string(ctx.repo_root.join(links_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", links_rel.display())))?;
    let links_json: serde_json::Value = serde_json::from_str(&links_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", links_rel.display())))?;
    let linked = links_json
        .get("links")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|entry| entry.get("drill_id").and_then(|value| value.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let unlinked = drill_ids.difference(&linked).cloned().collect::<Vec<_>>();
    if !unlinked.is_empty() {
        violations.push(violation(
            "OPS_INVENTORY_DRILL_UNUSED",
            format!(
                "inventory drills are not linked by drill-contract-links.json: {}",
                unlinked.join(", ")
            ),
            "link every inventory drill in ops/inventory/drill-contract-links.json",
            Some(links_rel),
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
        .and_then(|value| value.as_str())
        .is_none_or(|value| value.trim().is_empty())
    {
        violations.push(violation(
            "OPS_DELETION_IMPACT_REPORT_GENERATOR_MISSING",
            format!("deletion impact report `{}` must include non-empty `generated_by`", rel.display()),
            "add generated_by metadata to the deletion impact example report",
            Some(rel),
        ));
    }

    let targets = json
        .get("targets")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    if targets.is_empty() {
        violations.push(violation(
            "OPS_DELETION_IMPACT_REPORT_TARGETS_EMPTY",
            format!("deletion impact report `{}` must include at least one target", rel.display()),
            "keep representative deletion impact targets in the curated example report",
            Some(rel),
        ));
    }

    Ok(())
}

fn validate_ops_surface_budgets(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let ops_root = ctx.repo_root.join("ops");
    let file_count = walk_files(&ops_root).len();

    fn count_dirs(root: &Path) -> usize {
        let mut total = 0usize;
        let Ok(entries) = fs::read_dir(root) else {
            return 0;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += 1;
                total += count_dirs(&path);
            }
        }
        total
    }

    let dir_count = count_dirs(&ops_root);
    if dir_count > 140 {
        violations.push(violation(
            "OPS_DIRECTORY_COUNT_BUDGET_EXCEEDED",
            format!("ops directory count budget exceeded: count={dir_count}, budget=140"),
            "collapse redundant ops directories before adding more structure",
            Some(Path::new("ops")),
        ));
    }
    if file_count > 650 {
        violations.push(violation(
            "OPS_FILE_COUNT_BUDGET_EXCEEDED",
            format!("ops file count budget exceeded: count={file_count}, budget=650"),
            "remove unused ops files before adding more surface area",
            Some(Path::new("ops")),
        ));
    }

    Ok(())
}
