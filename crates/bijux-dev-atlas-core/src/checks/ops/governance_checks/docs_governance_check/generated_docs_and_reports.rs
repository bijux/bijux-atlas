fn validate_ops_generated_docs_and_reports(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let reference_index_rel = Path::new("docs/operations/ops-system/INDEX.md");
    let reference_index_text = fs::read_to_string(ctx.repo_root.join(reference_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let docs_root = ctx.repo_root.join("docs/operations/ops-system");
    for doc in walk_files(&docs_root) {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = rel.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == "INDEX.md" {
            continue;
        }
        if !reference_index_text.contains(&format!("({name})")) {
            violations.push(violation(
                "OPS_REPORT_DOC_ORPHAN",
                format!(
                    "ops doc `{}` is not linked from docs/operations/ops-system/INDEX.md",
                    rel.display()
                ),
                "add doc link to docs/operations/ops-system/INDEX.md or remove orphan ops-system doc",
                Some(reference_index_rel),
            ));
        }
    }
    for target in markdown_link_targets(&reference_index_text) {
        let rel = Path::new("docs/operations/ops-system").join(&target);
        if !ctx.adapters.fs.exists(ctx.repo_root, &rel) {
            violations.push(violation(
                "OPS_REPORT_DOC_REFERENCE_BROKEN_LINK",
                format!(
                    "docs/operations/ops-system/INDEX.md links missing ops doc `{}`",
                    rel.display()
                ),
                "fix broken docs links in docs/operations/ops-system/INDEX.md",
                Some(reference_index_rel),
            ));
        }
    }

    let control_plane_rel = Path::new("ops/CONTROL_PLANE.md");
    let control_plane_snapshot_rel = Path::new("ops/_generated.example/control-plane.snapshot.md");
    let control_plane_drift_rel = Path::new("ops/_generated.example/control-plane-drift-report.json");
    let control_plane_surface_list_rel =
        Path::new("ops/_generated.example/control-plane-surface-list.json");
    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_snapshot_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_SNAPSHOT_MISSING",
            format!(
                "missing control-plane snapshot `{}`",
                control_plane_snapshot_rel.display()
            ),
            "generate and commit control-plane snapshot for drift checks",
            Some(control_plane_snapshot_rel),
        ));
    } else {
        let current = fs::read_to_string(ctx.repo_root.join(control_plane_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let snapshot = fs::read_to_string(ctx.repo_root.join(control_plane_snapshot_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if current != snapshot {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SNAPSHOT_DRIFT",
                "ops/CONTROL_PLANE.md does not match ops/_generated.example/control-plane.snapshot.md"
                    .to_string(),
                "refresh control-plane snapshot to match current control-plane contract",
                Some(control_plane_snapshot_rel),
            ));
        }
        for line in current.lines() {
            let lower = line.to_ascii_lowercase();
            if (lower.contains("example") || lower.contains("examples")) || !line.contains("bijux-")
            {
                continue;
            }
            violations.push(violation(
                "OPS_CONTROL_PLANE_CRATE_LIST_FORBIDDEN",
                format!(
                    "ops/CONTROL_PLANE.md contains crate reference outside example context: `{}`",
                    line.trim()
                ),
                "keep ops/CONTROL_PLANE.md policy-only; move current crate inventory to ops/_generated.example/control-plane.snapshot.md",
                Some(control_plane_rel),
            ));
            break;
        }
    }

    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_drift_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_DRIFT_REPORT_MISSING",
            format!(
                "missing control-plane drift report `{}`",
                control_plane_drift_rel.display()
            ),
            "generate and commit control-plane drift report artifact",
            Some(control_plane_drift_rel),
        ));
    } else {
        let drift_text = fs::read_to_string(ctx.repo_root.join(control_plane_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let drift_json: serde_json::Value =
            serde_json::from_str(&drift_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        if drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_CONTROL_PLANE_DRIFT_REPORT_BLOCKING",
                "control-plane-drift-report.json status is not `pass`".to_string(),
                "resolve control-plane drift and regenerate control-plane-drift-report.json",
                Some(control_plane_drift_rel),
            ));
        }
        let has_surface_check = drift_json
            .get("checks")
            .and_then(|v| v.as_array())
            .map(|checks| {
                checks.iter().any(|item| {
                    item.get("id").and_then(|v| v.as_str())
                        == Some("control-plane-surface-list-present")
                        && item.get("status").and_then(|v| v.as_str()) == Some("pass")
                })
            })
            .unwrap_or(false);
        if !has_surface_check {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SURFACE_LIST_CHECK_MISSING",
                "control-plane-drift-report.json must include passing `control-plane-surface-list-present` check"
                    .to_string(),
                "regenerate control-plane drift report with control-plane surface-list status",
                Some(control_plane_drift_rel),
            ));
        }
    }

    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_surface_list_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_SURFACE_LIST_MISSING",
            format!(
                "missing control-plane surface list report `{}`",
                control_plane_surface_list_rel.display()
            ),
            "generate and commit control-plane-surface-list report",
            Some(control_plane_surface_list_rel),
        ));
    } else {
        let surface_text = fs::read_to_string(ctx.repo_root.join(control_plane_surface_list_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let surface_json: serde_json::Value = serde_json::from_str(&surface_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if surface_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SURFACE_LIST_BLOCKING",
                "control-plane-surface-list.json status is not `pass`".to_string(),
                "resolve control-plane surface-list drift and regenerate the report",
                Some(control_plane_surface_list_rel),
            ));
        }
        let surfaces = surface_json
            .get("surfaces")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let expected = ["check", "docs", "configs", "ops"];
        for required in expected {
            if !surfaces
                .iter()
                .any(|value| value.as_str() == Some(required))
            {
                violations.push(violation(
                    "OPS_CONTROL_PLANE_SURFACE_LIST_INCOMPLETE",
                    format!(
                        "control-plane-surface-list.json missing required surface `{required}`"
                    ),
                    "regenerate control-plane surface list report from command ownership source",
                    Some(control_plane_surface_list_rel),
                ));
            }
        }
    }

    Ok(())
}

