// SPDX-License-Identifier: Apache-2.0

use super::extract::ExtractResult;
use super::IngestError;
use crate::domain::canonical;
use crate::domain::dataset::{
    ArtifactChecksums, ArtifactManifest, DatasetId, ManifestInputHashes, ManifestStats,
    QcSeverity, ShardingPlan, ValidationError,
};
use crate::domain::sha256_hex;
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
    pub sharding_plan: ShardingPlan,
    pub canonical_model_schema_version: u64,
    pub canonical_query_semantic_sha256: &'a str,
    pub canonical_lineage_sha256: &'a str,
    pub canonical_feature_counts: &'a BTreeMap<String, u64>,
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
        sharding_plan,
        canonical_model_schema_version,
        canonical_query_semantic_sha256,
        canonical_lineage_sha256,
        canonical_feature_counts,
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
        &fs::read(workspace_file(
            "configs/sources/governance/policy/policy.json",
        ))
        .unwrap_or_default(),
    );
    manifest.input_hashes = ManifestInputHashes::new(
        manifest.checksums.gff3_sha256.clone(),
        manifest.checksums.fasta_sha256.clone(),
        manifest.checksums.fai_sha256.clone(),
        policy_hash.clone(),
    );
    manifest.source_gff3_filename = gff3_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    manifest.source_fasta_filename = fasta_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    manifest.source_fai_filename = fai_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    manifest.source_facts_path = "derived/source_facts.json".to_string();
    manifest.normalized_input_identity_sha256 =
        compute_normalized_input_identity_sha256(&manifest, contig_aliases)?;
    manifest.software_version = crate::version::runtime_version().to_string();
    manifest.config_version = compute_config_version();
    manifest.build_policy_version = format!("sha256:{policy_hash}");
    manifest.build_metadata_path = "derived/build_metadata.json".to_string();
    manifest.anomaly_summary_path = "derived/anomaly_summary.json".to_string();
    manifest.dataset_stats_path = "derived/dataset_stats.json".to_string();
    manifest.artifact_inventory_path = "derived/artifact_inventory.json".to_string();
    manifest.evidence_bundle_path = "derived/evidence_bundle.lock.json".to_string();
    manifest.evidence_bundle_sha256 = "pending".to_string();
    manifest.ingest_toolchain = option_env!("RUSTUP_TOOLCHAIN")
        .unwrap_or("unknown")
        .to_string();
    manifest.ingest_build_hash = crate::runtime::config::runtime_build_hash().to_string();
    manifest.toolchain_hash = compute_toolchain_hash();
    manifest.contig_normalization_aliases = contig_aliases.clone();
    manifest.sharding_plan = sharding_plan;
    manifest.canonical_feature_summary_path = "derived/canonical_summary.json".to_string();
    manifest.canonical_model_schema_version = canonical_model_schema_version;
    manifest.canonical_query_semantic_sha256 = canonical_query_semantic_sha256.to_string();
    manifest.canonical_lineage_sha256 = canonical_lineage_sha256.to_string();
    manifest.canonical_feature_counts = canonical_feature_counts.clone();
    manifest.db_hash = manifest.checksums.sqlite_sha256.clone();
    manifest.artifact_hash = compute_manifest_artifact_hash(&manifest)?;
    manifest.identity = crate::domain::dataset::DatasetIdentity::from_components(
        &manifest.dataset,
        &serde_json::json!({
            "gff3_sha256": manifest.checksums.gff3_sha256.clone(),
            "fasta_sha256": manifest.checksums.fasta_sha256.clone(),
            "fai_sha256": manifest.checksums.fai_sha256.clone()
        }),
        &serde_json::json!({
            "manifest_version": manifest.manifest_version.clone(),
            "schema_version": manifest.schema_version.clone(),
            "db_schema_version": manifest.db_schema_version.clone()
        }),
        &serde_json::json!({
            "sqlite_sha256": manifest.checksums.sqlite_sha256.clone()
        }),
    )
    .map_err(|e| IngestError(e.to_string()))?;

    manifest
        .validate_strict()
        .map_err(|e: ValidationError| IngestError(e.to_string()))?;

    let anomaly_bytes =
        canonical::stable_json_bytes(&extract.anomaly).map_err(|e| IngestError(e.to_string()))?;
    fs::write(anomaly_path, anomaly_bytes).map_err(|e| IngestError(e.to_string()))?;

    let qc_report = build_qc_report_json(
        dataset,
        extract,
        total_transcripts,
        Some(manifest.dataset_signature_sha256.clone()),
        Some(&json!({
            "schema_version": canonical_model_schema_version,
            "query_semantic_sha256": canonical_query_semantic_sha256,
            "lineage_sensitive_sha256": canonical_lineage_sha256,
            "feature_counts": canonical_feature_counts
        })),
        false,
    )?;
    let qc_bytes =
        canonical::stable_json_bytes(&qc_report).map_err(|e| IngestError(e.to_string()))?;
    let derived_dir = output_root
        .join(format!("release={}", dataset.release))
        .join(format!("species={}", dataset.species))
        .join(format!("assembly={}", dataset.assembly))
        .join("derived");
    let qc_report_path = derived_dir.join("qc_report.json");
    fs::write(&qc_report_path, &qc_bytes).map_err(|e| IngestError(e.to_string()))?;
    fs::write(derived_dir.join("qc.json"), qc_bytes).map_err(|e| IngestError(e.to_string()))?;
    manifest.qc_report_path = "derived/qc_report.json".to_string();

    let manifest_bytes =
        canonical::stable_json_bytes(&manifest).map_err(|e| IngestError(e.to_string()))?;
    fs::write(manifest_path, manifest_bytes).map_err(|e| IngestError(e.to_string()))?;

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

fn compute_config_version() -> String {
    let mut bytes = Vec::new();
    for rel in [
        "configs/sources/operations/ops/dataset-qc-thresholds.v1.json",
        "configs/sources/governance/policy/policy.json",
    ] {
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

fn compute_normalized_input_identity_sha256(
    manifest: &ArtifactManifest,
    contig_aliases: &BTreeMap<String, String>,
) -> Result<String, IngestError> {
    let payload = json!({
        "gff3_sha256": manifest.checksums.gff3_sha256.clone(),
        "fasta_sha256": manifest.checksums.fasta_sha256.clone(),
        "fai_sha256": manifest.checksums.fai_sha256.clone(),
        "source_filenames": {
            "gff3": manifest.source_gff3_filename.clone(),
            "fasta": manifest.source_fasta_filename.clone(),
            "fai": manifest.source_fai_filename.clone()
        },
        "contig_normalization_aliases": contig_aliases
    });
    let bytes = canonical::stable_json_bytes(&payload).map_err(|e| IngestError(e.to_string()))?;
    Ok(sha256_hex(&bytes))
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
        "canonical_model_schema_version": manifest.canonical_model_schema_version,
        "canonical_query_semantic_sha256": manifest.canonical_query_semantic_sha256,
        "canonical_lineage_sha256": manifest.canonical_lineage_sha256,
        "canonical_feature_counts": manifest.canonical_feature_counts,
        "toolchain_hash": manifest.toolchain_hash,
        "db_hash": manifest.db_hash
    });
    let bytes =
        canonical::stable_json_bytes(&digest_source).map_err(|e| IngestError(e.to_string()))?;
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

    let qc_report = build_qc_report_json(
        dataset,
        extract,
        total_transcripts,
        Some(dataset_signature_merkle(extract)?),
        None,
        true,
    )?;
    let qc_bytes =
        canonical::stable_json_bytes(&qc_report).map_err(|e| IngestError(e.to_string()))?;
    let derived_dir = output_root
        .join(format!("release={}", dataset.release))
        .join(format!("species={}", dataset.species))
        .join(format!("assembly={}", dataset.assembly))
        .join("derived");
    let qc_report_path = derived_dir.join("qc_report.json");
    fs::write(&qc_report_path, &qc_bytes).map_err(|e| IngestError(e.to_string()))?;
    fs::write(derived_dir.join("qc.json"), qc_bytes).map_err(|e| IngestError(e.to_string()))?;
    Ok(qc_report_path)
}

fn build_qc_report_json(
    dataset: &DatasetId,
    extract: &ExtractResult,
    total_transcripts: u64,
    manifest_signature: Option<String>,
    canonical_summary: Option<&serde_json::Value>,
    report_only: bool,
) -> Result<serde_json::Value, IngestError> {
    let class_counts = extract.anomaly.anomaly_class_counts();
    let class_items: Vec<serde_json::Value> = class_counts
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(class, count)| {
            let severity = crate::domain::dataset::IngestAnomalyReport::severity_for_class(*class);
            json!({
                "severity": severity,
                "class": class,
                "count": count,
            })
        })
        .collect();
    let warn_count: usize = class_items
        .iter()
        .filter(|v| v.get("severity") == Some(&json!(QcSeverity::Warn)))
        .count();
    let error_count: usize = class_items
        .iter()
        .filter(|v| v.get("severity") == Some(&json!(QcSeverity::Error)))
        .count();
    let mut rejections_by_reason = BTreeMap::<String, u64>::new();
    for rejection in &extract.anomaly.rejections {
        *rejections_by_reason
            .entry(rejection.code.clone())
            .or_insert(0) += 1;
    }
    let mut top_biotypes = extract
        .biotype_distribution
        .iter()
        .map(|(biotype, count)| (biotype.clone(), *count))
        .collect::<Vec<_>>();
    top_biotypes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top_biotypes.truncate(10);
    Ok(json!({
        "schema_version": 1,
        "dataset": dataset,
        "report_only": report_only,
        "manifest_signature_sha256": manifest_signature.unwrap_or_default(),
        "counts": {
            "genes": extract.gene_rows.len(),
            "transcripts": total_transcripts,
            "exons": extract.exon_rows.len(),
            "cds": extract.cds_feature_count
        },
        "orphan_counts": {
            "transcripts": extract.anomaly.orphan_transcripts.len()
        },
        "duplicate_id_events": {
            "overlapping_ids": extract.anomaly.overlapping_ids.len(),
            "duplicate_gene_ids": extract.anomaly.duplicate_gene_ids.len(),
            "overlapping_gene_ids_across_contigs": extract.anomaly.overlapping_gene_ids_across_contigs.len()
        },
        "rejected_record_count_by_reason": rejections_by_reason,
        "contig_stats": {
            "distribution": extract.contig_distribution,
            "unknown_contig_feature_ratio": if extract.total_features == 0 { 0.0 } else { extract.unknown_contig_features as f64 / extract.total_features as f64 },
            "max_contig_name_length": extract.max_contig_name_length,
            "total_features": extract.total_features
        },
        "biotype_distribution_top_n": top_biotypes,
        "biotype_distribution": extract.biotype_distribution,
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
            "WARN": warn_count,
            "ERROR": error_count
        },
        "severity_items": class_items,
        "anomaly_classes": class_counts,
        "canonical": canonical_summary.cloned().unwrap_or_else(|| json!({})),
    }))
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
