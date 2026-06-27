fn validate_fixture_inventory_and_drift_reports(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let fixtures_root = ctx.repo_root.join("ops/datasets/fixtures");
    let fixture_inventory_rel = Path::new("ops/datasets/generated/fixture-inventory.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, fixture_inventory_rel) {
        violations.push(violation(
            "OPS_FIXTURE_INVENTORY_ARTIFACT_MISSING",
            format!(
                "missing fixture inventory generated artifact `{}`",
                fixture_inventory_rel.display()
            ),
            "generate and commit ops/datasets/generated/fixture-inventory.json",
            Some(fixture_inventory_rel),
        ));
    } else {
        let text = fs::read_to_string(ctx.repo_root.join(fixture_inventory_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let Some(fixtures) = json.get("fixtures").and_then(|v| v.as_array()) else {
            violations.push(violation(
                "OPS_FIXTURE_INVENTORY_SHAPE_INVALID",
                "fixture inventory must contain a fixtures array".to_string(),
                "populate fixtures array in fixture-inventory.json",
                Some(fixture_inventory_rel),
            ));
            return Ok(());
        };

        let mut indexed_versions = BTreeMap::new();
        for entry in fixtures {
            let Some(name) = entry.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(version) = entry.get("version").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(asset) = entry.get("asset").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(asset_sha) = entry.get("asset_sha256").and_then(|v| v.as_str()) else {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_HASH_MISSING",
                    format!("fixture inventory entry `{name}/{version}` is missing asset_sha256"),
                    "add asset_sha256 for each fixture inventory entry",
                    Some(fixture_inventory_rel),
                ));
                continue;
            };
            indexed_versions.insert(
                format!("{name}/{version}"),
                (asset.to_string(), asset_sha.to_string()),
            );
        }

        let mut discovered_versions = BTreeMap::new();
        for manifest in walk_files(&fixtures_root)
            .into_iter()
            .filter(|p| p.file_name().and_then(|v| v.to_str()) == Some("manifest.lock"))
        {
            let rel = manifest
                .strip_prefix(ctx.repo_root)
                .unwrap_or(manifest.as_path())
                .display()
                .to_string();
            let parts = rel.split('/').collect::<Vec<_>>();
            if parts.len() < 6 {
                continue;
            }
            let fixture_name = parts[3];
            let fixture_version = parts[4];
            let key = format!("{fixture_name}/{fixture_version}");
            let manifest_text =
                fs::read_to_string(&manifest).map_err(|err| CheckError::Failed(err.to_string()))?;
            let archive = manifest_text
                .lines()
                .find_map(|line| line.strip_prefix("archive="))
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let manifest_sha = manifest_text
                .lines()
                .find_map(|line| line.strip_prefix("sha256="))
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let asset =
                format!("ops/datasets/fixtures/{fixture_name}/{fixture_version}/assets/{archive}");
            let asset_path = ctx.repo_root.join(format!(
                "ops/datasets/fixtures/{fixture_name}/{fixture_version}/assets/{archive}"
            ));
            let asset_sha = if archive.is_empty() || !asset_path.exists() {
                String::new()
            } else {
                sha256_hex(&asset_path)?
            };
            if !manifest_sha.is_empty() && manifest_sha != asset_sha {
                violations.push(violation(
                    "OPS_FIXTURE_MANIFEST_SHA_STALE",
                    format!(
                        "manifest sha256 is stale for fixture `{key}`: manifest={} actual={}",
                        manifest_sha, asset_sha
                    ),
                    "refresh fixture manifest.lock sha256 after asset changes",
                    Some(Path::new(&rel)),
                ));
            }
            discovered_versions.insert(key, (asset, asset_sha));
        }

        for (key, (asset, sha)) in &discovered_versions {
            let Some((indexed_asset, indexed_sha)) = indexed_versions.get(key) else {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ENTRY_MISSING",
                    format!("fixture inventory missing entry for `{key}`"),
                    "add fixture version entry to ops/datasets/generated/fixture-inventory.json",
                    Some(fixture_inventory_rel),
                ));
                continue;
            };
            if indexed_asset != asset {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ASSET_PATH_DRIFT",
                    format!(
                        "fixture inventory asset path drift for `{key}`: expected `{asset}` got `{indexed_asset}`"
                    ),
                    "refresh fixture inventory asset paths from fixture manifests",
                    Some(fixture_inventory_rel),
                ));
            }
            if indexed_sha != sha {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ASSET_HASH_DRIFT",
                    format!(
                        "fixture inventory hash drift for `{key}`: expected `{sha}` got `{indexed_sha}`"
                    ),
                    "refresh fixture inventory hashes from fixture assets",
                    Some(fixture_inventory_rel),
                ));
            }
        }
        for key in indexed_versions.keys() {
            if !discovered_versions.contains_key(key) {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_STALE_ENTRY",
                    format!("fixture inventory has stale entry `{key}`"),
                    "remove stale fixture inventory entries not backed by fixture manifests",
                    Some(fixture_inventory_rel),
                ));
            }
        }
    }

    let fixture_drift_rel = Path::new("ops/_generated.example/fixture-drift-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, fixture_drift_rel) {
        violations.push(violation(
            "OPS_FIXTURE_DRIFT_REPORT_MISSING",
            format!(
                "missing fixture drift report artifact `{}`",
                fixture_drift_rel.display()
            ),
            "generate and commit fixture drift report under ops/_generated.example",
            Some(fixture_drift_rel),
        ));
    } else {
        let fixture_drift_text = fs::read_to_string(ctx.repo_root.join(fixture_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let fixture_drift_json: serde_json::Value = serde_json::from_str(&fixture_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in ["schema_version", "generated_by", "status", "summary", "drift"] {
            if fixture_drift_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_FIXTURE_DRIFT_REPORT_INVALID",
                    format!("fixture drift report is missing required key `{key}`"),
                    "populate fixture drift report with required governance keys",
                    Some(fixture_drift_rel),
                ));
            }
        }
        if !matches!(
            fixture_drift_json.get("status").and_then(|v| v.as_str()),
            Some("clean" | "pass")
        ) {
            violations.push(violation(
                "OPS_FIXTURE_DRIFT_REPORT_BLOCKING",
                "fixture drift report status must be `clean` or `pass`".to_string(),
                "resolve fixture drift and regenerate fixture-drift-report.json",
                Some(fixture_drift_rel),
            ));
        }
    }
    Ok(())
}
