// SPDX-License-Identifier: Apache-2.0

use crate::{sha256_hex, OutputMode};
use bijux_atlas_core::canonical;
use bijux_atlas_model::{
    parse_dataset_key, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ReleaseGeneIndex,
    ShardCatalog,
};
use bijux_atlas_policies::{
    canonical_config_json, load_policy_from_workspace, resolve_mode_profile, PolicyMode,
};
use bijux_atlas_store::{
    canonical_catalog_json, sorted_catalog_entries, verify_expected_sha256, ArtifactStore,
    LocalFsStore, ManifestLock, StoreErrorCode,
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder, Header};

mod diff;
mod gc;

pub(crate) use diff::{build_release_diff, BuildReleaseDiffArgs};
pub(crate) use gc::{gc_apply, gc_plan};

mod dataset_validation;

#[cfg(test)]
use dataset_validation::validate_qc_thresholds;
pub(crate) use dataset_validation::{publish_dataset, validate_dataset, validate_ingest_qc};
#[cfg(test)]
use gc::compute_gc_plan;

pub(crate) fn parse_alias_map(input: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for pair in input.split(',') {
        let p = pair.trim();
        if p.is_empty() {
            continue;
        }
        if let Some((k, v)) = p.split_once('=') {
            out.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

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

pub(crate) fn validate_policy(output_mode: OutputMode) -> Result<(), String> {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "failed to resolve workspace root from CARGO_MANIFEST_DIR".to_string())?
        .to_path_buf();
    let policy = load_policy_from_workspace(&workspace).map_err(|e| e.to_string())?;
    let canonical = canonical_config_json(&policy).map_err(|e| e.to_string())?;
    if output_mode.json {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "command":"atlas policy validate",
                "status":"ok",
                "schema_version": policy.schema_version.as_str(),
                "canonical": serde_json::from_str::<serde_json::Value>(&canonical).map_err(|e| e.to_string())?
            }))
            .map_err(|e| e.to_string())?
        );
    } else {
        println!("{canonical}");
    }
    Ok(())
}

pub(crate) fn explain_policy(
    mode_override: Option<PolicyMode>,
    output_mode: OutputMode,
) -> Result<(), String> {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "failed to resolve workspace root from CARGO_MANIFEST_DIR".to_string())?
        .to_path_buf();
    let policy = load_policy_from_workspace(&workspace).map_err(|e| e.to_string())?;
    let active_mode = mode_override.unwrap_or(policy.mode);
    let strict = resolve_mode_profile(&policy, PolicyMode::Strict).map_err(|e| e.to_string())?;
    let active = resolve_mode_profile(&policy, active_mode).map_err(|e| e.to_string())?;
    let deltas = json!({
      "max_page_size": {
        "strict": strict.max_page_size,
        "active": active.max_page_size
      },
      "max_region_span": {
        "strict": strict.max_region_span,
        "active": active.max_region_span
      },
      "max_response_bytes": {
        "strict": strict.max_response_bytes,
        "active": active.max_response_bytes
      }
    });
    let payload = json!({
      "command": "atlas policy explain",
      "status": "ok",
      "mode": active_mode.as_str(),
      "strict_mode": "strict",
      "deltas_vs_strict": deltas
    });
    emit_ok_payload(output_mode, payload)
}

pub(crate) fn publish_catalog(
    store_root: PathBuf,
    catalog_path: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let raw = fs::read_to_string(&catalog_path).map_err(|e| e.to_string())?;
    let mut catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.datasets = sorted_catalog_entries(catalog.datasets);
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    let canonical = canonical_catalog_json(&catalog)?;

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
    let paths = bijux_atlas_model::artifact_paths(&store_root, &dataset);
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
    let alias_path = store_root.join("latest.alias.json");
    let tmp = store_root.join("latest.alias.json.tmp");
    fs::write(
        &tmp,
        canonical::stable_json_bytes(&json!({
            "dataset": dataset,
            "policy": "promotion-gated"
        }))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
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

pub(crate) fn pack_dataset(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    out: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::artifact_paths(&root, &dataset);
    let manifest = fs::read(&paths.manifest).map_err(|e| e.to_string())?;
    let sqlite = fs::read(&paths.sqlite).map_err(|e| e.to_string())?;
    let lock = ManifestLock::from_bytes(&manifest, &sqlite);
    let lock_bytes = serde_json::to_vec(&lock).map_err(|e| e.to_string())?;

    let file = fs::File::create(&out).map_err(|e| e.to_string())?;
    let mut builder = Builder::new(file);
    append_tar_file(&mut builder, "manifest.json", &manifest)?;
    append_tar_file(&mut builder, "gene_summary.sqlite", &sqlite)?;
    append_tar_file(&mut builder, "manifest.lock", &lock_bytes)?;
    builder.finish().map_err(|e| e.to_string())?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas dataset pack","status":"ok","out":out}),
    )
}

pub(crate) fn verify_pack(pack: PathBuf, output_mode: OutputMode) -> Result<(), String> {
    let file = fs::File::open(pack).map_err(|e| e.to_string())?;
    let mut archive = Archive::new(file);
    let mut manifest: Option<Vec<u8>> = None;
    let mut sqlite: Option<Vec<u8>> = None;
    let mut lock_raw: Option<Vec<u8>> = None;
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut e = entry.map_err(|e| e.to_string())?;
        let path = e
            .path()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut e, &mut bytes).map_err(|e| e.to_string())?;
        match path.as_str() {
            "manifest.json" => manifest = Some(bytes),
            "gene_summary.sqlite" => sqlite = Some(bytes),
            "manifest.lock" => lock_raw = Some(bytes),
            _ => {}
        }
    }
    let manifest = manifest.ok_or_else(|| "manifest.json missing in pack".to_string())?;
    let sqlite = sqlite.ok_or_else(|| "gene_summary.sqlite missing in pack".to_string())?;
    let lock_raw = lock_raw.ok_or_else(|| "manifest.lock missing in pack".to_string())?;
    let lock: ManifestLock = serde_json::from_slice(&lock_raw).map_err(|e| e.to_string())?;
    lock.validate(&manifest, &sqlite)?;
    emit_ok_payload(
        output_mode,
        json!({"command":"atlas dataset verify-pack","status":"ok"}),
    )
}

fn append_tar_file(
    builder: &mut Builder<std::fs::File>,
    name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_cksum();
    builder
        .append_data(&mut header, name, bytes)
        .map_err(|e| e.to_string())
}

fn emit_ok_payload(output_mode: OutputMode, payload: serde_json::Value) -> Result<(), String> {
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

fn validate_sqlite_contract(sqlite_path: &PathBuf) -> Result<(), String> {
    let conn = rusqlite::Connection::open(sqlite_path).map_err(|e| e.to_string())?;
    let required_indexes = [
        "idx_gene_summary_gene_id",
        "idx_gene_summary_name",
        "idx_gene_summary_name_normalized",
        "idx_gene_summary_biotype",
        "idx_gene_summary_region",
        "idx_gene_summary_cover_lookup",
        "idx_gene_summary_cover_region",
        "idx_transcript_summary_transcript_id",
        "idx_transcript_summary_parent_gene_id",
        "idx_transcript_summary_biotype",
        "idx_transcript_summary_type",
        "idx_transcript_summary_region",
    ];
    for index in required_indexes {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                [index],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err(format!("required index missing: {index}"));
        }
    }
    let has_rtree: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='gene_summary_rtree'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_rtree == 0 {
        return Err("required rtree table missing: gene_summary_rtree".to_string());
    }
    let has_transcript_table: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transcript_summary'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_transcript_table == 0 {
        return Err("required table missing: transcript_summary".to_string());
    }
    let schema_version = read_schema_version(&conn)?;
    if schema_version <= 0 {
        return Err("schema_version must be positive".to_string());
    }
    let analyzed: String = conn
        .query_row(
            "SELECT v FROM atlas_meta WHERE k='analyze_completed'",
            [],
            |r| r.get(0),
        )
        .map_err(|_| "atlas_meta.analyze_completed missing".to_string())?;
    if analyzed != "true" {
        return Err("ANALYZE required gate failed: analyze_completed != true".to_string());
    }
    Ok(())
}

fn read_schema_version(conn: &rusqlite::Connection) -> Result<i64, String> {
    let has_schema_table: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_schema_table > 0 {
        return conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string());
    }
    let legacy_schema_version: String = conn
        .query_row(
            "SELECT v FROM atlas_meta WHERE k='schema_version'",
            [],
            |r| r.get(0),
        )
        .map_err(|_| "atlas_meta.schema_version missing".to_string())?;
    legacy_schema_version
        .parse::<i64>()
        .map_err(|_| format!("invalid atlas_meta.schema_version: {legacy_schema_version}"))
}

fn validate_shard_catalog_and_indexes(derived_dir: &std::path::Path) -> Result<(), String> {
    let path = derived_dir.join("catalog_shards.json");
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let catalog: ShardCatalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    for shard in &catalog.shards {
        let shard_path = derived_dir.join(&shard.sqlite_path);
        validate_sqlite_contract(&shard_path)?;
        let bytes = fs::read(&shard_path).map_err(|e| e.to_string())?;
        let actual = sha256_hex(&bytes);
        if actual != shard.sqlite_sha256 {
            return Err(format!(
                "shard checksum mismatch for {}",
                shard_path.display()
            ));
        }
    }
    Ok(())
}

fn check_sha(path: &PathBuf, expected: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!(
            "checksum mismatch for {}: expected {}, got {}",
            path.display(),
            expected,
            actual
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
