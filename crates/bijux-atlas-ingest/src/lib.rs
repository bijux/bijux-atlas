#![forbid(unsafe_code)]

mod diff_index;
mod extract;
mod fai;
mod gff3;
mod manifest;
mod normalized;
mod sqlite;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    artifact_paths, BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy,
    FeatureIdUniquenessPolicy, GeneIdentifierPolicy, GeneNamePolicy, IngestAnomalyReport,
    SeqidNormalizationPolicy, ShardCatalog, ShardingPlan, StrictnessMode, TranscriptIdPolicy,
    TranscriptTypePolicy, UnknownFeaturePolicy,
};
use diff_index::build_and_write_release_gene_index;
use extract::extract_gene_rows;
use gff3::parse_gff3_records;
use manifest::{
    build_and_write_manifest_and_reports, write_qc_and_anomaly_reports_only, BuildManifestArgs,
};
use normalized::{replay_counts_from_normalized, write_normalized_jsonl_zst};
#[cfg(test)]
use sqlite::{explain_plan_for_gene_id_query, explain_plan_for_name_query};
use sqlite::{
    explain_plan_for_region_query, write_sharded_sqlite_catalog, write_sqlite, WriteSqliteInput,
};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

pub const CRATE_NAME: &str = "bijux-atlas-ingest";

#[derive(Debug)]
pub struct IngestError(pub String);
impl Display for IngestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for IngestError {}

#[derive(Debug, Clone)]
pub struct IngestOptions {
    pub gff3_path: PathBuf,
    pub fasta_path: PathBuf,
    pub fai_path: PathBuf,
    pub output_root: PathBuf,
    pub dataset: DatasetId,
    pub strictness: StrictnessMode,
    pub duplicate_gene_id_policy: DuplicateGeneIdPolicy,
    pub duplicate_transcript_id_policy: DuplicateTranscriptIdPolicy,
    pub gene_identifier_policy: GeneIdentifierPolicy,
    pub gene_name_policy: GeneNamePolicy,
    pub biotype_policy: BiotypePolicy,
    pub transcript_type_policy: TranscriptTypePolicy,
    pub transcript_id_policy: TranscriptIdPolicy,
    pub seqid_policy: SeqidNormalizationPolicy,
    pub unknown_feature_policy: UnknownFeaturePolicy,
    pub feature_id_uniqueness_policy: FeatureIdUniquenessPolicy,
    pub reject_normalized_seqid_collisions: bool,
    pub max_threads: usize,
    pub emit_shards: bool,
    pub shard_partitions: usize,
    pub sharding_plan: ShardingPlan,
    pub max_shards: usize,
    pub compute_gene_signatures: bool,
    pub compute_contig_fractions: bool,
    pub fasta_scanning_enabled: bool,
    pub fasta_scan_max_bases: u64,
    pub compute_transcript_spliced_length: bool,
    pub compute_transcript_cds_length: bool,
    pub report_only: bool,
    pub fail_on_warn: bool,
    pub allow_overlap_gene_ids_across_contigs: bool,
    pub dev_allow_auto_generate_fai: bool,
    pub emit_normalized_debug: bool,
    pub normalized_replay_mode: bool,
    pub prod_mode: bool,
}

impl Default for IngestOptions {
    fn default() -> Self {
        Self {
            gff3_path: PathBuf::new(),
            fasta_path: PathBuf::new(),
            fai_path: PathBuf::new(),
            output_root: PathBuf::new(),
            dataset: DatasetId::new("0", "unknown", "unknown").expect("default dataset"),
            strictness: StrictnessMode::Strict,
            duplicate_gene_id_policy: DuplicateGeneIdPolicy::Fail,
            duplicate_transcript_id_policy: DuplicateTranscriptIdPolicy::Reject,
            gene_identifier_policy: GeneIdentifierPolicy::Gff3Id,
            gene_name_policy: GeneNamePolicy::default(),
            biotype_policy: BiotypePolicy::default(),
            transcript_type_policy: TranscriptTypePolicy::default(),
            transcript_id_policy: TranscriptIdPolicy::default(),
            seqid_policy: SeqidNormalizationPolicy::default(),
            unknown_feature_policy: UnknownFeaturePolicy::IgnoreWithWarning,
            feature_id_uniqueness_policy: FeatureIdUniquenessPolicy::Reject,
            reject_normalized_seqid_collisions: true,
            max_threads: 1,
            fail_on_warn: false,
            allow_overlap_gene_ids_across_contigs: false,
            emit_shards: false,
            shard_partitions: 0,
            sharding_plan: ShardingPlan::None,
            max_shards: 512,
            compute_gene_signatures: true,
            compute_contig_fractions: false,
            fasta_scanning_enabled: false,
            fasta_scan_max_bases: 2_000_000_000,
            compute_transcript_spliced_length: false,
            compute_transcript_cds_length: false,
            report_only: false,
            dev_allow_auto_generate_fai: false,
            emit_normalized_debug: false,
            normalized_replay_mode: false,
            prod_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IngestResult {
    pub manifest_path: PathBuf,
    pub sqlite_path: PathBuf,
    pub anomaly_report_path: PathBuf,
    pub qc_report_path: PathBuf,
    pub release_gene_index_path: PathBuf,
    pub normalized_debug_path: Option<PathBuf>,
    pub shard_catalog_path: Option<PathBuf>,
    pub shard_catalog: Option<ShardCatalog>,
    pub manifest: bijux_atlas_model::ArtifactManifest,
    pub anomaly_report: IngestAnomalyReport,
}

pub fn ingest_dataset(opts: &IngestOptions) -> Result<IngestResult, IngestError> {
    if opts.dataset.release.as_str() == "0"
        && opts.dataset.species.as_str() == "unknown"
        && opts.dataset.assembly.as_str() == "unknown"
    {
        return Err(IngestError(
            "dataset identity is required; implicit default dataset is forbidden".to_string(),
        ));
    }
    let _effective_threads = extract::parallelism_policy(opts.max_threads)?;

    if !opts.fai_path.exists() {
        if opts.dev_allow_auto_generate_fai {
            fai::write_fai_from_fasta(&opts.fasta_path, &opts.fai_path)?;
        } else {
            return Err(IngestError(
                "FAI index is required for ingest (enable dev auto-generate explicitly)"
                    .to_string(),
            ));
        }
    }
    if opts.prod_mode && opts.emit_normalized_debug {
        return Err(IngestError(
            "policy gate: normalized debug output is disabled in production mode".to_string(),
        ));
    }
    let contig_lengths = fai::read_fai_contig_lengths(&opts.fai_path)?;
    let contig_stats = if opts.fasta_scanning_enabled {
        fai::read_fasta_contig_stats(
            &opts.fasta_path,
            opts.compute_contig_fractions,
            opts.fasta_scan_max_bases,
        )?
    } else {
        contig_lengths
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    fai::ContigStats {
                        length: *v,
                        gc_fraction: None,
                        n_fraction: None,
                    },
                )
            })
            .collect()
    };
    let records = parse_gff3_records(&opts.gff3_path)?;
    let extracted = extract_gene_rows(records, &contig_lengths, opts)?;
    if opts.fail_on_warn && has_qc_warn(&extracted.anomaly) {
        return Err(IngestError(
            "strict warning policy rejected ingest: QC WARN present".to_string(),
        ));
    }

    let paths = artifact_paths(&opts.output_root, &opts.dataset);
    fs::create_dir_all(&paths.inputs_dir).map_err(|e| IngestError(e.to_string()))?;
    fs::create_dir_all(&paths.derived_dir).map_err(|e| IngestError(e.to_string()))?;

    if opts.report_only {
        let qc_report_path = write_qc_and_anomaly_reports_only(
            &opts.output_root,
            &opts.dataset,
            &paths.anomaly_report,
            &extracted,
        )?;
        let manifest = bijux_atlas_model::ArtifactManifest::new(
            "1".to_string(),
            "report-only".to_string(),
            opts.dataset.clone(),
            bijux_atlas_model::ArtifactChecksums::new(
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ),
            bijux_atlas_model::ManifestStats::new(
                extracted.gene_rows.len() as u64,
                extracted
                    .gene_rows
                    .iter()
                    .map(|x| x.transcript_count)
                    .sum::<u64>(),
                extracted.contig_distribution.len() as u64,
            ),
        );
        return Ok(IngestResult {
            manifest_path: paths.manifest,
            sqlite_path: paths.sqlite,
            anomaly_report_path: paths.anomaly_report,
            qc_report_path,
            release_gene_index_path: paths.release_gene_index,
            normalized_debug_path: None,
            shard_catalog_path: None,
            shard_catalog: None,
            manifest,
            anomaly_report: extracted.anomaly,
        });
    }

    fs::copy(&opts.gff3_path, &paths.gff3).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fasta_path, &paths.fasta).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fai_path, &paths.fai).map_err(|e| IngestError(e.to_string()))?;

    write_sqlite(WriteSqliteInput {
        path: &paths.sqlite,
        dataset: &opts.dataset,
        genes: &extracted.gene_rows,
        transcripts: &extracted.transcript_rows,
        exons: &extracted.exon_rows,
        contigs: &contig_stats,
        gff3_sha256: &sha256_hex(&fs::read(&paths.gff3).map_err(|e| IngestError(e.to_string()))?),
        fasta_sha256: &sha256_hex(&fs::read(&paths.fasta).map_err(|e| IngestError(e.to_string()))?),
        fai_sha256: &sha256_hex(&fs::read(&paths.fai).map_err(|e| IngestError(e.to_string()))?),
    })?;
    let effective_sharding_plan = if opts.emit_shards {
        ShardingPlan::Contig
    } else {
        opts.sharding_plan
    };
    let (shard_catalog_path, shard_catalog) =
        if matches!(effective_sharding_plan, ShardingPlan::Contig) {
            let (catalog_path, catalog) = write_sharded_sqlite_catalog(
                &paths.derived_dir,
                &opts.dataset,
                &extracted.gene_rows,
                &extracted.transcript_rows,
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
            &extracted.gene_rows,
            &extracted.transcript_rows,
            &extracted.exon_rows,
        )?;
        if opts.normalized_replay_mode {
            let replay = replay_counts_from_normalized(&path)?;
            if replay.genes != extracted.gene_rows.len() as u64
                || replay.transcripts != extracted.transcript_rows.len() as u64
                || replay.exons != extracted.exon_rows.len() as u64
            {
                return Err(IngestError(format!(
                    "normalized replay mismatch: replay=({},{},{}) extracted=({},{},{})",
                    replay.genes,
                    replay.transcripts,
                    replay.exons,
                    extracted.gene_rows.len(),
                    extracted.transcript_rows.len(),
                    extracted.exon_rows.len()
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
        extract: &extracted,
        contig_aliases: &opts.seqid_policy.aliases,
        sharding_plan: effective_sharding_plan,
    })?;
    if opts.compute_gene_signatures {
        build_and_write_release_gene_index(
            &opts.dataset,
            &paths.release_gene_index,
            &extracted.gene_rows,
        )?;
    }

    Ok(IngestResult {
        manifest_path: paths.manifest,
        sqlite_path: paths.sqlite,
        anomaly_report_path: paths.anomaly_report,
        qc_report_path: built.qc_report_path,
        release_gene_index_path: paths.release_gene_index,
        normalized_debug_path,
        shard_catalog_path,
        shard_catalog,
        manifest: built.manifest,
        anomaly_report: extracted.anomaly,
    })
}

fn has_qc_warn(anomaly: &IngestAnomalyReport) -> bool {
    !anomaly.missing_parents.is_empty()
        || !anomaly.missing_transcript_parents.is_empty()
        || !anomaly.multiple_parent_transcripts.is_empty()
        || !anomaly.unknown_contigs.is_empty()
        || !anomaly.overlapping_ids.is_empty()
        || !anomaly.duplicate_gene_ids.is_empty()
        || !anomaly.overlapping_gene_ids_across_contigs.is_empty()
        || !anomaly.orphan_transcripts.is_empty()
        || !anomaly.parent_cycles.is_empty()
        || !anomaly.attribute_fallbacks.is_empty()
        || !anomaly.unknown_feature_types.is_empty()
        || !anomaly.missing_required_fields.is_empty()
}

pub fn read_fai_contig_lengths(
    path: &Path,
) -> Result<std::collections::BTreeMap<String, u64>, IngestError> {
    fai::read_fai_contig_lengths(path)
}

pub fn explain_region_query_plan(sqlite_path: &Path) -> Result<Vec<String>, IngestError> {
    explain_plan_for_region_query(sqlite_path)
}

pub fn replay_normalized_counts(path: &Path) -> Result<normalized::ReplayCounts, IngestError> {
    replay_counts_from_normalized(path)
}

pub fn diff_normalized_ids(
    base: &Path,
    target: &Path,
) -> Result<(Vec<String>, Vec<String>), IngestError> {
    normalized::diff_normalized_record_ids(base, target)
}

#[cfg(test)]
mod tests;
