// SPDX-License-Identifier: Apache-2.0

pub(super) use super::*;
pub(super) use crate::domain::sha256_hex;
pub(super) use tempfile::tempdir;

pub(super) fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny")
}

pub(super) fn read_canonical_summary(run: &IngestResult) -> serde_json::Value {
    let path = run
        .manifest_path
        .parent()
        .expect("manifest dir")
        .join("canonical_summary.json");
    serde_json::from_slice(&std::fs::read(path).expect("read canonical summary"))
        .expect("parse canonical summary")
}

pub(super) fn sqlite_logical_fingerprint(path: &Path) -> String {
    use rusqlite::types::ValueRef;

    fn render_value(value: ValueRef<'_>) -> String {
        match value {
            ValueRef::Null => "null".to_string(),
            ValueRef::Integer(v) => v.to_string(),
            ValueRef::Real(v) => format!("{v:.17}"),
            ValueRef::Text(v) => String::from_utf8_lossy(v).into_owned(),
            ValueRef::Blob(v) => hex::encode(v),
        }
    }

    fn collect_rows(conn: &rusqlite::Connection, sql: &str) -> Vec<String> {
        let mut stmt = conn.prepare(sql).expect("prepare fingerprint query");
        let cols = stmt.column_count();
        stmt.query_map([], |row| {
            let mut values = Vec::with_capacity(cols);
            for idx in 0..cols {
                values.push(render_value(row.get_ref(idx)?));
            }
            Ok(values.join("|"))
        })
        .expect("run fingerprint query")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect fingerprint rows")
    }

    let conn = rusqlite::Connection::open(path).expect("open sqlite for fingerprint");
    let sections = [
        (
            "sqlite_master",
            "SELECT type, name, COALESCE(sql, '') FROM sqlite_master \
             WHERE type IN ('table','index','trigger','view') AND name NOT LIKE 'sqlite_%' \
             ORDER BY type, name",
        ),
        (
            "schema_version",
            "SELECT version FROM schema_version ORDER BY version",
        ),
        (
            "atlas_meta",
            "SELECT k, v FROM atlas_meta \
             WHERE k NOT IN ('created_by') \
             ORDER BY k",
        ),
        (
            "gene_summary",
            "SELECT id, gene_id, name, name_normalized, biotype, seqid, start, end, \
             transcript_count, exon_count, total_exon_span, cds_present, sequence_length \
             FROM gene_summary ORDER BY id",
        ),
        (
            "genes",
            "SELECT id, gene_id, name, name_normalized, biotype, seqid, start, end, \
             transcript_count, exon_count, total_exon_span, cds_present, sequence_length \
             FROM genes ORDER BY id",
        ),
        (
            "transcript_summary",
            "SELECT id, transcript_id, parent_gene_id, transcript_type, biotype, seqid, \
             start, end, exon_count, total_exon_span, cds_present \
             FROM transcript_summary ORDER BY id",
        ),
        (
            "transcripts",
            "SELECT id, transcript_id, parent_gene_id, transcript_type, biotype, seqid, \
             start, end, exon_count, total_exon_span, cds_present, sequence_length, \
             COALESCE(spliced_length, -1), COALESCE(cds_length, -1) \
             FROM transcripts ORDER BY id",
        ),
        (
            "exons",
            "SELECT id, exon_id, transcript_id, seqid, start, end, exon_length \
             FROM exons ORDER BY id",
        ),
        (
            "transcript_exon_map",
            "SELECT transcript_id, exon_id FROM transcript_exon_map \
             ORDER BY transcript_id, exon_id",
        ),
        (
            "gene_summary_rtree",
            "SELECT gene_rowid, start, end FROM gene_summary_rtree ORDER BY gene_rowid",
        ),
        (
            "contigs",
            "SELECT name, length, gc_fraction, n_fraction FROM contigs ORDER BY name",
        ),
        (
            "dataset_stats",
            "SELECT dimension, value, gene_count FROM dataset_stats ORDER BY dimension, value",
        ),
    ];
    let mut lines = Vec::new();
    for (name, sql) in sections {
        lines.push(format!("[{name}]"));
        lines.extend(collect_rows(&conn, sql));
    }
    sha256_hex(lines.join("\n").as_bytes())
}

pub(super) fn opts(root: &Path, strictness: StrictnessMode) -> IngestOptions {
    IngestOptions {
        gff3_path: fixture_dir().join("genes.gff3"),
        fasta_path: fixture_dir().join("genome.fa"),
        fai_path: fixture_dir().join("genome.fa.fai"),
        output_root: root.to_path_buf(),
        build_hash: String::new(),
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
