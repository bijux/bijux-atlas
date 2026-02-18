use crate::extract::ExtractResult;
use crate::IngestError;
use bijux_atlas_core::canonical;
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, DatasetId, ManifestInputHashes, ManifestStats, QcSeverity,
    ValidationError,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BuiltManifest {
    pub manifest: ArtifactManifest,
    pub qc_report_path: PathBuf,
}

pub struct BuildManifestArgs<'a> {
    pub output_root: &'a Path,
    pub dataset: &'a DatasetId,
    pub gff3_path: &'a Path,
    pub fasta_path: &'a Path,
    pub fai_path: &'a Path,
    pub sqlite_path: &'a Path,
    pub manifest_path: &'a Path,
    pub anomaly_path: &'a Path,
    pub extract: &'a ExtractResult,
    pub contig_aliases: &'a BTreeMap<String, String>,
}

pub fn build_and_write_manifest_and_reports(
    args: BuildManifestArgs<'_>,
) -> Result<BuiltManifest, IngestError> {
    let BuildManifestArgs {
        output_root,
        dataset,
        gff3_path,
        fasta_path,
        fai_path,
        sqlite_path,
        manifest_path,
        anomaly_path,
        extract,
        contig_aliases,
    } = args;
    let mut total_transcripts = 0_u64;
    let mut contigs = BTreeSet::new();
    for g in &extract.gene_rows {
        total_transcripts += g.transcript_count;
        contigs.insert(g.seqid.clone());
    }

    let mut manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        dataset.clone(),
        ArtifactChecksums::new(
            sha256_hex(&fs::read(gff3_path).map_err(|e| IngestError(e.to_string()))?),
            sha256_hex(&fs::read(fasta_path).map_err(|e| IngestError(e.to_string()))?),
            sha256_hex(&fs::read(fai_path).map_err(|e| IngestError(e.to_string()))?),
            sha256_hex(&fs::read(sqlite_path).map_err(|e| IngestError(e.to_string()))?),
        ),
        ManifestStats::new(
            extract.gene_rows.len() as u64,
            total_transcripts,
            contigs.len() as u64,
        ),
    );
    manifest.dataset_signature_sha256 = dataset_signature_merkle(extract)?;
    let policy_hash = sha256_hex(
        &fs::read(workspace_file("configs/policy/policy.json")).unwrap_or_default(),
    );
    manifest.input_hashes = ManifestInputHashes::new(
        manifest.checksums.gff3_sha256.clone(),
        manifest.checksums.fasta_sha256.clone(),
        manifest.checksums.fai_sha256.clone(),
        policy_hash,
    );
    manifest.ingest_toolchain = option_env!("RUSTUP_TOOLCHAIN")
        .unwrap_or("unknown")
        .to_string();
    manifest.ingest_build_hash = option_env!("BIJUX_BUILD_HASH").unwrap_or("dev").to_string();
    manifest.toolchain_hash = compute_toolchain_hash();
    manifest.contig_normalization_aliases = contig_aliases.clone();
    manifest.db_hash = manifest.checksums.sqlite_sha256.clone();
    manifest.artifact_hash = compute_manifest_artifact_hash(&manifest)?;

    manifest
        .validate_strict()
        .map_err(|e: ValidationError| IngestError(e.to_string()))?;

    let manifest_bytes =
        canonical::stable_json_bytes(&manifest).map_err(|e| IngestError(e.to_string()))?;
    fs::write(manifest_path, manifest_bytes).map_err(|e| IngestError(e.to_string()))?;

    let anomaly_bytes =
        canonical::stable_json_bytes(&extract.anomaly).map_err(|e| IngestError(e.to_string()))?;
    fs::write(anomaly_path, anomaly_bytes).map_err(|e| IngestError(e.to_string()))?;

    let warn_items = vec![
        ("missing_parents", extract.anomaly.missing_parents.len()),
        (
            "missing_transcript_parents",
            extract.anomaly.missing_transcript_parents.len(),
        ),
        (
            "multiple_parent_transcripts",
            extract.anomaly.multiple_parent_transcripts.len(),
        ),
        ("unknown_contigs", extract.anomaly.unknown_contigs.len()),
        ("overlapping_ids", extract.anomaly.overlapping_ids.len()),
        (
            "duplicate_gene_ids",
            extract.anomaly.duplicate_gene_ids.len(),
        ),
        (
            "overlapping_gene_ids_across_contigs",
            extract.anomaly.overlapping_gene_ids_across_contigs.len(),
        ),
        (
            "orphan_transcripts",
            extract.anomaly.orphan_transcripts.len(),
        ),
        ("parent_cycles", extract.anomaly.parent_cycles.len()),
        (
            "attribute_fallbacks",
            extract.anomaly.attribute_fallbacks.len(),
        ),
        (
            "unknown_feature_types",
            extract.anomaly.unknown_feature_types.len(),
        ),
        (
            "missing_required_fields",
            extract.anomaly.missing_required_fields.len(),
        ),
        ("rejections", extract.anomaly.rejections.len()),
    ];
    let warn_codes: Vec<serde_json::Value> = warn_items
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(code, count)| {
            json!({
                "severity": QcSeverity::Warn,
                "code": code,
                "count": count,
            })
        })
        .collect();

    let qc_report = json!({
        "dataset": dataset,
        "manifest_signature_sha256": manifest.dataset_signature_sha256,
        "gene_count": extract.gene_rows.len(),
        "transcript_count": total_transcripts,
        "transcript_summary_count": extract.transcript_rows.len(),
        "biotype_distribution": extract.biotype_distribution,
        "contig_distribution": extract.contig_distribution,
        "qc_counters": {
            "unknown_contig_feature_ratio": if extract.total_features == 0 { 0.0 } else { extract.unknown_contig_features as f64 / extract.total_features as f64 },
            "max_contig_name_length": extract.max_contig_name_length,
            "total_features": extract.total_features
        },
        "anomalies": {
            "missing_parents": extract.anomaly.missing_parents,
            "missing_transcript_parents": extract.anomaly.missing_transcript_parents,
            "multiple_parent_transcripts": extract.anomaly.multiple_parent_transcripts,
            "unknown_contigs": extract.anomaly.unknown_contigs,
            "overlapping_ids": extract.anomaly.overlapping_ids,
            "duplicate_gene_ids": extract.anomaly.duplicate_gene_ids,
            "overlapping_gene_ids_across_contigs": extract.anomaly.overlapping_gene_ids_across_contigs,
            "orphan_transcripts": extract.anomaly.orphan_transcripts,
            "parent_cycles": extract.anomaly.parent_cycles,
            "attribute_fallbacks": extract.anomaly.attribute_fallbacks,
            "unknown_feature_types": extract.anomaly.unknown_feature_types,
            "missing_required_fields": extract.anomaly.missing_required_fields,
            "rejections": extract.anomaly.rejections,
        },
        "severity_summary": {
            "INFO": 0,
            "WARN": warn_codes.len(),
            "ERROR": 0
        }
        ,
        "severity_items": warn_codes,
    });
    let qc_bytes =
        canonical::stable_json_bytes(&qc_report).map_err(|e| IngestError(e.to_string()))?;
    let qc_report_path = output_root
        .join(format!("release={}", dataset.release))
        .join(format!("species={}", dataset.species))
        .join(format!("assembly={}", dataset.assembly))
        .join("derived")
        .join("qc_report.json");
    fs::write(&qc_report_path, qc_bytes).map_err(|e| IngestError(e.to_string()))?;

    Ok(BuiltManifest {
        manifest,
        qc_report_path,
    })
}

fn workspace_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."))
        .join(rel)
}

fn compute_toolchain_hash() -> String {
    let mut bytes = Vec::new();
    for rel in ["rust-toolchain.toml", "Cargo.lock"] {
        let p = workspace_file(rel);
        if let Ok(b) = fs::read(p) {
            bytes.extend_from_slice(&b);
        }
    }
    if bytes.is_empty() {
        "unknown".to_string()
    } else {
        sha256_hex(&bytes)
    }
}

fn compute_manifest_artifact_hash(manifest: &ArtifactManifest) -> Result<String, IngestError> {
    let digest_source = serde_json::json!({
        "artifact_version": manifest.artifact_version,
        "schema_version": manifest.schema_version,
        "db_schema_version": manifest.db_schema_version,
        "dataset": manifest.dataset,
        "checksums": manifest.checksums,
        "input_hashes": manifest.input_hashes,
        "stats": manifest.stats,
        "dataset_signature_sha256": manifest.dataset_signature_sha256,
        "toolchain_hash": manifest.toolchain_hash,
        "db_hash": manifest.db_hash
    });
    let bytes = canonical::stable_json_bytes(&digest_source).map_err(|e| IngestError(e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

pub fn write_qc_and_anomaly_reports_only(
    output_root: &Path,
    dataset: &DatasetId,
    anomaly_path: &Path,
    extract: &ExtractResult,
) -> Result<PathBuf, IngestError> {
    let mut total_transcripts = 0_u64;
    let mut contigs = BTreeSet::new();
    for g in &extract.gene_rows {
        total_transcripts += g.transcript_count;
        contigs.insert(g.seqid.clone());
    }

    let anomaly_bytes =
        canonical::stable_json_bytes(&extract.anomaly).map_err(|e| IngestError(e.to_string()))?;
    fs::write(anomaly_path, anomaly_bytes).map_err(|e| IngestError(e.to_string()))?;

    let warn_items = vec![
        ("missing_parents", extract.anomaly.missing_parents.len()),
        (
            "missing_transcript_parents",
            extract.anomaly.missing_transcript_parents.len(),
        ),
        (
            "multiple_parent_transcripts",
            extract.anomaly.multiple_parent_transcripts.len(),
        ),
        ("unknown_contigs", extract.anomaly.unknown_contigs.len()),
        ("overlapping_ids", extract.anomaly.overlapping_ids.len()),
        (
            "duplicate_gene_ids",
            extract.anomaly.duplicate_gene_ids.len(),
        ),
        (
            "overlapping_gene_ids_across_contigs",
            extract.anomaly.overlapping_gene_ids_across_contigs.len(),
        ),
        (
            "orphan_transcripts",
            extract.anomaly.orphan_transcripts.len(),
        ),
        ("parent_cycles", extract.anomaly.parent_cycles.len()),
        (
            "attribute_fallbacks",
            extract.anomaly.attribute_fallbacks.len(),
        ),
        (
            "unknown_feature_types",
            extract.anomaly.unknown_feature_types.len(),
        ),
        (
            "missing_required_fields",
            extract.anomaly.missing_required_fields.len(),
        ),
        ("rejections", extract.anomaly.rejections.len()),
    ];
    let warn_codes: Vec<serde_json::Value> = warn_items
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(code, count)| {
            json!({
                "severity": QcSeverity::Warn,
                "code": code,
                "count": count,
            })
        })
        .collect();

    let qc_report = json!({
        "dataset": dataset,
        "report_only": true,
        "manifest_signature_sha256": dataset_signature_merkle(extract)?,
        "gene_count": extract.gene_rows.len(),
        "transcript_count": total_transcripts,
        "transcript_summary_count": extract.transcript_rows.len(),
        "contig_count": contigs.len(),
        "biotype_distribution": extract.biotype_distribution,
        "contig_distribution": extract.contig_distribution,
        "qc_counters": {
            "unknown_contig_feature_ratio": if extract.total_features == 0 { 0.0 } else { extract.unknown_contig_features as f64 / extract.total_features as f64 },
            "max_contig_name_length": extract.max_contig_name_length,
            "total_features": extract.total_features
        },
        "anomalies": {
            "missing_parents": extract.anomaly.missing_parents,
            "missing_transcript_parents": extract.anomaly.missing_transcript_parents,
            "multiple_parent_transcripts": extract.anomaly.multiple_parent_transcripts,
            "unknown_contigs": extract.anomaly.unknown_contigs,
            "overlapping_ids": extract.anomaly.overlapping_ids,
            "duplicate_gene_ids": extract.anomaly.duplicate_gene_ids,
            "overlapping_gene_ids_across_contigs": extract.anomaly.overlapping_gene_ids_across_contigs,
            "orphan_transcripts": extract.anomaly.orphan_transcripts,
            "parent_cycles": extract.anomaly.parent_cycles,
            "attribute_fallbacks": extract.anomaly.attribute_fallbacks,
            "unknown_feature_types": extract.anomaly.unknown_feature_types,
            "missing_required_fields": extract.anomaly.missing_required_fields,
            "rejections": extract.anomaly.rejections,
        },
        "severity_summary": {
            "INFO": 0,
            "WARN": warn_codes.len(),
            "ERROR": 0
        },
        "severity_items": warn_codes,
    });
    let qc_bytes =
        canonical::stable_json_bytes(&qc_report).map_err(|e| IngestError(e.to_string()))?;
    let qc_report_path = output_root
        .join(format!("release={}", dataset.release))
        .join(format!("species={}", dataset.species))
        .join(format!("assembly={}", dataset.assembly))
        .join("derived")
        .join("qc_report.json");
    fs::write(&qc_report_path, qc_bytes).map_err(|e| IngestError(e.to_string()))?;
    Ok(qc_report_path)
}

fn dataset_signature_merkle(extract: &ExtractResult) -> Result<String, IngestError> {
    let gene_hashes: Vec<String> = extract
        .gene_rows
        .iter()
        .map(|row| canonical::stable_json_bytes(row).map(|b| sha256_hex(&b)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IngestError(e.to_string()))?;
    let tx_hashes: Vec<String> = extract
        .transcript_rows
        .iter()
        .map(|row| canonical::stable_json_bytes(row).map(|b| sha256_hex(&b)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IngestError(e.to_string()))?;
    let root_payload = json!({
        "gene_table_hash": merkle_root(&gene_hashes),
        "transcript_table_hash": merkle_root(&tx_hashes),
        "gene_count": extract.gene_rows.len(),
        "transcript_count": extract.transcript_rows.len(),
    });
    let bytes =
        canonical::stable_json_bytes(&root_payload).map_err(|e| IngestError(e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

fn merkle_root(leaves: &[String]) -> String {
    if leaves.is_empty() {
        return sha256_hex(b"");
    }
    let mut level = leaves.to_vec();
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
    level[0].clone()
}
