// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod canonical_model;
mod decode;
mod diff_index;
mod extract;
mod fai;
mod gff3;
mod hashing;
mod job;
mod logging;
mod manifest;
mod normalized;
mod sqlite;
mod write;

use crate::domain::dataset::{
    ArtifactManifest, DatasetId, IngestAnomalyReport, ShardCatalog, ShardingPlan,
};
use crate::domain::policy::{GeneIdentifierPolicy, StrictnessMode};
use crate::domain::query::{
    BiotypePolicy, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy,
    GeneNamePolicy, SeqidNormalizationPolicy, TranscriptIdPolicy, TranscriptTypePolicy,
    UnknownFeaturePolicy,
};
use sqlite::explain_plan_for_region_query;
#[cfg(test)]
use sqlite::{explain_plan_for_gene_id_query, explain_plan_for_name_query};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

pub const CRATE_NAME: &str = "bijux-atlas";

pub use hashing::{compute_input_hashes, hash_file, InputHashes};
pub use job::{IngestInputs, IngestJob};
pub use logging::{IngestEvent, IngestLog, IngestStage};
pub use normalized::ReplayCounts;

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
    pub max_warn_anomalies: Option<u64>,
    pub max_error_anomalies: Option<u64>,
    pub allow_overlap_gene_ids_across_contigs: bool,
    pub dev_allow_auto_generate_fai: bool,
    pub emit_normalized_debug: bool,
    pub normalized_replay_mode: bool,
    pub prod_mode: bool,
    pub timestamp_policy: TimestampPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampPolicy {
    DeterministicZero,
    SourceMetadataOnly,
}

impl IngestOptions {
    #[must_use]
    pub fn for_dataset(dataset: DatasetId) -> Self {
        Self {
            gff3_path: PathBuf::new(),
            fasta_path: PathBuf::new(),
            fai_path: PathBuf::new(),
            output_root: PathBuf::new(),
            dataset,
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
            max_warn_anomalies: None,
            max_error_anomalies: None,
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
            timestamp_policy: TimestampPolicy::DeterministicZero,
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
    pub manifest: ArtifactManifest,
    pub anomaly_report: IngestAnomalyReport,
    pub events: Vec<IngestEvent>,
}

pub fn ingest_dataset(opts: &IngestOptions) -> Result<IngestResult, IngestError> {
    ingest_dataset_with_events(opts).map(|(result, _)| result)
}

pub fn ingest_dataset_with_events(
    opts: &IngestOptions,
) -> Result<(IngestResult, Vec<IngestEvent>), IngestError> {
    let mut log = logging::IngestLog::default();
    log.emit(
        logging::IngestStage::Prepare,
        "ingest.start",
        std::collections::BTreeMap::new(),
    );

    if opts.dataset.release.as_str() == "0"
        && opts.dataset.species.as_str() == "unknown"
        && opts.dataset.assembly.as_str() == "unknown"
    {
        return Err(IngestError(
            "dataset identity is required; implicit default dataset is forbidden".to_string(),
        ));
    }
    let _effective_threads = extract::parallelism_policy(opts.max_threads)?;
    if opts.prod_mode && opts.emit_normalized_debug {
        return Err(IngestError(
            "policy gate: normalized debug output is disabled in production mode".to_string(),
        ));
    }
    let job = job::IngestJob::from_options(opts);
    log.emit(
        logging::IngestStage::Decode,
        "ingest.decode.begin",
        std::collections::BTreeMap::new(),
    );
    let decoded = decode::decode_ingest_inputs(&job)?;
    log.emit(
        logging::IngestStage::Decode,
        "ingest.decode.complete",
        std::collections::BTreeMap::new(),
    );

    let anomaly_gate = evaluate_anomaly_thresholds(&decoded.extract.anomaly, opts);
    if let Err(err) = anomaly_gate {
        return Err(err);
    }
    if opts.fail_on_warn && has_qc_warn(&decoded.extract.anomaly) {
        let warn_count = decoded
            .extract
            .anomaly
            .anomaly_class_counts()
            .into_iter()
            .filter(|(class, _)| {
                matches!(
                    IngestAnomalyReport::severity_for_class(*class),
                    crate::domain::dataset::QcSeverity::Warn
                )
            })
            .map(|(_, count)| count)
            .sum::<u64>();
        return Err(IngestError(
            format!(
                "INGEST_WARN_POLICY_REJECTED: {} warning anomalies present. Resolve source warnings or run without --strict when policy allows.",
                warn_count
            ),
        ));
    }
    log.emit(
        logging::IngestStage::Persist,
        "ingest.persist.begin",
        std::collections::BTreeMap::new(),
    );
    let mut result = write::write_ingest_outputs(&job, decoded)?;
    log.emit(
        logging::IngestStage::Finalize,
        "ingest.persist.complete",
        std::collections::BTreeMap::new(),
    );
    result.events = log.events().to_vec();
    Ok((result, log.events().to_vec()))
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

fn evaluate_anomaly_thresholds(
    anomaly: &IngestAnomalyReport,
    opts: &IngestOptions,
) -> Result<(), IngestError> {
    let class_counts = anomaly.anomaly_class_counts();
    let mut warn_total = 0_u64;
    let mut error_total = 0_u64;
    for (class, count) in class_counts {
        match IngestAnomalyReport::severity_for_class(class) {
            crate::domain::dataset::QcSeverity::Warn => {
                warn_total = warn_total.saturating_add(count)
            }
            crate::domain::dataset::QcSeverity::Error => {
                error_total = error_total.saturating_add(count)
            }
            crate::domain::dataset::QcSeverity::Info => {}
        }
    }
    if let Some(max_warn) = opts.max_warn_anomalies {
        if warn_total > max_warn {
            return Err(IngestError(format!(
                "ingest anomaly threshold exceeded: WARN {} > max_warn_anomalies {}. Reduce WARN-class anomalies in sources or raise max_warn_anomalies deliberately.",
                warn_total, max_warn
            )));
        }
    }
    if let Some(max_error) = opts.max_error_anomalies {
        if error_total > max_error {
            return Err(IngestError(format!(
                "ingest anomaly threshold exceeded: ERROR {} > max_error_anomalies {}. Fix source errors before publication; threshold increases should be exceptional.",
                error_total, max_error
            )));
        }
    }
    Ok(())
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
    normalized::replay_counts_from_normalized(path)
}

pub fn diff_normalized_ids(
    base: &Path,
    target: &Path,
) -> Result<(Vec<String>, Vec<String>), IngestError> {
    normalized::diff_normalized_record_ids(base, target)
}

#[cfg(test)]
mod tests;
