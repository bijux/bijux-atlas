// SPDX-License-Identifier: Apache-2.0

use super::super::{output, OutputMode};
use bijux_atlas_model::dataset::{ArtifactManifest, DatasetId};
use bijux_atlas_query::{query_genes, QueryLimits};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

pub(crate) fn inspect_db(
    db: PathBuf,
    sample_rows: usize,
    output_mode: OutputMode,
) -> Result<(), String> {
    let conn = Connection::open(db).map_err(|e| e.to_string())?;
    let schema_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut idx_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|e| e.to_string())?;
    let indexes = idx_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let sql = format!(
        "SELECT gene_id, name, seqid, start, end FROM gene_summary ORDER BY seqid, start, gene_id LIMIT {}",
        sample_rows
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas inspect db",
            "schema_version": schema_version,
            "indexes": indexes,
            "gene_count": count,
            "sample_rows": rows
        }),
    )?;
    Ok(())
}

pub(crate) fn inspect_dataset(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::dataset::artifact_paths(&root, &dataset);
    let manifest_raw = fs::read_to_string(&paths.manifest).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas inspect dataset",
            "dataset": dataset.canonical_string(),
            "paths": {
                "manifest": paths.manifest,
                "sqlite": paths.sqlite,
                "derived_dir": paths.derived_dir,
            },
            "stats": manifest.stats,
            "identity": manifest.identity,
            "canonical_feature_counts": manifest.canonical_feature_counts,
        }),
    )?;
    Ok(())
}

pub(crate) fn inspect_provenance(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::dataset::artifact_paths(&root, &dataset);
    let manifest_raw = fs::read_to_string(&paths.manifest).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    let source_facts: Value =
        serde_json::from_str(&fs::read_to_string(&paths.source_facts).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let build_metadata: Value = serde_json::from_str(
        &fs::read_to_string(&paths.build_metadata).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let artifact_inventory: Value = serde_json::from_str(
        &fs::read_to_string(&paths.artifact_inventory).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let scientific_profile: Value = serde_json::from_str(
        &fs::read_to_string(&paths.scientific_profile).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas inspect provenance",
            "dataset": dataset.canonical_string(),
            "build_evidence": {
                "manifest_identity": manifest.identity,
                "input_hashes": manifest.input_hashes,
                "normalized_input_identity_sha256": manifest.normalized_input_identity_sha256,
                "build_policy_version": manifest.build_policy_version,
                "reference_build_identity_sha256": manifest.reference_build_identity_sha256,
                "contig_naming_style": manifest.contig_naming_style,
                "scientific_prerequisites_status": manifest.scientific_prerequisites_status
            },
            "runtime_api_evidence": {
                "carrier": "http provenance envelope",
                "fields": ["dataset_hash", "release", "species", "assembly", "manifest_version", "db_schema_version", "dataset_signature_sha256"]
            },
            "source_facts": source_facts,
            "build_metadata": build_metadata,
            "artifact_inventory": artifact_inventory,
            "scientific_profile": scientific_profile,
        }),
    )?;
    Ok(())
}

pub(crate) fn smoke_dataset(
    root: PathBuf,
    dataset: &str,
    golden_queries: PathBuf,
    write_snapshot: bool,
    snapshot_out: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let (release, species, assembly) = output::parse_dataset_id(dataset)?;
    let id = DatasetId::new(&release, &species, &assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::dataset::artifact_paths(&root, &id);
    let conn = Connection::open(&paths.sqlite).map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count <= 0 {
        return Err("smoke failed: gene_summary is empty".to_string());
    }

    let raw = fs::read_to_string(golden_queries).map_err(|e| e.to_string())?;
    let queries: Vec<Value> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let mut out = Vec::new();

    for q in queries {
        let name = q
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "golden query missing name".to_string())?;
        let body = q
            .get("query")
            .ok_or_else(|| "golden query missing query object".to_string())?;
        let req = output::query_request_from_json(body)?;
        let resp = query_genes(&conn, &req, &QueryLimits::default(), b"smoke")
            .map_err(|e| e.to_string())?;
        if resp.rows.is_empty() && name == "by_gene_id" {
            return Err("smoke failed: by_gene_id returned zero rows".to_string());
        }
        out.push(serde_json::json!({
            "name": name,
            "row_count": resp.rows.len(),
            "next_cursor": resp.next_cursor,
        }));
    }

    if write_snapshot {
        let payload = serde_json::json!({ "dataset": dataset, "queries": out });
        fs::write(
            snapshot_out,
            serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }

    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas smoke",
            "status":"ok",
            "dataset": dataset,
            "queries": out.len()
        }),
    )?;
    Ok(())
}
