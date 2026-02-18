use super::*;

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
    if !deep {
        validate_dataset_qc_thresholds(&root, &dataset)?;
    }
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

fn validate_dataset_qc_thresholds(root: &Path, dataset: &DatasetId) -> Result<(), String> {
    let workspace = std::env::current_dir().map_err(|e| e.to_string())?;
    let thresholds_path = workspace.join("configs/ops/dataset-qc-thresholds.json");
    let paths = bijux_atlas_model::artifact_paths(root, dataset);
    let qc_report = paths.derived_dir.join("qc.json");
    let qc_raw = fs::read_to_string(&qc_report)
        .map_err(|e| format!("dataset validate failed: {}: {e}", qc_report.display()))?;
    let thresholds_raw = fs::read_to_string(&thresholds_path).map_err(|e| {
        format!(
            "dataset validate failed: {}: {e}",
            thresholds_path.display()
        )
    })?;
    let qc: serde_json::Value = serde_json::from_str(&qc_raw)
        .map_err(|e| format!("dataset validate failed: invalid qc json: {e}"))?;
    let thresholds: serde_json::Value = serde_json::from_str(&thresholds_raw)
        .map_err(|e| format!("dataset validate failed: invalid thresholds json: {e}"))?;
    validate_qc_thresholds(&qc, &thresholds)
        .map_err(|e| format!("dataset validate failed: qc gate failed: {e}"))
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

pub(super) fn validate_qc_thresholds(
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
