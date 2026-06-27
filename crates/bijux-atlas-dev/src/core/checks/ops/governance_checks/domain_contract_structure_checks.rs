pub(super) fn check_ops_domain_contract_structure(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let fragments_root = ctx.repo_root.join("ops/inventory/contracts");
    let mut violations = Vec::new();

    for file in walk_files(&fragments_root) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let text = fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;

        if json.get("schema_version").and_then(|value| value.as_i64()).unwrap_or(0) < 1 {
            violations.push(violation(
                "OPS_CONTRACT_FRAGMENT_SCHEMA_VERSION_INVALID",
                format!(
                    "contract fragment `{}` must declare schema_version >= 1",
                    rel.display()
                ),
                "set schema_version on every ops contract fragment",
                Some(rel),
            ));
        }

        let path = json
            .get("path")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        if path.is_empty() || !ctx.adapters.fs.exists(ctx.repo_root, Path::new(path)) {
            violations.push(violation(
                "OPS_CONTRACT_FRAGMENT_PATH_INVALID",
                format!(
                    "contract fragment `{}` points at missing path `{path}`",
                    rel.display()
                ),
                "keep each contract fragment aligned with a live ops subtree",
                Some(rel),
            ));
        }

        let contract_doc = json
            .get("contract_doc")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        if contract_doc != "ops/CONTRACT.md" {
            violations.push(violation(
                "OPS_CONTRACT_FRAGMENT_DOC_DRIFT",
                format!(
                    "contract fragment `{}` must point at `ops/CONTRACT.md`, found `{contract_doc}`",
                    rel.display()
                ),
                "use ops/CONTRACT.md as the single human contract document for ops",
                Some(rel),
            ));
        }

        if json
            .get("title")
            .and_then(|value| value.as_str())
            .is_none_or(|value| value.trim().is_empty())
        {
            violations.push(violation(
                "OPS_CONTRACT_FRAGMENT_TITLE_MISSING",
                format!("contract fragment `{}` must include a non-empty title", rel.display()),
                "set a stable title on every ops contract fragment",
                Some(rel),
            ));
        }

        if json.get("version").and_then(|value| value.as_i64()).unwrap_or(0) < 1 {
            violations.push(violation(
                "OPS_CONTRACT_FRAGMENT_VERSION_INVALID",
                format!("contract fragment `{}` must include version >= 1", rel.display()),
                "set a positive version number on every ops contract fragment",
                Some(rel),
            ));
        }
    }

    Ok(violations)
}
