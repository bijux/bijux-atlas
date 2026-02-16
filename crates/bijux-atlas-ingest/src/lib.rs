#![forbid(unsafe_code)]

mod extract;
mod fai;
mod gff3;
mod manifest;
mod sqlite;

use bijux_atlas_model::{
    artifact_paths, BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, GeneIdentifierPolicy,
    GeneNamePolicy, IngestAnomalyReport, SeqidNormalizationPolicy, StrictnessMode,
    TranscriptTypePolicy,
};
use extract::extract_gene_rows;
use gff3::parse_gff3_records;
use manifest::{build_and_write_manifest_and_reports, BuildManifestArgs};
use sqlite::{explain_plan_for_region_query, write_sqlite};
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
    pub gene_identifier_policy: GeneIdentifierPolicy,
    pub gene_name_policy: GeneNamePolicy,
    pub biotype_policy: BiotypePolicy,
    pub transcript_type_policy: TranscriptTypePolicy,
    pub seqid_policy: SeqidNormalizationPolicy,
    pub max_threads: usize,
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
            gene_identifier_policy: GeneIdentifierPolicy::Gff3Id,
            gene_name_policy: GeneNamePolicy::default(),
            biotype_policy: BiotypePolicy::default(),
            transcript_type_policy: TranscriptTypePolicy::default(),
            seqid_policy: SeqidNormalizationPolicy::default(),
            max_threads: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IngestResult {
    pub manifest_path: PathBuf,
    pub sqlite_path: PathBuf,
    pub anomaly_report_path: PathBuf,
    pub qc_report_path: PathBuf,
    pub manifest: bijux_atlas_model::ArtifactManifest,
    pub anomaly_report: IngestAnomalyReport,
}

pub fn ingest_dataset(opts: &IngestOptions) -> Result<IngestResult, IngestError> {
    let _effective_threads = extract::parallelism_policy(opts.max_threads)?;

    let contig_lengths = fai::read_fai_contig_lengths(&opts.fai_path)?;
    let records = parse_gff3_records(&opts.gff3_path)?;
    let extracted = extract_gene_rows(records, &contig_lengths, opts)?;

    let paths = artifact_paths(&opts.output_root, &opts.dataset);
    fs::create_dir_all(&paths.inputs_dir).map_err(|e| IngestError(e.to_string()))?;
    fs::create_dir_all(&paths.derived_dir).map_err(|e| IngestError(e.to_string()))?;

    fs::copy(&opts.gff3_path, &paths.gff3).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fasta_path, &paths.fasta).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fai_path, &paths.fai).map_err(|e| IngestError(e.to_string()))?;

    write_sqlite(&paths.sqlite, &extracted.gene_rows)?;
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
    })?;

    Ok(IngestResult {
        manifest_path: paths.manifest,
        sqlite_path: paths.sqlite,
        anomaly_report_path: paths.anomaly_report,
        qc_report_path: built.qc_report_path,
        manifest: built.manifest,
        anomaly_report: extracted.anomaly,
    })
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
            gene_identifier_policy: GeneIdentifierPolicy::Gff3Id,
            gene_name_policy: GeneNamePolicy::default(),
            biotype_policy: BiotypePolicy::default(),
            transcript_type_policy: TranscriptTypePolicy::default(),
            seqid_policy: SeqidNormalizationPolicy::default(),
            max_threads: 1,
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
        assert_eq!(schema_version, "2");
        assert_eq!(journal_mode, "WAL");
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
    }
}
