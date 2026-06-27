pub(super) fn check_ops_evidence_bundle_discipline(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();

    let allowlist_rel = Path::new("ops/_generated.example/ALLOWLIST.json");
    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_json: serde_json::Value =
        serde_json::from_str(&allowlist_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlisted_files = allowlist_json
        .get("allowed_files")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if allowlisted_files.is_empty() {
        violations.push(violation(
            "OPS_EVIDENCE_ALLOWLIST_EMPTY",
            "ops/_generated.example/ALLOWLIST.json must declare non-empty `allowed_files`"
                .to_string(),
            "list the curated evidence files that are allowed under ops/_generated.example",
            Some(allowlist_rel),
        ));
    }

    for file in walk_files(&ctx.repo_root.join("ops/_generated.example")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let rel_str = rel.display().to_string();
        if !allowlisted_files.contains(&rel_str) {
            violations.push(violation(
                "OPS_EVIDENCE_ALLOWLIST_MISSING_FILE",
                format!(
                    "committed file `{}` is not declared in ops/_generated.example/ALLOWLIST.json",
                    rel.display()
                ),
                "update ALLOWLIST.json when adding or removing curated evidence artifacts",
                Some(allowlist_rel),
            ));
        }
    }

    let bundle_rel = Path::new("ops/report/generated/release-evidence-bundle.json");
    let bundle_text = fs::read_to_string(ctx.repo_root.join(bundle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let bundle_json: serde_json::Value =
        serde_json::from_str(&bundle_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in ["schema_version", "status", "gates"] {
        if bundle_json.get(key).is_none() {
            violations.push(violation(
                "OPS_RELEASE_EVIDENCE_BUNDLE_INCOMPLETE",
                format!("release evidence bundle is missing `{key}`"),
                "keep release-evidence-bundle.json aligned with the release evidence schema",
                Some(bundle_rel),
            ));
        }
    }

    let generated_root = ctx.repo_root.join("ops/_generated");
    if generated_root.exists() {
        let allowed = BTreeSet::from([
            "ops/_generated/.gitkeep".to_string(),
            "ops/_generated/control-plane-surface-list.json".to_string(),
        ]);
        for file in walk_files(&generated_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            if !allowed.contains(&rel_str) {
                violations.push(violation(
                    "OPS_GENERATED_DIRECTORY_COMMITTED_EVIDENCE_FORBIDDEN",
                    format!("ops/_generated contains unexpected committed file `{}`", rel.display()),
                    "keep ops/_generated limited to the small committed runtime surface",
                    Some(rel),
                ));
            }
        }
    }

    Ok(violations)
}
