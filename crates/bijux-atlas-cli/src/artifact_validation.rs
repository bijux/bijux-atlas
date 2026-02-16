use crate::{sha256_hex, OutputMode};
use bijux_atlas_model::{ArtifactManifest, Catalog, DatasetId};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

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

pub(crate) fn validate_dataset(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::artifact_paths(&root, &dataset);

    let manifest_raw = fs::read_to_string(&paths.manifest).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    manifest.validate_strict().map_err(|e| e.to_string())?;

    check_sha(&paths.gff3, &manifest.checksums.gff3_sha256)?;
    check_sha(&paths.fasta, &manifest.checksums.fasta_sha256)?;
    check_sha(&paths.fai, &manifest.checksums.fai_sha256)?;
    check_sha(&paths.sqlite, &manifest.checksums.sqlite_sha256)?;

    let sqlite_bytes = fs::read(&paths.sqlite).map_err(|e| e.to_string())?;
    if !sqlite_bytes.starts_with(b"SQLite format 3\0") {
        return Err("sqlite artifact does not start with SQLite header".to_string());
    }

    if manifest.stats.gene_count == 0 {
        return Err("manifest gene_count must be > 0".to_string());
    }
    validate_sqlite_contract(&paths.sqlite)?;

    let payload = json!({"command":"atlas dataset validate","status":"ok"});
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
    let schema_version: String = conn
        .query_row(
            "SELECT v FROM atlas_meta WHERE k='schema_version'",
            [],
            |r| r.get(0),
        )
        .map_err(|_| "atlas_meta.schema_version missing".to_string())?;
    if schema_version.trim().is_empty() {
        return Err("atlas_meta.schema_version is empty".to_string());
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
