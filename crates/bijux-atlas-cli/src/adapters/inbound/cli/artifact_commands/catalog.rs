// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::adapters::inbound::cli::canonical_json;

pub(crate) fn validate_catalog(path: PathBuf, output_mode: OutputMode) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    let payload = json!({"command":"atlas catalog validate","status":"ok"});
    if output_mode.json {
        println!(
            "{}",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

pub(crate) fn publish_catalog(
    store_root: PathBuf,
    catalog_path: PathBuf,
    dry_run: bool,
    explain: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    let raw = fs::read_to_string(&catalog_path).map_err(|e| e.to_string())?;
    let mut catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.datasets = sorted_catalog_entries(catalog.datasets);
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    let canonical = canonical_catalog_json(&catalog)?;

    if dry_run || explain {
        return emit_ok_payload(
            output_mode,
            json!({
                "command":"atlas catalog publish",
                "mode": if explain { "explain" } else { "dry-run" },
                "status":"ok",
                "catalog_entries": catalog.datasets.len(),
                "target": store_root.join("catalog.json"),
                "writes_artifacts": false
            }),
        );
    }

    fs::create_dir_all(&store_root).map_err(|e| e.to_string())?;
    let tmp = store_root.join("catalog.json.tmp");
    fs::write(&tmp, canonical.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, store_root.join("catalog.json")).map_err(|e| e.to_string())?;

    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog publish","status":"ok"}),
    )
}

pub(crate) fn rollback_catalog(
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let path = store_root.join("catalog.json");
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let target = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    catalog.datasets.retain(|x| x.dataset != target);
    catalog.datasets = sorted_catalog_entries(catalog.datasets);
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    let canonical = canonical_catalog_json(&catalog)?;
    let tmp = store_root.join("catalog.json.tmp");
    fs::write(&tmp, canonical.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog rollback","status":"ok"}),
    )
}

pub(crate) fn promote_catalog(
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::dataset::artifact_paths(&store_root, &dataset);
    if !paths.manifest.exists() || !paths.sqlite.exists() {
        return Err(format!(
            "promote requires published artifact first: missing {} or {}",
            paths.manifest.display(),
            paths.sqlite.display()
        ));
    }

    let mut catalog = read_catalog_or_empty(&store_root)?;
    if !catalog.datasets.iter().any(|x| x.dataset == dataset) {
        catalog.datasets.push(CatalogEntry::new(
            dataset.clone(),
            rel_display_path(&store_root, &paths.manifest)?,
            rel_display_path(&store_root, &paths.sqlite)?,
        ));
    }
    catalog.datasets = sorted_catalog_entries(catalog.datasets);
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    write_catalog(&store_root, &catalog)?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog promote","status":"ok","dataset":dataset}),
    )
}

pub(crate) fn update_latest_alias(
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let catalog = read_catalog_or_empty(&store_root)?;
    if !catalog.datasets.iter().any(|x| x.dataset == dataset) {
        return Err(
            "latest alias update is gated by promotion: dataset not present in catalog".to_string(),
        );
    }
    fs::create_dir_all(&store_root).map_err(|e| e.to_string())?;
    let canonical_catalog = canonical_catalog_json(&catalog)?;
    let alias_record = bijux_atlas_model::dataset::LatestAliasRecord::new(
        dataset,
        "promotion-gated".to_string(),
        format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs()
        ),
        "atlas-cli".to_string(),
        sha256_hex(canonical_catalog.as_bytes()),
    );
    alias_record.validate().map_err(|e| e.to_string())?;
    let alias_path = store_root.join("latest.alias.json");
    let tmp = store_root.join("latest.alias.json.tmp");
    fs::write(&tmp, canonical_json::bytes(&alias_record)?).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &alias_path).map_err(|e| e.to_string())?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas catalog latest-alias-update","status":"ok","alias_path":alias_path}),
    )
}

fn read_catalog_or_empty(store_root: &Path) -> Result<Catalog, String> {
    let path = store_root.join("catalog.json");
    if !path.exists() {
        return Ok(Catalog::new(Vec::new()));
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(catalog)
}

fn write_catalog(store_root: &Path, catalog: &Catalog) -> Result<(), String> {
    let canonical = canonical_catalog_json(catalog)?;
    fs::create_dir_all(store_root).map_err(|e| e.to_string())?;
    let tmp = store_root.join("catalog.json.tmp");
    fs::write(&tmp, canonical.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, store_root.join("catalog.json")).map_err(|e| e.to_string())
}

fn rel_display_path(root: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| format!("path {} is outside {}", path.display(), root.display()))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}
