// SPDX-License-Identifier: Apache-2.0

use std::fs;

use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, ManifestStats, ShardCatalog, ShardingPlan,
};

use crate::diff_index::build_and_write_release_gene_index;
use crate::hashing::compute_input_hashes;
use crate::job::IngestJob;
use crate::manifest::{
    build_and_write_manifest_and_reports, write_qc_and_anomaly_reports_only, BuildManifestArgs,
};
use crate::normalized::{replay_counts_from_normalized, write_normalized_jsonl_zst};
use crate::sqlite::{write_sharded_sqlite_catalog, write_sqlite, WriteSqliteInput};
use crate::{IngestError, IngestResult};

use crate::decode::DecodedIngest;

pub fn write_ingest_outputs(
    job: &IngestJob,
    decoded: DecodedIngest,
) -> Result<IngestResult, IngestError> {
    let opts = &job.options;
    let paths = &job.output_layout;

    fs::create_dir_all(&paths.inputs_dir).map_err(|e| IngestError(e.to_string()))?;
    fs::create_dir_all(&paths.derived_dir).map_err(|e| IngestError(e.to_string()))?;

    if opts.report_only {
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
    })?;

    if opts.compute_gene_signatures {
        build_and_write_release_gene_index(
            &opts.dataset,
            &paths.release_gene_index,
            &decoded.extract.gene_rows,
        )?;
    }

    Ok(IngestResult {
        manifest_path: paths.manifest.clone(),
        sqlite_path: paths.sqlite.clone(),
        anomaly_report_path: paths.anomaly_report.clone(),
        qc_report_path: built.qc_report_path,
        release_gene_index_path: paths.release_gene_index.clone(),
        normalized_debug_path,
        shard_catalog_path,
        shard_catalog,
        manifest: built.manifest,
        anomaly_report: decoded.extract.anomaly,
        events: Vec::new(),
    })
}
