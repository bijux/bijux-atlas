fn validate_ops_generated_docs_and_reports(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let drift_rel = Path::new("ops/_generated.example/control-plane-drift-report.json");
    let drift_text = fs::read_to_string(ctx.repo_root.join(drift_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let drift_json: serde_json::Value =
        serde_json::from_str(&drift_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if drift_json.get("status").and_then(|value| value.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_CONTROL_PLANE_DRIFT_REPORT_BLOCKING",
            "control-plane-drift-report.json status is not `pass`".to_string(),
            "refresh control-plane drift evidence after fixing the underlying mismatch",
            Some(drift_rel),
        ));
    }

    let surface_rel = Path::new("ops/_generated.example/control-plane-surface-list.json");
    let surface_text = fs::read_to_string(ctx.repo_root.join(surface_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surface_json: serde_json::Value =
        serde_json::from_str(&surface_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if surface_json.get("status").and_then(|value| value.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_CONTROL_PLANE_SURFACE_LIST_BLOCKING",
            "control-plane-surface-list.json status is not `pass`".to_string(),
            "refresh control-plane surface evidence after fixing the command surface drift",
            Some(surface_rel),
        ));
    }

    let surfaces = surface_json
        .get("surfaces")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    for required in ["check", "configs", "docs", "ops"] {
        if !surfaces.contains(required) {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SURFACE_LIST_INCOMPLETE",
                format!("control-plane surface evidence is missing `{required}`"),
                "refresh ops/_generated.example/control-plane-surface-list.json from the live command surface",
                Some(surface_rel),
            ));
        }
    }

    Ok(())
}
