#![forbid(unsafe_code)]

use bijux_atlas_core::canonical;
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    artifact_paths, ArtifactChecksums, ArtifactManifest, BiotypePolicy, DatasetId,
    DuplicateGeneIdPolicy, GeneIdentifierPolicy, GeneNamePolicy, IngestAnomalyReport,
    ManifestStats, SeqidNormalizationPolicy, StrictnessMode, TranscriptTypePolicy, ValidationError,
};
use rusqlite::{params, Connection};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{BufRead, BufReader};
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
}

impl Default for IngestOptions {
    fn default() -> Self {
        Self {
            gff3_path: PathBuf::new(),
            fasta_path: PathBuf::new(),
            fai_path: PathBuf::new(),
            output_root: PathBuf::new(),
            dataset: DatasetId {
                release: "0".to_string(),
                species: "unknown".to_string(),
                assembly: "unknown".to_string(),
            },
            strictness: StrictnessMode::Strict,
            duplicate_gene_id_policy: DuplicateGeneIdPolicy::Fail,
            gene_identifier_policy: GeneIdentifierPolicy::Gff3Id,
            gene_name_policy: GeneNamePolicy::default(),
            biotype_policy: BiotypePolicy::default(),
            transcript_type_policy: TranscriptTypePolicy::default(),
            seqid_policy: SeqidNormalizationPolicy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IngestResult {
    pub manifest_path: PathBuf,
    pub sqlite_path: PathBuf,
    pub anomaly_report_path: PathBuf,
    pub manifest: ArtifactManifest,
    pub anomaly_report: IngestAnomalyReport,
}

#[derive(Debug, Clone)]
struct GeneRecord {
    gene_id: String,
    gene_name: String,
    biotype: String,
    seqid: String,
    start: u64,
    end: u64,
    transcript_count: u64,
    sequence_length: u64,
}

pub fn ingest_dataset(opts: &IngestOptions) -> Result<IngestResult, IngestError> {
    let contig_lengths = read_fai_contig_lengths(&opts.fai_path)?;
    let mut genes: HashMap<String, Vec<GeneRecord>> = HashMap::new();
    let mut transcript_parents: Vec<String> = Vec::new();
    let mut anomaly = IngestAnomalyReport::default();

    let mut seen_feature_ids: HashMap<String, String> = HashMap::new();
    let file = fs::File::open(&opts.gff3_path).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 9 {
            return Err(IngestError(format!(
                "invalid GFF3 row (expected 9 columns): {line}"
            )));
        }

        let raw_seqid = cols[0].trim().to_string();
        let seqid = opts.seqid_policy.normalize(&raw_seqid);
        let feature_type = cols[2].trim().to_string();
        let start: u64 = cols[3]
            .parse()
            .map_err(|_| IngestError(format!("invalid start coordinate: {}", cols[3])))?;
        let end: u64 = cols[4]
            .parse()
            .map_err(|_| IngestError(format!("invalid end coordinate: {}", cols[4])))?;
        if start == 0 || end < start {
            return Err(IngestError(format!(
                "invalid coordinate span: {start}-{end}"
            )));
        }

        let attrs = parse_attributes(cols[8]);
        if let Some(fid) = attrs.get("ID") {
            if let Some(previous_kind) = seen_feature_ids.get(fid) {
                if previous_kind != &feature_type {
                    anomaly.overlapping_ids.push(fid.clone());
                }
            } else {
                seen_feature_ids.insert(fid.clone(), feature_type.clone());
            }
        }

        if feature_type == "gene" {
            let gff3_id = attrs
                .get("ID")
                .cloned()
                .ok_or_else(|| IngestError("gene feature missing ID attribute".to_string()))?;
            let gene_id = opts
                .gene_identifier_policy
                .resolve(
                    &attrs,
                    &gff3_id,
                    matches!(opts.strictness, StrictnessMode::Strict),
                )
                .map_err(|e| IngestError(e.to_string()))?;

            let Some(contig_len) = contig_lengths.get(&seqid) else {
                anomaly.unknown_contigs.push(seqid.clone());
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(format!("contig not found in FAI: {seqid}")));
                }
                continue;
            };
            if end > *contig_len {
                let msg = format!(
                    "gene {gene_id} coordinate end {end} exceeds contig {seqid} length {contig_len}"
                );
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(msg));
                }
                anomaly.unknown_contigs.push(seqid.clone());
                continue;
            }

            let record = GeneRecord {
                gene_id: gene_id.clone(),
                gene_name: opts.gene_name_policy.resolve(&attrs, &gene_id),
                biotype: opts.biotype_policy.resolve(&attrs),
                seqid,
                start,
                end,
                transcript_count: 0,
                sequence_length: end - start + 1,
            };
            genes.entry(gene_id).or_default().push(record);
        } else if opts.transcript_type_policy.accepts(&feature_type) {
            let Some(parent_attr) = attrs.get("Parent") else {
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(
                        "transcript feature missing Parent attribute".to_string(),
                    ));
                }
                anomaly
                    .missing_parents
                    .push("<missing Parent attr>".to_string());
                continue;
            };
            for p in parent_attr.split(',') {
                transcript_parents.push(p.trim().to_string());
            }
        }
    }

    let mut deduped: HashMap<String, GeneRecord> = HashMap::new();
    let mut keys: Vec<String> = genes.keys().cloned().collect();
    keys.sort();
    for key in keys {
        let Some(mut candidates) = genes.remove(&key) else {
            continue;
        };
        if candidates.len() > 1 {
            anomaly.duplicate_gene_ids.push(key.clone());
            match opts.duplicate_gene_id_policy {
                DuplicateGeneIdPolicy::Fail => {
                    if matches!(opts.strictness, StrictnessMode::Strict) {
                        return Err(IngestError(format!("duplicate gene_id: {key}")));
                    }
                }
                DuplicateGeneIdPolicy::DedupeKeepLexicographicallySmallest => {
                    candidates.sort_by(|a, b| {
                        a.seqid
                            .cmp(&b.seqid)
                            .then(a.start.cmp(&b.start))
                            .then(a.end.cmp(&b.end))
                            .then(a.gene_name.cmp(&b.gene_name))
                            .then(a.biotype.cmp(&b.biotype))
                    });
                }
                _ => {
                    if matches!(opts.strictness, StrictnessMode::Strict) {
                        return Err(IngestError(
                            "unsupported duplicate gene_id policy variant".to_string(),
                        ));
                    }
                }
            }
        }
        if let Some(first) = candidates.into_iter().next() {
            deduped.insert(key, first);
        }
    }

    for parent in transcript_parents {
        if let Some(gene) = deduped.get_mut(&parent) {
            gene.transcript_count += 1;
        } else {
            anomaly.missing_parents.push(parent.clone());
            if matches!(opts.strictness, StrictnessMode::Strict) {
                return Err(IngestError(format!(
                    "transcript parent {parent} does not reference a known gene"
                )));
            }
        }
    }

    anomaly.missing_parents = canonical::stable_sort_by_key(anomaly.missing_parents, |x| x.clone());
    anomaly.unknown_contigs = canonical::stable_sort_by_key(anomaly.unknown_contigs, |x| x.clone());
    anomaly.overlapping_ids = canonical::stable_sort_by_key(anomaly.overlapping_ids, |x| x.clone());
    anomaly.duplicate_gene_ids =
        canonical::stable_sort_by_key(anomaly.duplicate_gene_ids, |x| x.clone());
    anomaly.missing_parents.dedup();
    anomaly.unknown_contigs.dedup();
    anomaly.overlapping_ids.dedup();
    anomaly.duplicate_gene_ids.dedup();

    let mut gene_rows: Vec<GeneRecord> = deduped.into_values().collect();
    gene_rows.sort_by(|a, b| {
        a.seqid
            .cmp(&b.seqid)
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
            .then(a.gene_id.cmp(&b.gene_id))
    });

    let paths = artifact_paths(&opts.output_root, &opts.dataset);
    fs::create_dir_all(&paths.inputs_dir).map_err(|e| IngestError(e.to_string()))?;
    fs::create_dir_all(&paths.derived_dir).map_err(|e| IngestError(e.to_string()))?;

    fs::copy(&opts.gff3_path, &paths.gff3).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fasta_path, &paths.fasta).map_err(|e| IngestError(e.to_string()))?;
    fs::copy(&opts.fai_path, &paths.fai).map_err(|e| IngestError(e.to_string()))?;

    write_sqlite(&paths.sqlite, &gene_rows)?;

    let mut biotype_distribution: BTreeMap<String, u64> = BTreeMap::new();
    let mut contig_distribution: BTreeMap<String, u64> = BTreeMap::new();
    let mut total_transcripts = 0_u64;
    let mut contigs = BTreeSet::new();
    for g in &gene_rows {
        *biotype_distribution.entry(g.biotype.clone()).or_insert(0) += 1;
        *contig_distribution.entry(g.seqid.clone()).or_insert(0) += 1;
        total_transcripts += g.transcript_count;
        contigs.insert(g.seqid.clone());
    }

    let manifest = ArtifactManifest {
        manifest_version: "1".to_string(),
        db_schema_version: "1".to_string(),
        dataset: opts.dataset.clone(),
        checksums: ArtifactChecksums {
            gff3_sha256: sha256_hex(
                &fs::read(&paths.gff3).map_err(|e| IngestError(e.to_string()))?,
            ),
            fasta_sha256: sha256_hex(
                &fs::read(&paths.fasta).map_err(|e| IngestError(e.to_string()))?,
            ),
            fai_sha256: sha256_hex(&fs::read(&paths.fai).map_err(|e| IngestError(e.to_string()))?),
            sqlite_sha256: sha256_hex(
                &fs::read(&paths.sqlite).map_err(|e| IngestError(e.to_string()))?,
            ),
        },
        stats: ManifestStats {
            gene_count: gene_rows.len() as u64,
            transcript_count: total_transcripts,
            contig_count: contigs.len() as u64,
        },
    };
    manifest
        .validate_strict()
        .map_err(|e: ValidationError| IngestError(e.to_string()))?;

    let manifest_bytes =
        canonical::stable_json_bytes(&manifest).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.manifest, manifest_bytes).map_err(|e| IngestError(e.to_string()))?;

    let anomaly_bytes =
        canonical::stable_json_bytes(&anomaly).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&paths.anomaly_report, anomaly_bytes).map_err(|e| IngestError(e.to_string()))?;

    eprintln!("ingest stats: genes={}", manifest.stats.gene_count);
    eprintln!(
        "ingest stats: transcripts={}",
        manifest.stats.transcript_count
    );
    eprintln!("ingest stats: contigs={}", manifest.stats.contig_count);
    eprintln!(
        "ingest stats: biotype_distribution={:?}",
        biotype_distribution
    );
    eprintln!(
        "ingest stats: contig_distribution={:?}",
        contig_distribution
    );

    Ok(IngestResult {
        manifest_path: paths.manifest,
        sqlite_path: paths.sqlite,
        anomaly_report_path: paths.anomaly_report,
        manifest,
        anomaly_report: anomaly,
    })
}

fn parse_attributes(raw: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for token in raw.split(';') {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        if let Some((k, v)) = t.split_once('=') {
            out.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

pub fn read_fai_contig_lengths(path: &Path) -> Result<BTreeMap<String, u64>, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut out = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 2 {
            return Err(IngestError(format!("invalid FAI line: {line}")));
        }
        let len: u64 = cols[1]
            .parse()
            .map_err(|_| IngestError(format!("invalid FAI contig length: {}", cols[1])))?;
        out.insert(cols[0].to_string(), len);
    }
    Ok(out)
}

fn write_sqlite(path: &Path, genes: &[GeneRecord]) -> Result<(), IngestError> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| IngestError(e.to_string()))?;
    }
    let mut conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    conn.execute_batch(
        "
        PRAGMA journal_mode=DELETE;
        PRAGMA synchronous=FULL;
        PRAGMA temp_store=MEMORY;
        CREATE TABLE gene_summary (
          id INTEGER PRIMARY KEY,
          gene_id TEXT NOT NULL,
          name TEXT NOT NULL,
          biotype TEXT NOT NULL,
          seqid TEXT NOT NULL,
          start INTEGER NOT NULL,
          end INTEGER NOT NULL,
          transcript_count INTEGER NOT NULL,
          sequence_length INTEGER NOT NULL
        );
        CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(
          gene_rowid,
          start,
          end
        );
        ",
    )
    .map_err(|e| IngestError(e.to_string()))?;

    let tx = conn.transaction().map_err(|e| IngestError(e.to_string()))?;
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO gene_summary (
                  id, gene_id, name, biotype, seqid, start, end, transcript_count, sequence_length
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .map_err(|e| IngestError(e.to_string()))?;
        let mut rtree_stmt = tx
            .prepare("INSERT INTO gene_summary_rtree (gene_rowid, start, end) VALUES (?1, ?2, ?3)")
            .map_err(|e| IngestError(e.to_string()))?;

        for (idx, g) in genes.iter().enumerate() {
            let rowid = (idx + 1) as i64;
            stmt.execute(params![
                rowid,
                g.gene_id,
                g.gene_name,
                g.biotype,
                g.seqid,
                g.start as i64,
                g.end as i64,
                g.transcript_count as i64,
                g.sequence_length as i64
            ])
            .map_err(|e| IngestError(e.to_string()))?;
            rtree_stmt
                .execute(params![rowid, g.start as f64, g.end as f64])
                .map_err(|e| IngestError(e.to_string()))?;
        }
    }

    tx.execute_batch(
        "
        CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
        CREATE INDEX idx_gene_summary_name ON gene_summary(name);
        CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
        ",
    )
    .map_err(|e| IngestError(e.to_string()))?;

    tx.commit().map_err(|e| IngestError(e.to_string()))?;
    conn.execute_batch("VACUUM;")
        .map_err(|e| IngestError(e.to_string()))?;
    Ok(())
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
        assert_eq!(
            serde_json::to_string(&run1.manifest).expect("serialize run1 manifest"),
            serde_json::to_string(&run2.manifest).expect("serialize run2 manifest")
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
    fn multiple_parents_count_transcript_for_each_parent() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Lenient);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/minimal/genes_multiple_parents.gff3");
        let result = ingest_dataset(&o).expect("lenient ingest");
        assert_eq!(result.manifest.stats.transcript_count, 2);
    }

    #[test]
    fn transcript_type_mismatch_is_ignored_by_policy() {
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/minimal/genes_transcript_type_mismatch.gff3");
        let result = ingest_dataset(&o).expect("ingest mismatch should still pass");
        assert_eq!(result.manifest.stats.transcript_count, 0);
    }
}
