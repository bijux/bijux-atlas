use crate::{sha256_hex, OutputMode};
use bijux_atlas_model::{ArtifactManifest, Catalog, DatasetId, ShardCatalog};
use bijux_atlas_policies::load_policy_from_workspace;
use bijux_atlas_store::{
    canonical_catalog_json, sorted_catalog_entries, verify_expected_sha256, ArtifactStore,
    LocalFsStore, ManifestLock, StoreErrorCode,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use tar::{Archive, Builder, Header};

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
    validate_shard_catalog_and_indexes(&paths.derived_dir)?;

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

pub(crate) fn publish_dataset(
    source_root: PathBuf,
    store_root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let source_paths = bijux_atlas_model::artifact_paths(&source_root, &dataset);
    let manifest_bytes = fs::read(&source_paths.manifest).map_err(|e| e.to_string())?;
    let sqlite_bytes = fs::read(&source_paths.sqlite).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|e| e.to_string())?;
    manifest.validate_strict().map_err(|e| e.to_string())?;
    verify_expected_sha256(&sqlite_bytes, &manifest.checksums.sqlite_sha256)?;
    enforce_publish_gates(&source_root, &dataset, &manifest)?;
    let manifest_sha = sha256_hex(&manifest_bytes);
    let sqlite_sha = sha256_hex(&sqlite_bytes);

    let store = LocalFsStore::new(store_root);
    match store.put_dataset(
        &dataset,
        &manifest_bytes,
        &sqlite_bytes,
        &manifest_sha,
        &sqlite_sha,
    ) {
        Ok(()) => emit_ok_payload(
            output_mode,
            json!({"command":"atlas dataset publish","status":"ok"}),
        ),
        Err(e) if e.code == StoreErrorCode::Conflict => {
            Err(format!("immutability gate rejected publish: {}", e.message))
        }
        Err(e) => Err(e.to_string()),
    }
}

fn enforce_publish_gates(
    source_root: &PathBuf,
    dataset: &DatasetId,
    manifest: &ArtifactManifest,
) -> Result<(), String> {
    let workspace = std::env::current_dir().map_err(|e| e.to_string())?;
    let policy = load_policy_from_workspace(&workspace).map_err(|e| e.to_string())?;
    if manifest.stats.gene_count < policy.publish_gates.min_gene_count {
        return Err(format!(
            "publish gate failed: gene_count {} < min_gene_count {}",
            manifest.stats.gene_count, policy.publish_gates.min_gene_count
        ));
    }
    let paths = bijux_atlas_model::artifact_paths(source_root, dataset);
    let anomaly_raw = fs::read_to_string(paths.anomaly_report).map_err(|e| e.to_string())?;
    let anomaly: bijux_atlas_model::IngestAnomalyReport =
        serde_json::from_str(&anomaly_raw).map_err(|e| e.to_string())?;
    if (anomaly.missing_parents.len() as u64) > policy.publish_gates.max_missing_parents {
        return Err(format!(
            "publish gate failed: missing_parents {} > max_missing_parents {}",
            anomaly.missing_parents.len(),
            policy.publish_gates.max_missing_parents
        ));
    }
    let conn = rusqlite::Connection::open(paths.sqlite).map_err(|e| e.to_string())?;
    for idx in &policy.publish_gates.required_indexes {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                [idx],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err(format!(
                "publish gate failed: required index missing: {idx}"
            ));
        }
    }
    Ok(())
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
