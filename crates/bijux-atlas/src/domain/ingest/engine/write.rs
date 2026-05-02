// SPDX-License-Identifier: Apache-2.0

use std::fs;

use crate::domain::canonical;
use crate::domain::dataset::{
    ArtifactChecksums, ArtifactManifest, IngestAnomalyReport, ManifestStats, ShardCatalog,
    ShardingPlan,
};
use crate::domain::sha256_hex;
use serde_json::json;

use super::decode::DecodedIngest;
use super::diff_index::build_and_write_release_gene_index;
use super::hashing::compute_input_hashes;
use super::job::IngestJob;
use super::manifest::{
    build_and_write_manifest_and_reports, write_qc_and_anomaly_reports_only, BuildManifestArgs,
};
use super::normalized::{replay_counts_from_normalized, write_normalized_jsonl_zst};
use super::sqlite::{write_sharded_sqlite_catalog, write_sqlite, WriteSqliteInput};
use super::{IngestError, IngestResult};

pub fn write_ingest_outputs(
    job: &IngestJob,
    decoded: DecodedIngest,
) -> Result<IngestResult, IngestError> {
    let opts = &job.options;
    let paths = &job.output_layout;

    fs::create_dir_all(&paths.inputs_dir).map_err(|e| IngestError(e.to_string()))?;
    fs::create_dir_all(&paths.derived_dir).map_err(|e| IngestError(e.to_string()))?;

    if opts.report_only {
        write_canonical_evidence(&decoded, &paths.derived_dir)?;
        let qc_report_path = write_qc_and_anomaly_reports_only(
            &opts.output_root,
            &opts.dataset,
            &paths.anomaly_report,
            &decoded.extract,
        )?;
        let manifest = ArtifactManifest::new(
            "1".to_string(),
            "report-only".to_string(),
            opts.dataset.clone(),
            ArtifactChecksums::new(String::new(), String::new(), String::new(), String::new()),
            ManifestStats::new(
                decoded.extract.gene_rows.len() as u64,
                decoded
                    .extract
                    .gene_rows
                    .iter()
                    .map(|x| x.transcript_count)
                    .sum::<u64>(),
                decoded.extract.contig_distribution.len() as u64,
            ),
        );
        return Ok(IngestResult {
            manifest_path: paths.manifest.clone(),
            sqlite_path: paths.sqlite.clone(),
            anomaly_report_path: paths.anomaly_report.clone(),
            qc_report_path,
            release_gene_index_path: paths.release_gene_index.clone(),
            normalized_debug_path: None,
            shard_catalog_path: None,
            shard_catalog: None,
            manifest,
            anomaly_report: decoded.extract.anomaly,
            events: Vec::new(),
        });
    }

    fs::copy(&job.inputs.gff3_path, &paths.gff3).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&job.inputs.fasta_path, &paths.fasta).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&job.inputs.fai_path, &paths.fai).map_err(|e| IngestError(e.to_string()))?;

    let hashes = compute_input_hashes(&paths.gff3, &paths.fasta, &paths.fai)?;
    write_canonical_evidence(&decoded, &paths.derived_dir)?;
    write_source_facts(job, &decoded, &hashes)?;

    write_sqlite(WriteSqliteInput {
        path: &paths.sqlite,
        dataset: &opts.dataset,
        genes: &decoded.extract.gene_rows,
        transcripts: &decoded.extract.transcript_rows,
        exons: &decoded.extract.exon_rows,
        contigs: &decoded.contig_stats,
        gff3_sha256: &hashes.gff3_sha256,
        fasta_sha256: &hashes.fasta_sha256,
        fai_sha256: &hashes.fai_sha256,
    })?;

    let effective_sharding_plan = if opts.emit_shards {
        ShardingPlan::Contig
    } else {
        opts.sharding_plan
    };

    let (shard_catalog_path, shard_catalog): (Option<std::path::PathBuf>, Option<ShardCatalog>) =
        if matches!(effective_sharding_plan, ShardingPlan::Contig) {
            let (catalog_path, catalog) = write_sharded_sqlite_catalog(
                &paths.derived_dir,
                &opts.dataset,
                &decoded.extract.gene_rows,
                &decoded.extract.transcript_rows,
                effective_sharding_plan,
                opts.shard_partitions,
                opts.max_shards,
            )?;
            (Some(catalog_path), Some(catalog))
        } else if matches!(effective_sharding_plan, ShardingPlan::RegionGrid) {
            return Err(IngestError(
                "region_grid sharding plan is reserved for future implementation".to_string(),
            ));
        } else {
            (None, None)
        };

    let normalized_debug_path = if opts.emit_normalized_debug || opts.normalized_replay_mode {
        let path = paths.derived_dir.join("normalized_features.jsonl.zst");
        write_normalized_jsonl_zst(
            &path,
            &decoded.extract.gene_rows,
            &decoded.extract.transcript_rows,
            &decoded.extract.exon_rows,
        )?;
        if opts.normalized_replay_mode {
            let replay = replay_counts_from_normalized(&path)?;
            if replay.genes != decoded.extract.gene_rows.len() as u64
                || replay.transcripts != decoded.extract.transcript_rows.len() as u64
                || replay.exons != decoded.extract.exon_rows.len() as u64
            {
                return Err(IngestError(format!(
                    "normalized replay mismatch: replay=({},{},{}) extracted=({},{},{})",
                    replay.genes,
                    replay.transcripts,
                    replay.exons,
                    decoded.extract.gene_rows.len(),
                    decoded.extract.transcript_rows.len(),
                    decoded.extract.exon_rows.len()
                )));
            }
        }
        Some(path)
    } else {
        None
    };

    let built = build_and_write_manifest_and_reports(BuildManifestArgs {
        output_root: &opts.output_root,
        dataset: &opts.dataset,
        gff3_path: &paths.gff3,
        fasta_path: &paths.fasta,
        fai_path: &paths.fai,
        sqlite_path: &paths.sqlite,
        manifest_path: &paths.manifest,
        anomaly_path: &paths.anomaly_report,
        extract: &decoded.extract,
        contig_aliases: &opts.seqid_policy.aliases,
        sharding_plan: effective_sharding_plan,
        canonical_model_schema_version: decoded.canonical_model.schema_version,
        canonical_query_semantic_sha256: &decoded.canonical_model.hashes.query_semantic_sha256,
        canonical_lineage_sha256: &decoded.canonical_model.hashes.lineage_sensitive_sha256,
        canonical_feature_counts: &decoded.canonical_model.summary.feature_type_counts,
    })?;

    if opts.compute_gene_signatures {
        build_and_write_release_gene_index(
            &opts.dataset,
            &paths.release_gene_index,
            &decoded.extract.gene_rows,
        )?;
    }

    let mut manifest = built.manifest.clone();
    let evidence_bundle_sha256 = write_evidence_sidecars(
        paths,
        &opts.dataset,
        &manifest,
        &decoded.extract.anomaly,
        &decoded.extract.contig_distribution,
        &decoded.extract.biotype_distribution,
        &decoded.extract.contig_class_distribution,
        &decoded.extract.seqid_normalization_traces,
        &decoded.extract.biotype_source_counts,
    )?;
    manifest.evidence_bundle_sha256 = evidence_bundle_sha256;
    let manifest_bytes =
        canonical::stable_json_bytes(&manifest).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.manifest, manifest_bytes).map_err(|e| IngestError(e.to_string()))?;

    Ok(IngestResult {
        manifest_path: paths.manifest.clone(),
        sqlite_path: paths.sqlite.clone(),
        anomaly_report_path: paths.anomaly_report.clone(),
        qc_report_path: built.qc_report_path,
        release_gene_index_path: paths.release_gene_index.clone(),
        normalized_debug_path,
        shard_catalog_path,
        shard_catalog,
        manifest,
        anomaly_report: decoded.extract.anomaly,
        events: Vec::new(),
    })
}

fn write_canonical_evidence(decoded: &DecodedIngest, derived_dir: &std::path::Path) -> Result<(), IngestError> {
    let canonical_path = derived_dir.join("canonical_features.json");
    let summary_path = derived_dir.join("canonical_summary.json");
    let model_bytes =
        canonical::stable_json_bytes(&decoded.canonical_model).map_err(|e| IngestError(e.to_string()))?;
    std::fs::write(canonical_path, model_bytes).map_err(|e| IngestError(e.to_string()))?;
    let summary_payload = serde_json::json!({
        "schema_version": decoded.canonical_model.schema_version,
        "summary": decoded.canonical_model.summary,
        "hashes": decoded.canonical_model.hashes,
        "query_semantic_payload": decoded.canonical_query_semantic_payload
    });
    let summary_bytes =
        canonical::stable_json_bytes(&summary_payload).map_err(|e| IngestError(e.to_string()))?;
    std::fs::write(summary_path, summary_bytes).map_err(|e| IngestError(e.to_string()))?;
    Ok(())
}

fn write_source_facts(
    job: &IngestJob,
    decoded: &DecodedIngest,
    hashes: &super::hashing::InputHashes,
) -> Result<(), IngestError> {
    let path = job.output_layout.source_facts.clone();
    let payload = serde_json::json!({
        "schema_version": 1,
        "dataset": job.options.dataset,
        "inputs": {
            "gff3_path": job.inputs.gff3_path,
            "fasta_path": job.inputs.fasta_path,
            "fai_path": job.inputs.fai_path,
            "hashes": {
                "gff3_sha256": hashes.gff3_sha256,
                "fasta_sha256": hashes.fasta_sha256,
                "fai_sha256": hashes.fai_sha256
            }
        },
        "normalization": {
            "seqid_aliases": job.options.seqid_policy.aliases,
            "reject_normalized_seqid_collisions": job.options.reject_normalized_seqid_collisions,
            "seqid_traces": decoded.extract.seqid_normalization_traces
        },
        "contigs": decoded.contig_stats,
        "feature_distribution": {
            "contigs": decoded.extract.contig_distribution,
            "contig_classes": decoded.extract.contig_class_distribution,
            "biotypes": decoded.extract.biotype_distribution,
            "biotype_source_keys": decoded.extract.biotype_source_counts
        },
        "canonical_model": {
            "schema_version": decoded.canonical_model.schema_version,
            "hashes": decoded.canonical_model.hashes,
            "summary": decoded.canonical_model.summary,
        },
        "scientific_ambiguities": decoded.extract.anomaly.scientific_ambiguities,
    });
    let bytes = canonical::stable_json_bytes(&payload).map_err(|e| IngestError(e.to_string()))?;
    fs::write(path, bytes).map_err(|e| IngestError(e.to_string()))
}

fn write_evidence_sidecars(
    paths: &crate::domain::dataset::ArtifactPaths,
    dataset: &crate::domain::dataset::DatasetId,
    manifest: &ArtifactManifest,
    anomaly: &IngestAnomalyReport,
    contig_distribution: &std::collections::BTreeMap<String, u64>,
    biotype_distribution: &std::collections::BTreeMap<String, u64>,
    contig_class_distribution: &std::collections::BTreeMap<String, u64>,
    seqid_normalization_traces: &std::collections::BTreeMap<
        String,
        crate::domain::query::SeqidNormalizationTrace,
    >,
    biotype_source_counts: &std::collections::BTreeMap<String, u64>,
) -> Result<String, IngestError> {
    let anomaly_counts = anomaly.anomaly_class_counts();
    let mut severity_summary = std::collections::BTreeMap::from([
        ("INFO".to_string(), 0_u64),
        ("WARN".to_string(), 0_u64),
        ("ERROR".to_string(), 0_u64),
    ]);
    let mut class_items = Vec::new();
    for (class, count) in anomaly_counts.iter().filter(|(_, count)| **count > 0) {
        let severity = IngestAnomalyReport::severity_for_class(*class);
        let key = match severity {
            crate::domain::dataset::QcSeverity::Info => "INFO",
            crate::domain::dataset::QcSeverity::Warn => "WARN",
            crate::domain::dataset::QcSeverity::Error => "ERROR",
        };
        *severity_summary.entry(key.to_string()).or_insert(0) += *count;
        class_items.push(json!({
            "class": class,
            "severity": severity,
            "count": count
        }));
    }
    let anomaly_summary = json!({
        "schema_version": 1,
        "dataset": dataset,
        "release_id": manifest.identity.release_id,
        "anomaly_class_counts": anomaly_counts,
        "severity_summary": severity_summary,
        "items": class_items
    });
    let anomaly_summary_bytes =
        canonical::stable_json_bytes(&anomaly_summary).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.anomaly_summary, anomaly_summary_bytes).map_err(|e| IngestError(e.to_string()))?;

    let build_metadata = json!({
        "schema_version": 1,
        "dataset": dataset,
        "release_id": manifest.identity.release_id,
        "software_version": manifest.software_version,
        "config_version": manifest.config_version,
        "build_policy_version": manifest.build_policy_version,
        "toolchain_hash": manifest.toolchain_hash,
        "ingest_toolchain": manifest.ingest_toolchain,
        "ingest_build_hash": manifest.ingest_build_hash
    });
    let build_metadata_bytes =
        canonical::stable_json_bytes(&build_metadata).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.build_metadata, build_metadata_bytes).map_err(|e| IngestError(e.to_string()))?;

    let dataset_stats = json!({
        "schema_version": 1,
        "dataset": dataset,
        "release_id": manifest.identity.release_id,
        "stats": manifest.stats,
        "canonical_feature_counts": manifest.canonical_feature_counts,
        "contig_distribution": contig_distribution,
        "contig_class_distribution": contig_class_distribution,
        "biotype_distribution": biotype_distribution,
        "biotype_source_counts": biotype_source_counts,
        "rejected_record_count": anomaly.rejections.len(),
    });
    let dataset_stats_bytes =
        canonical::stable_json_bytes(&dataset_stats).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.dataset_stats, dataset_stats_bytes).map_err(|e| IngestError(e.to_string()))?;

    let scientific_profile = json!({
        "schema_version": 1,
        "dataset": dataset,
        "release_id": manifest.identity.release_id,
        "coordinate_system": manifest.coordinate_system,
        "reference_build_identity_sha256": manifest.reference_build_identity_sha256,
        "contig_naming_style": manifest.contig_naming_style,
        "scientific_prerequisites_status": manifest.scientific_prerequisites_status,
        "contig_class_distribution": contig_class_distribution,
        "seqid_normalization_traces": seqid_normalization_traces,
        "scientific_ambiguities": anomaly.scientific_ambiguities,
    });
    let scientific_profile_bytes = canonical::stable_json_bytes(&scientific_profile)
        .map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.scientific_profile, scientific_profile_bytes)
        .map_err(|e| IngestError(e.to_string()))?;

    let mut inventory_items = Vec::new();
    for (role, path) in [
        ("sqlite", &paths.sqlite),
        ("anomaly_report", &paths.anomaly_report),
        ("anomaly_summary", &paths.anomaly_summary),
        ("qc_report", &paths.qc_report),
        ("source_facts", &paths.source_facts),
        ("build_metadata", &paths.build_metadata),
        ("dataset_stats", &paths.dataset_stats),
        ("scientific_profile", &paths.scientific_profile),
        ("canonical_features", &paths.derived_dir.join("canonical_features.json")),
        ("canonical_summary", &paths.derived_dir.join("canonical_summary.json")),
        ("release_gene_index", &paths.release_gene_index),
        ("gff3", &paths.gff3),
        ("fasta", &paths.fasta),
        ("fai", &paths.fai),
    ] {
        if !path.exists() {
            continue;
        }
        let raw = fs::read(path).map_err(|e| IngestError(e.to_string()))?;
        let rel = path
            .strip_prefix(&paths.dataset_root)
            .unwrap_or(path)
            .display()
            .to_string();
        inventory_items.push(json!({
            "role": role,
            "path": rel,
            "sha256": sha256_hex(&raw),
            "bytes": raw.len(),
        }));
    }
    inventory_items.sort_by(|a, b| {
        a.get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .cmp(
                b.get("path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
            )
    });
    let inventory = json!({
        "schema_version": 1,
        "dataset": dataset,
        "release_id": manifest.identity.release_id,
        "normalized_input_identity_sha256": manifest.normalized_input_identity_sha256,
        "items": inventory_items
    });
    let inventory_bytes =
        canonical::stable_json_bytes(&inventory).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.artifact_inventory, inventory_bytes).map_err(|e| IngestError(e.to_string()))?;

    let mut bundle_files = std::collections::BTreeMap::new();
    for path in [
        &paths.anomaly_report,
        &paths.anomaly_summary,
        &paths.qc_report,
        &paths.source_facts,
        &paths.build_metadata,
        &paths.dataset_stats,
        &paths.scientific_profile,
        &paths.artifact_inventory,
    ] {
        if !path.exists() {
            continue;
        }
        let raw = fs::read(path).map_err(|e| IngestError(e.to_string()))?;
        let rel = path
            .strip_prefix(&paths.dataset_root)
            .unwrap_or(path)
            .display()
            .to_string();
        bundle_files.insert(rel, sha256_hex(&raw));
    }
    let bundle_payload = json!({
        "schema_version": 1,
        "dataset": dataset,
        "release_id": manifest.identity.release_id,
        "files": bundle_files,
    });
    let bundle_bytes =
        canonical::stable_json_bytes(&bundle_payload).map_err(|e| IngestError(e.to_string()))?;
    let bundle_sha = sha256_hex(&bundle_bytes);
    let bundle_lock = json!({
        "schema_version": 1,
        "dataset": dataset,
        "release_id": manifest.identity.release_id,
        "bundle_sha256": bundle_sha,
        "files": bundle_payload["files"],
    });
    let bundle_lock_bytes =
        canonical::stable_json_bytes(&bundle_lock).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.evidence_bundle, bundle_lock_bytes).map_err(|e| IngestError(e.to_string()))?;
    Ok(bundle_sha)
}
