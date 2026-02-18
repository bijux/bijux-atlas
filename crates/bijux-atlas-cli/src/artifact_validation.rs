use crate::{sha256_hex, OutputMode};
use bijux_atlas_core::canonical;
use bijux_atlas_model::{ArtifactManifest, Catalog, DatasetId, ShardCatalog};
use bijux_atlas_policies::{canonical_config_json, load_policy_from_workspace};
use bijux_atlas_store::{
    canonical_catalog_json, sorted_catalog_entries, verify_expected_sha256, ArtifactStore,
    LocalFsStore, ManifestLock, StoreErrorCode,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
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

pub(crate) fn validate_policy(output_mode: OutputMode) -> Result<(), String> {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
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
    deep: bool,
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
    if deep {
        let lock_path = paths.derived_dir.join("manifest.lock");
        let lock_raw = fs::read(&lock_path)
            .map_err(|_| format!("manifest.lock missing: {}", lock_path.display()))?;
        let lock: ManifestLock = serde_json::from_slice(&lock_raw).map_err(|e| e.to_string())?;
        lock.validate(manifest_raw.as_bytes(), &sqlite_bytes)?;

        let actual_signature = compute_dataset_signature_from_sqlite(&paths.sqlite)?;
        if manifest.dataset_signature_sha256.is_empty() {
            return Err(
                "manifest dataset_signature_sha256 is empty; cannot deep-verify".to_string(),
            );
        }
        if actual_signature != manifest.dataset_signature_sha256 {
            return Err(format!(
                "dataset signature mismatch: manifest={} actual={}",
                manifest.dataset_signature_sha256, actual_signature
            ));
        }
        if manifest.derived_column_origins.is_empty() {
            return Err("manifest derived_column_origins must not be empty".to_string());
        }
        enforce_publish_gates(&root, &dataset, &manifest)?;
    }

    let command_name = if deep {
        "atlas dataset verify"
    } else {
        "atlas dataset validate"
    };
    let payload = json!({"command":command_name,"status":"ok","deep":deep});
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

pub(crate) fn validate_ingest_qc(
    qc_report: PathBuf,
    thresholds: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let qc_raw = fs::read_to_string(&qc_report)
        .map_err(|e| format!("failed to read {}: {e}", qc_report.display()))?;
    let thresholds_raw = fs::read_to_string(&thresholds)
        .map_err(|e| format!("failed to read {}: {e}", thresholds.display()))?;
    let qc: serde_json::Value = serde_json::from_str(&qc_raw)
        .map_err(|e| format!("invalid QC json {}: {e}", qc_report.display()))?;
    let t: serde_json::Value = serde_json::from_str(&thresholds_raw)
        .map_err(|e| format!("invalid thresholds json {}: {e}", thresholds.display()))?;
    validate_qc_thresholds(&qc, &t)?;
    emit_ok_payload(
        output_mode,
        json!({
            "command":"atlas ingest-validate",
            "status":"ok",
            "qc_report": qc_report,
            "thresholds": thresholds
        }),
    )
}

fn compute_dataset_signature_from_sqlite(sqlite_path: &PathBuf) -> Result<String, String> {
    let conn = rusqlite::Connection::open(sqlite_path).map_err(|e| e.to_string())?;
    let mut gene_stmt = conn
        .prepare(
            "SELECT gene_id, name, biotype, seqid, start, end, transcript_count, exon_count, total_exon_span, cds_present, sequence_length
             FROM gene_summary ORDER BY seqid, start, end, gene_id",
        )
        .map_err(|e| e.to_string())?;
    let genes = gene_stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "gene_id": r.get::<_, String>(0)?,
                "gene_name": r.get::<_, String>(1)?,
                "biotype": r.get::<_, String>(2)?,
                "seqid": r.get::<_, String>(3)?,
                "start": r.get::<_, i64>(4)?,
                "end": r.get::<_, i64>(5)?,
                "transcript_count": r.get::<_, i64>(6)?,
                "exon_count": r.get::<_, i64>(7)?,
                "total_exon_span": r.get::<_, i64>(8)?,
                "cds_present": r.get::<_, i64>(9)? != 0,
                "sequence_length": r.get::<_, i64>(10)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut tx_stmt = conn
        .prepare(
            "SELECT transcript_id, parent_gene_id, transcript_type, COALESCE(biotype,''), seqid, start, end, exon_count, total_exon_span, cds_present
             FROM transcript_summary ORDER BY seqid, start, end, transcript_id",
        )
        .map_err(|e| e.to_string())?;
    let txs = tx_stmt
        .query_map([], |r| {
            let raw_biotype: String = r.get(3)?;
            let biotype = if raw_biotype.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(raw_biotype)
            };
            Ok(serde_json::json!({
                "transcript_id": r.get::<_, String>(0)?,
                "parent_gene_id": r.get::<_, String>(1)?,
                "transcript_type": r.get::<_, String>(2)?,
                "biotype": biotype,
                "seqid": r.get::<_, String>(4)?,
                "start": r.get::<_, i64>(5)?,
                "end": r.get::<_, i64>(6)?,
                "exon_count": r.get::<_, i64>(7)?,
                "total_exon_span": r.get::<_, i64>(8)?,
                "cds_present": r.get::<_, i64>(9)? != 0,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let root = serde_json::json!({
        "gene_table_hash": merkle_from_json_rows(&genes)?,
        "transcript_table_hash": merkle_from_json_rows(&txs)?,
        "gene_count": genes.len(),
        "transcript_count": txs.len(),
    });
    let bytes = canonical::stable_json_bytes(&root).map_err(|e| e.to_string())?;
    Ok(sha256_hex(&bytes))
}

fn merkle_from_json_rows(rows: &[serde_json::Value]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(sha256_hex(b""));
    }
    let mut level: Vec<String> = rows
        .iter()
        .map(|r| canonical::stable_json_bytes(r).map(|b| sha256_hex(&b)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        let mut i = 0usize;
        while i < level.len() {
            let left = &level[i];
            let right = if i + 1 < level.len() {
                &level[i + 1]
            } else {
                left
            };
            let mut joined = String::with_capacity(left.len() + right.len());
            joined.push_str(left);
            joined.push_str(right);
            next.push(sha256_hex(joined.as_bytes()));
            i += 2;
        }
        level = next;
    }
    Ok(level[0].clone())
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
    source_root: &Path,
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
    let qc_report = paths.derived_dir.join("qc.json");
    let thresholds_path = workspace.join("configs/ops/dataset-qc-thresholds.json");
    let qc_raw = fs::read_to_string(&qc_report)
        .map_err(|e| format!("publish gate failed: {}: {e}", qc_report.display()))?;
    let thresholds_raw = fs::read_to_string(&thresholds_path)
        .map_err(|e| format!("publish gate failed: {}: {e}", thresholds_path.display()))?;
    let qc: serde_json::Value = serde_json::from_str(&qc_raw).map_err(|e| {
        format!(
            "publish gate failed: invalid qc json {}: {e}",
            qc_report.display()
        )
    })?;
    let thresholds: serde_json::Value = serde_json::from_str(&thresholds_raw).map_err(|e| {
        format!(
            "publish gate failed: invalid thresholds json {}: {e}",
            thresholds_path.display()
        )
    })?;
    validate_qc_thresholds(&qc, &thresholds).map_err(|e| format!("publish gate failed: {e}"))?;
    Ok(())
}

fn validate_qc_thresholds(
    qc: &serde_json::Value,
    thresholds: &serde_json::Value,
) -> Result<(), String> {
    let min_gene_count = thresholds
        .get("min_gene_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "threshold missing min_gene_count".to_string())?;
    let max_orphan_pct = thresholds
        .get("max_orphan_percent")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "threshold missing max_orphan_percent".to_string())?;
    let max_rejected_pct = thresholds
        .get("max_rejected_percent")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "threshold missing max_rejected_percent".to_string())?;
    let max_unknown_contig_pct = thresholds
        .get("max_unknown_contig_feature_percent")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "threshold missing max_unknown_contig_feature_percent".to_string())?;
    let max_duplicate_gene_id_events = thresholds
        .get("max_duplicate_gene_id_events")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "threshold missing max_duplicate_gene_id_events".to_string())?;
    let genes = qc
        .pointer("/counts/genes")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing counts.genes".to_string())?;
    let transcripts = qc
        .pointer("/counts/transcripts")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing counts.transcripts".to_string())?;
    let orphan_transcripts = qc
        .pointer("/orphan_counts/transcripts")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing orphan_counts.transcripts".to_string())?;
    let duplicate_gene_ids = qc
        .pointer("/duplicate_id_events/duplicate_gene_ids")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing duplicate_id_events.duplicate_gene_ids".to_string())?;
    let unknown_contig_ratio = qc
        .pointer("/contig_stats/unknown_contig_feature_ratio")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| "qc missing contig_stats.unknown_contig_feature_ratio".to_string())?;
    let rejected: u64 = qc
        .get("rejected_record_count_by_reason")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "qc missing rejected_record_count_by_reason".to_string())?
        .values()
        .map(|v| v.as_u64().unwrap_or(0))
        .sum();
    if genes < min_gene_count {
        return Err(format!(
            "gene_count {} < min_gene_count {}",
            genes, min_gene_count
        ));
    }
    let orphan_pct = if transcripts == 0 {
        0.0
    } else {
        (orphan_transcripts as f64) * 100.0 / (transcripts as f64)
    };
    if orphan_pct > max_orphan_pct {
        return Err(format!(
            "orphan_percent {:.4} > max_orphan_percent {}",
            orphan_pct, max_orphan_pct
        ));
    }
    let total_features = qc
        .pointer("/contig_stats/total_features")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "qc missing contig_stats.total_features".to_string())?;
    let rejected_pct = if total_features == 0 {
        0.0
    } else {
        (rejected as f64) * 100.0 / (total_features as f64)
    };
    if rejected_pct > max_rejected_pct {
        return Err(format!(
            "rejected_percent {:.4} > max_rejected_percent {}",
            rejected_pct, max_rejected_pct
        ));
    }
    if unknown_contig_ratio * 100.0 > max_unknown_contig_pct {
        return Err(format!(
            "unknown_contig_feature_percent {:.4} > max_unknown_contig_feature_percent {}",
            unknown_contig_ratio * 100.0,
            max_unknown_contig_pct
        ));
    }
    if duplicate_gene_ids > max_duplicate_gene_id_events {
        return Err(format!(
            "duplicate_gene_id_events {} > max_duplicate_gene_id_events {}",
            duplicate_gene_ids, max_duplicate_gene_id_events
        ));
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
mod tests {
    use super::validate_qc_thresholds;
    use serde_json::json;

    #[test]
    fn qc_thresholds_pass_for_healthy_report() {
        let qc = json!({
            "counts": {"genes": 10, "transcripts": 20, "exons": 50, "cds": 12},
            "orphan_counts": {"transcripts": 0},
            "duplicate_id_events": {"duplicate_gene_ids": 0},
            "rejected_record_count_by_reason": {"GFF3_UNKNOWN_FEATURE": 0},
            "contig_stats": {"unknown_contig_feature_ratio": 0.0, "total_features": 100}
        });
        let t = json!({
            "min_gene_count": 1,
            "max_orphan_percent": 1.0,
            "max_rejected_percent": 1.0,
            "max_unknown_contig_feature_percent": 0.5,
            "max_duplicate_gene_id_events": 0
        });
        assert!(validate_qc_thresholds(&qc, &t).is_ok());
    }

    #[test]
    fn qc_thresholds_fail_when_orphan_rate_exceeds_max() {
        let qc = json!({
            "counts": {"genes": 10, "transcripts": 10, "exons": 10, "cds": 10},
            "orphan_counts": {"transcripts": 2},
            "duplicate_id_events": {"duplicate_gene_ids": 0},
            "rejected_record_count_by_reason": {},
            "contig_stats": {"unknown_contig_feature_ratio": 0.0, "total_features": 100}
        });
        let t = json!({
            "min_gene_count": 1,
            "max_orphan_percent": 10.0,
            "max_rejected_percent": 10.0,
            "max_unknown_contig_feature_percent": 10.0,
            "max_duplicate_gene_id_events": 0
        });
        let err = validate_qc_thresholds(&qc, &t).expect_err("orphan gate must fail");
        assert!(err.contains("orphan_percent"));
    }

    #[test]
    fn qc_edgecase_fixture_orphan_rate_regression_is_rejected() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let qc = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(root.join("tests/fixtures/qc_edgecases/qc_orphan_high.json"))
                .expect("read qc fixture"),
        )
        .expect("parse qc fixture");
        let t = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(root.join("tests/fixtures/qc_edgecases/thresholds_strict.json"))
                .expect("read threshold fixture"),
        )
        .expect("parse threshold fixture");
        assert!(validate_qc_thresholds(&qc, &t).is_err());
    }
}
