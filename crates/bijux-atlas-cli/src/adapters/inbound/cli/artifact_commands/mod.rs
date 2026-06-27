// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::cli::OutputMode;
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::dataset::{
    parse_dataset_key, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ShardCatalog,
};
use bijux_atlas_query::ReleaseGeneIndex;
use bijux_atlas_store::{
    canonical_catalog_json, sorted_catalog_entries, verify_expected_sha256, ArtifactStore,
    LocalFsStore, ManifestLock, StoreErrorCode,
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder, Header};

mod catalog;
mod diff;
mod gc;

pub(crate) use catalog::{
    promote_catalog, publish_catalog, rollback_catalog, update_latest_alias, validate_catalog,
};
pub(crate) use diff::{build_release_diff, BuildReleaseDiffArgs};
pub(crate) use gc::{gc_apply, gc_plan};

mod dataset;

#[cfg(test)]
use dataset::validate_qc_thresholds;
pub(crate) use dataset::{
    pack_dataset, publish_dataset, validate_dataset, validate_dataset_evidence, validate_ingest_qc,
    verify_pack, PublishDatasetRequest,
};
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
mod artifact_contracts;
