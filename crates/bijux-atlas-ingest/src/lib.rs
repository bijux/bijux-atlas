#![forbid(unsafe_code)]

mod diff_index;
mod extract;
mod fai;
mod gff3;
mod manifest;
mod sqlite;

use bijux_atlas_model::{
    artifact_paths, BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, GeneIdentifierPolicy,
    DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy, GeneNamePolicy, IngestAnomalyReport,
    SeqidNormalizationPolicy, ShardCatalog, StrictnessMode, TranscriptIdPolicy,
    TranscriptTypePolicy, UnknownFeaturePolicy,
};
use diff_index::build_and_write_release_gene_index;
use extract::extract_gene_rows;
use gff3::parse_gff3_records;
use manifest::{
    build_and_write_manifest_and_reports, write_qc_and_anomaly_reports_only, BuildManifestArgs,
};
use sqlite::{explain_plan_for_region_query, write_sharded_sqlite_catalog, write_sqlite};
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
    pub compute_gene_signatures: bool,
    pub compute_contig_fractions: bool,
    pub compute_transcript_spliced_length: bool,
    pub compute_transcript_cds_length: bool,
    pub report_only: bool,
    pub fail_on_warn: bool,
    pub allow_overlap_gene_ids_across_contigs: bool,
    pub dev_allow_auto_generate_fai: bool,
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
            compute_gene_signatures: true,
            compute_contig_fractions: false,
            compute_transcript_spliced_length: false,
            compute_transcript_cds_length: false,
            report_only: false,
            dev_allow_auto_generate_fai: false,
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
                "FAI index is required for ingest (enable dev auto-generate explicitly)".to_string(),
            ));
        }
    }
    let contig_lengths = fai::read_fai_contig_lengths(&opts.fai_path)?;
    let contig_stats = fai::read_fasta_contig_stats(&opts.fasta_path, opts.compute_contig_fractions)?;
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
            shard_catalog_path: None,
            shard_catalog: None,
            manifest,
            anomaly_report: extracted.anomaly,
        });
    }

    fs::copy(&opts.gff3_path, &paths.gff3).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fasta_path, &paths.fasta).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fai_path, &paths.fai).map_err(|e| IngestError(e.to_string()))?;

    write_sqlite(
        &paths.sqlite,
        &opts.dataset,
        &extracted.gene_rows,
        &extracted.transcript_rows,
        &contig_stats,
    )?;
    let (shard_catalog_path, shard_catalog) = if opts.emit_shards {
        let (catalog_path, catalog) = write_sharded_sqlite_catalog(
            &paths.derived_dir,
            &opts.dataset,
            &extracted.gene_rows,
            &extracted.transcript_rows,
            opts.shard_partitions,
        )?;
        (Some(catalog_path), Some(catalog))
    } else {
        (None, None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny")
    }

    fn opts(root: &Path, strictness: StrictnessMode) -> IngestOptions {
        IngestOptions {
            gff3_path: fixture_dir().join("genes.gff3"),
            fasta_path: fixture_dir().join("genome.fa"),
            fai_path: fixture_dir().join("genome.fa.fai"),
            output_root: root.to_path_buf(),
            dataset: DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id"),
            strictness,
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
            compute_gene_signatures: true,
            compute_contig_fractions: false,
            compute_transcript_spliced_length: false,
            compute_transcript_cds_length: false,
            report_only: false,
            dev_allow_auto_generate_fai: false,
        }
    }

    #[test]
    fn ingest_is_deterministic_and_matches_contract() {
        let root = tempdir().expect("tempdir");
        let run1 = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("run1");
        let alt = tempdir().expect("tempdir2");
        let run2 = ingest_dataset(&opts(alt.path(), StrictnessMode::Strict)).expect("run2");

        assert_eq!(
            run1.manifest.checksums.sqlite_sha256,
            run2.manifest.checksums.sqlite_sha256
        );
        assert_eq!(run1.manifest.stats.gene_count, 2);
        assert_eq!(run1.manifest.stats.transcript_count, 3);
        assert!(run1.release_gene_index_path.exists());
    }

    #[test]
    fn deterministic_across_parallelism_settings() {
        let root = tempdir().expect("tempdir");
        let mut o1 = opts(root.path(), StrictnessMode::Strict);
        o1.max_threads = 1;
        let run1 = ingest_dataset(&o1).expect("run1");

        let alt = tempdir().expect("tempdir2");
        let mut o2 = opts(alt.path(), StrictnessMode::Strict);
        o2.max_threads = 8;
        let run2 = ingest_dataset(&o2).expect("run2");

        assert_eq!(
            run1.manifest.dataset_signature_sha256,
            run2.manifest.dataset_signature_sha256
        );
        assert_eq!(
            run1.manifest.checksums.sqlite_sha256,
            run2.manifest.checksums.sqlite_sha256
        );
    }

    #[test]
    fn strict_mode_rejects_missing_parent() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = fixture_dir().join("genes_missing_parent.gff3");
        assert!(ingest_dataset(&o).is_err());
    }

    #[test]
    fn report_only_collects_anomalies() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::ReportOnly);
        o.gff3_path = fixture_dir().join("genes_missing_parent.gff3");
        let result = ingest_dataset(&o).expect("report only should succeed");
        assert!(!result.anomaly_report.missing_parents.is_empty());
    }

    #[test]
    fn strict_warn_mode_fails_on_qc_warn() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::ReportOnly);
        o.gff3_path = fixture_dir().join("genes_missing_parent.gff3");
        o.fail_on_warn = true;
        let err = ingest_dataset(&o).expect_err("strict warn must fail");
        assert!(err.to_string().contains("QC WARN"));
    }

    #[test]
    fn report_only_writes_qc_and_anomaly_without_sqlite_manifest() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::ReportOnly);
        o.report_only = true;
        let out = ingest_dataset(&o).expect("report-only ingest");
        assert!(out.qc_report_path.exists());
        assert!(out.anomaly_report_path.exists());
        assert!(!out.sqlite_path.exists());
        assert!(!out.manifest_path.exists());
    }

    #[test]
    fn cyclic_parent_graph_is_detected() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = fixture_dir().join("genes_parent_cycle.gff3");
        let err = ingest_dataset(&o).expect_err("cycle must fail in strict mode");
        assert!(err.to_string().contains("cyclic Parent graph"));
    }

    #[test]
    fn overlapping_gene_ids_across_contigs_requires_explicit_allow() {
        let root = tempdir().expect("tempdir");
        let mut strict = opts(root.path(), StrictnessMode::Strict);
        strict.gff3_path = fixture_dir().join("genes_overlap_contig_ids.gff3");
        let err = ingest_dataset(&strict).expect_err("strict overlap must fail");
        assert!(err.to_string().contains("appears across multiple contigs"));

        let ok_root = tempdir().expect("tempdir2");
        let mut allowed = opts(ok_root.path(), StrictnessMode::Lenient);
        allowed.gff3_path = fixture_dir().join("genes_overlap_contig_ids.gff3");
        allowed.allow_overlap_gene_ids_across_contigs = true;
        let run = ingest_dataset(&allowed).expect("allowed overlap should ingest");
        assert!(!run
            .anomaly_report
            .overlapping_gene_ids_across_contigs
            .is_empty());
    }

    #[test]
    fn contig_coordinate_validation_rejects_out_of_bounds() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/contigs/genes_invalid_coord.gff3");
        o.fasta_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contigs/genome.fa");
        o.fai_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contigs/genome.fa.fai");
        assert!(ingest_dataset(&o).is_err());
    }

    #[test]
    fn unknown_contig_is_contractual_deterministic_failure() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/edgecases/case_9_unknown_contig.gff3");
        let e1 = ingest_dataset(&o).expect_err("unknown contig must fail");
        let e2 = ingest_dataset(&o).expect_err("unknown contig must fail deterministically");
        assert_eq!(e1.to_string(), e2.to_string());
    }

    #[test]
    fn missing_fai_fails_by_default_but_can_autogenerate_in_dev_mode() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.fai_path = root.path().join("autogen.fai");
        let err = ingest_dataset(&o).expect_err("missing fai must fail by default");
        assert!(err.to_string().contains("FAI index is required"));

        let mut dev = opts(root.path(), StrictnessMode::Strict);
        dev.fai_path = root.path().join("autogen-dev.fai");
        dev.dev_allow_auto_generate_fai = true;
        let run = ingest_dataset(&dev).expect("dev autogen should pass");
        assert!(run.sqlite_path.exists());
        assert!(dev.fai_path.exists());
    }

    #[test]
    fn explain_query_plan_uses_index_strategy() {
        let root = tempdir().expect("tempdir");
        let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
        let plans = explain_region_query_plan(&run.sqlite_path).expect("plan");
        let joined = plans.join("\n").to_ascii_lowercase();
        assert!(
            joined.contains("index") || joined.contains("rtree"),
            "expected index/rtree usage in plan: {joined}"
        );
    }

    #[test]
    fn ingest_sqlite_meta_includes_build_pragmas() {
        let root = tempdir().expect("tempdir");
        let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
        let conn = rusqlite::Connection::open(run.sqlite_path).expect("open sqlite");
        let schema_version: String = conn
            .query_row(
                "SELECT v FROM atlas_meta WHERE k='schema_version'",
                [],
                |r| r.get(0),
            )
            .expect("schema_version");
        let journal_mode: String = conn
            .query_row(
                "SELECT v FROM atlas_meta WHERE k='ingest_journal_mode'",
                [],
                |r| r.get(0),
            )
            .expect("journal mode");
        assert_eq!(schema_version, "3");
        assert_eq!(journal_mode, "WAL");
        let schema_table_version: i64 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .expect("schema_version table");
        assert_eq!(schema_table_version, 3);
        let contigs: i64 = conn
            .query_row("SELECT COUNT(*) FROM contigs", [], |r| r.get(0))
            .expect("contigs table count");
        assert!(contigs > 0);
    }

    #[test]
    fn fixture_matrix_edgecases_runs_leniently() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/edgecases");
        let mut count = 0usize;
        let mut succeeded = 0usize;
        for entry in std::fs::read_dir(dir).expect("read edgecases") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|x| x.to_str()) != Some("gff3") {
                continue;
            }
            let root = tempdir().expect("tempdir");
            let mut o = opts(root.path(), StrictnessMode::Lenient);
            o.gff3_path = path;
            match ingest_dataset(&o) {
                Ok(_) => succeeded += 1,
                Err(err) => {
                    let msg = err.to_string();
                    assert!(
                        msg.contains("gene_count must be > 0")
                            || msg.contains("contig")
                            || msg.contains("invalid"),
                        "unexpected edgecase failure: {msg}"
                    );
                }
            }
            count += 1;
        }
        assert!(count >= 10, "expected edgecase fixture matrix coverage");
        assert!(
            succeeded >= 6,
            "expected most edgecases to ingest successfully"
        );
    }

    #[test]
    fn realistic_fixture_smoke_is_deterministic() {
        let realistic = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/realistic");
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Lenient);
        o.gff3_path = realistic.join("genes.gff3");
        o.fasta_path = realistic.join("genome.fa");
        o.fai_path = realistic.join("genome.fa.fai");
        let r1 = ingest_dataset(&o).expect("realistic run1");

        let alt = tempdir().expect("tempdir2");
        o.output_root = alt.path().to_path_buf();
        let r2 = ingest_dataset(&o).expect("realistic run2");

        assert_eq!(
            r1.manifest.checksums.sqlite_sha256,
            r2.manifest.checksums.sqlite_sha256
        );
        let i1 = std::fs::read_to_string(r1.release_gene_index_path).expect("index1");
        let i2 = std::fs::read_to_string(r2.release_gene_index_path).expect("index2");
        assert_eq!(i1, i2);
    }

    #[test]
    fn sharded_ingest_emits_catalog_and_shards() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.emit_shards = true;
        o.shard_partitions = 0;
        let run = ingest_dataset(&o).expect("sharded ingest");
        let catalog_path = run.shard_catalog_path.expect("catalog path");
        assert!(catalog_path.exists());
        let catalog = run.shard_catalog.expect("catalog");
        assert!(!catalog.shards.is_empty());
        for shard in &catalog.shards {
            let shard_file = run
                .sqlite_path
                .parent()
                .expect("derived dir")
                .join(&shard.sqlite_path);
            assert!(
                shard_file.exists(),
                "missing shard file {}",
                shard_file.display()
            );
        }
    }

    #[test]
    fn unknown_feature_policy_can_reject() {
        let root = tempdir().expect("tempdir");
        let gff = root.path().join("unknown_feature.gff3");
        std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\trepeat_region\t1\t10\t.\t+\t.\tID=r1\n",
        )
        .expect("write gff3");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = gff;
        o.unknown_feature_policy = UnknownFeaturePolicy::Reject;
        let err = ingest_dataset(&o).expect_err("unknown feature should fail");
        assert!(err.to_string().contains("unknown GFF3 feature type"));
    }

    #[test]
    fn transcript_id_policy_supports_transcript_id_attribute() {
        let root = tempdir().expect("tempdir");
        let gff = root.path().join("transcript_id_variant.gff3");
        std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t50\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\ttranscript\t1\t50\t.\t+\t.\ttranscript_id=tx1;Parent=g1\n",
        )
        .expect("write gff3");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = gff;
        let run = ingest_dataset(&o).expect("ingest with transcript_id attr");
        assert_eq!(run.manifest.stats.transcript_count, 1);
    }

    #[test]
    fn feature_id_uniqueness_policy_normalized_rejects_case_collisions() {
        let root = tempdir().expect("tempdir");
        let gff = root.path().join("id_case_collision.gff3");
        std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t50\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\ttranscript\t1\t50\t.\t+\t.\tID=Tx1;Parent=g1\nchr1\tsrc\ttranscript\t2\t40\t.\t+\t.\tID=tx1;Parent=g1\n",
        )
        .expect("write gff3");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = gff;
        o.feature_id_uniqueness_policy = FeatureIdUniquenessPolicy::NormalizeAsciiLowercaseReject;
        let err = ingest_dataset(&o).expect_err("case-colliding IDs must fail");
        assert!(err.to_string().contains("duplicate feature ID"));
    }

    #[test]
    fn tiny_fixture_matches_cross_machine_golden_hashes() {
        let root = tempdir().expect("tempdir");
        let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
        assert_eq!(
            run.manifest.checksums.sqlite_sha256,
            "b6161b3a91ea657e510cf0d9202fab01f18eedfd2f2a62c3ac2ec5020b3c3361"
        );
        assert_eq!(
            run.manifest.dataset_signature_sha256,
            "d8ec88ad2dd813f4c53bf8b5f13b6026a6f91a11d018df0bb482b02e7acc7100"
        );
    }

    #[test]
    fn strict_mode_rejects_invalid_strand() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/policies/invalid_strand.gff3");
        let err = ingest_dataset(&o).expect_err("invalid strand must fail");
        assert!(err.to_string().contains("GFF3_INVALID_STRAND"));
    }

    #[test]
    fn strict_mode_rejects_invalid_cds_phase() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/policies/invalid_cds_phase.gff3");
        let err = ingest_dataset(&o).expect_err("invalid phase must fail");
        assert!(err.to_string().contains("GFF3_INVALID_PHASE"));
    }

    #[test]
    fn duplicate_transcript_policy_rejects_in_strict_mode() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/policies/duplicate_transcript_ids.gff3");
        assert!(ingest_dataset(&o).is_err());
    }

    #[test]
    fn duplicate_transcript_policy_can_dedupe() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Lenient);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/policies/duplicate_transcript_ids.gff3");
        o.duplicate_transcript_id_policy = DuplicateTranscriptIdPolicy::DedupeKeepLexicographicallySmallest;
        let run = ingest_dataset(&o).expect("dedupe policy should pass");
        let conn = rusqlite::Connection::open(run.sqlite_path).expect("open sqlite");
        let tx_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM transcript_summary", [], |r| r.get(0))
            .expect("tx count");
        assert_eq!(tx_count, 1);
    }

    #[test]
    fn feature_ordering_independence_holds() {
        let root_a = tempdir().expect("tempdir");
        let run_a = ingest_dataset(&opts(root_a.path(), StrictnessMode::Strict)).expect("baseline");
        let root_b = tempdir().expect("tempdir2");
        let mut o = opts(root_b.path(), StrictnessMode::Strict);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/policies/unordered_features.gff3");
        let run_b1 = ingest_dataset(&o).expect("unordered ingest #1");
        let root_c = tempdir().expect("tempdir3");
        o.output_root = root_c.path().to_path_buf();
        let run_b2 = ingest_dataset(&o).expect("unordered ingest #2");
        assert_eq!(
            run_b1.manifest.checksums.sqlite_sha256,
            run_b2.manifest.checksums.sqlite_sha256
        );
        assert_eq!(run_a.manifest.stats, run_b1.manifest.stats);
    }

    #[test]
    fn report_contains_structured_rejections() {
        let root = tempdir().expect("tempdir");
        let gff = root.path().join("unknown_feature_lenient.gff3");
        std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\trepeat_region\t1\t10\t.\t+\t.\tID=r1\n",
        )
        .expect("write gff3");
        let mut o = opts(root.path(), StrictnessMode::Lenient);
        o.gff3_path = gff;
        let run = ingest_dataset(&o).expect("lenient ingest");
        assert!(!run.anomaly_report.rejections.is_empty());
        assert_eq!(run.anomaly_report.rejections[0].code, "GFF3_UNKNOWN_FEATURE");
    }

    #[test]
    fn manifest_stores_contig_normalization_aliases() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.seqid_policy =
            SeqidNormalizationPolicy::from_aliases(std::collections::BTreeMap::from([(
                "1".to_string(),
                "chr1".to_string(),
            )]));
        let run = ingest_dataset(&o).expect("ingest");
        assert_eq!(
            run.manifest
                .contig_normalization_aliases
                .get("1")
                .map(String::as_str),
            Some("chr1")
        );
    }
}
