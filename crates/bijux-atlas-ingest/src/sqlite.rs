use crate::extract::{ExonRecord, GeneRecord, TranscriptRecord};
use crate::fai::ContigStats;
use crate::IngestError;
use bijux_atlas_core::{canonical, sha256_hex};
use bijux_atlas_model::{DatasetId, ShardCatalog, ShardEntry, ShardingPlan};
use rusqlite::{params, Connection};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const SQLITE_SCHEMA_VERSION: i64 = 4;
pub const SQLITE_SCHEMA_SSOT: &str = include_str!("../sql/schema_v4.sql");
#[allow(dead_code)] // ATLAS-EXC-0001
pub const SQLITE_SCHEMA_SSOT_SHA256: &str =
    "a695a4e39b45e4fd87491dd9a55817142059100480d77b59168db0f5fe0a6901";
#[allow(dead_code)] // ATLAS-EXC-0001
pub const SQLITE_REQUIRED_INDEXES: &[&str] = &[
    "idx_gene_summary_gene_id",
    "idx_gene_summary_name",
    "idx_gene_summary_biotype",
    "idx_gene_summary_cover_region",
    "idx_gene_summary_region",
    "idx_genes_gene_id",
    "idx_genes_name",
    "idx_genes_biotype",
    "idx_genes_order_page",
    "idx_transcripts_parent_gene",
    "idx_exons_transcript",
];
const INGEST_JOURNAL_MODE: &str = "WAL";
const INGEST_LOCKING_MODE: &str = "EXCLUSIVE";
const INGEST_PAGE_SIZE: i64 = 4096;
const INGEST_MMAP_SIZE: i64 = 268_435_456;

pub struct WriteSqliteInput<'a> {
    pub path: &'a Path,
    pub dataset: &'a DatasetId,
    pub genes: &'a [GeneRecord],
    pub transcripts: &'a [TranscriptRecord],
    pub exons: &'a [ExonRecord],
    pub contigs: &'a BTreeMap<String, ContigStats>,
    pub gff3_sha256: &'a str,
    pub fasta_sha256: &'a str,
    pub fai_sha256: &'a str,
}

#[allow(dead_code)] // ATLAS-EXC-0001
pub fn migrate_forward_schema(conn: &Connection, target_version: i64) -> Result<i64, IngestError> {
    let current = detect_schema_version(conn)?;
    if current > target_version {
        return Err(IngestError(format!(
            "forward-only schema migration violation: current={current}, target={target_version}"
        )));
    }
    if current < 4 && target_version >= 4 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_version (
              version INTEGER PRIMARY KEY
            ) WITHOUT ROWID;
            DELETE FROM schema_version;
            INSERT INTO schema_version (version) VALUES (4);
            PRAGMA user_version=4;
            ",
        )
        .map_err(|e| IngestError(e.to_string()))?;
        let _ = conn.execute("UPDATE atlas_meta SET v='4' WHERE k='schema_version'", []);
    }
    Ok(target_version.max(current))
}

#[allow(dead_code)] // ATLAS-EXC-0001
fn detect_schema_version(conn: &Connection) -> Result<i64, IngestError> {
    let has_schema_table: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| IngestError(e.to_string()))?;
    if has_schema_table > 0 {
        let v: i64 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| IngestError(e.to_string()))?;
        return Ok(v);
    }
    let pragma_v: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| IngestError(e.to_string()))?;
    Ok(pragma_v)
}

pub fn write_sqlite(input: WriteSqliteInput<'_>) -> Result<(), IngestError> {
    let WriteSqliteInput {
        path,
        dataset,
        genes,
        transcripts,
        exons,
        contigs,
        gff3_sha256,
        fasta_sha256,
        fai_sha256,
    } = input;
    if path.exists() {
        fs::remove_file(path).map_err(|e| IngestError(e.to_string()))?;
    }
    let mut conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    conn.execute_batch(SQLITE_SCHEMA_SSOT)
        .map_err(|e| IngestError(e.to_string()))?;
    conn.execute_batch(&format!("PRAGMA user_version={};", SQLITE_SCHEMA_VERSION))
        .map_err(|e| IngestError(e.to_string()))?;

    let tx = conn.transaction().map_err(|e| IngestError(e.to_string()))?;
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO gene_summary (
                  id, gene_id, name, name_normalized, biotype, seqid, start, end, transcript_count, exon_count, total_exon_span, cds_present, sequence_length
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            )
            .map_err(|e| IngestError(e.to_string()))?;
        let mut tx_stmt = tx
            .prepare(
                "INSERT INTO transcript_summary (
                  id, transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )
            .map_err(|e| IngestError(e.to_string()))?;
        let mut tx_v2_stmt = tx
            .prepare(
                "INSERT INTO transcripts (
                  id, transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present, sequence_length, spliced_length, cds_length
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            )
            .map_err(|e| IngestError(e.to_string()))?;
        let mut gene_v2_stmt = tx
            .prepare(
                "INSERT INTO genes (
                  id, gene_id, name, name_normalized, biotype, seqid, start, end, transcript_count, exon_count, total_exon_span, cds_present, sequence_length
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
                g.gene_name.to_ascii_lowercase(),
                g.biotype,
                g.seqid,
                g.start as i64,
                g.end as i64,
                g.transcript_count as i64,
                g.exon_count as i64,
                g.total_exon_span as i64,
                if g.cds_present { 1 } else { 0 },
                g.sequence_length as i64
            ])
            .map_err(|e| IngestError(e.to_string()))?;
            gene_v2_stmt
                .execute(params![
                    rowid,
                    g.gene_id,
                    g.gene_name,
                    g.gene_name.to_ascii_lowercase(),
                    g.biotype,
                    g.seqid,
                    g.start as i64,
                    g.end as i64,
                    g.transcript_count as i64,
                    g.exon_count as i64,
                    g.total_exon_span as i64,
                    if g.cds_present { 1 } else { 0 },
                    g.sequence_length as i64
                ])
                .map_err(|e| IngestError(e.to_string()))?;
            rtree_stmt
                .execute(params![rowid, g.start as f64, g.end as f64])
                .map_err(|e| IngestError(e.to_string()))?;
        }
        let mut contig_stmt = tx
            .prepare("INSERT INTO contigs (name, length, gc_fraction, n_fraction) VALUES (?1, ?2, ?3, ?4)")
            .map_err(|e| IngestError(e.to_string()))?;
        for (name, s) in contigs {
            contig_stmt
                .execute(params![name, s.length as i64, s.gc_fraction, s.n_fraction])
                .map_err(|e| IngestError(e.to_string()))?;
        }

        for (idx, txrow) in transcripts.iter().enumerate() {
            let rowid = (idx + 1) as i64;
            tx_stmt
                .execute(params![
                    rowid,
                    txrow.transcript_id,
                    txrow.parent_gene_id,
                    txrow.transcript_type,
                    txrow.biotype,
                    txrow.seqid,
                    txrow.start as i64,
                    txrow.end as i64,
                    txrow.exon_count as i64,
                    txrow.total_exon_span as i64,
                    if txrow.cds_present { 1 } else { 0 },
                ])
                .map_err(|e| IngestError(e.to_string()))?;
            tx_v2_stmt
                .execute(params![
                    rowid,
                    txrow.transcript_id,
                    txrow.parent_gene_id,
                    txrow.transcript_type,
                    txrow.biotype,
                    txrow.seqid,
                    txrow.start as i64,
                    txrow.end as i64,
                    txrow.exon_count as i64,
                    txrow.total_exon_span as i64,
                    if txrow.cds_present { 1 } else { 0 },
                    txrow.sequence_length as i64,
                    txrow.spliced_length.map(|v| v as i64),
                    txrow.cds_span_length.map(|v| v as i64),
                ])
                .map_err(|e| IngestError(e.to_string()))?;
        }

        let mut exon_stmt = tx
            .prepare(
                "INSERT INTO exons (id, exon_id, transcript_id, seqid, start, end, exon_length) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .map_err(|e| IngestError(e.to_string()))?;
        let mut tx_exon_map_stmt = tx
            .prepare("INSERT OR IGNORE INTO transcript_exon_map (transcript_id, exon_id) VALUES (?1, ?2)")
            .map_err(|e| IngestError(e.to_string()))?;
        for (idx, ex) in exons.iter().enumerate() {
            let rowid = (idx + 1) as i64;
            exon_stmt
                .execute(params![
                    rowid,
                    ex.exon_id,
                    ex.transcript_id,
                    ex.seqid,
                    ex.start as i64,
                    ex.end as i64,
                    ex.exon_length as i64
                ])
                .map_err(|e| IngestError(e.to_string()))?;
            tx_exon_map_stmt
                .execute(params![ex.transcript_id, ex.exon_id])
                .map_err(|e| IngestError(e.to_string()))?;
        }

        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('schema_version', ?1)",
            params![SQLITE_SCHEMA_VERSION.to_string()],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            params![SQLITE_SCHEMA_VERSION],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('ingest_journal_mode', ?1)",
            params![INGEST_JOURNAL_MODE],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('ingest_locking_mode', ?1)",
            params![INGEST_LOCKING_MODE],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('ingest_page_size', ?1)",
            params![INGEST_PAGE_SIZE.to_string()],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('ingest_mmap_size', ?1)",
            params![INGEST_MMAP_SIZE.to_string()],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('dataset_release', ?1)",
            params![dataset.release.as_str()],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('dataset_species', ?1)",
            params![dataset.species.as_str()],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('dataset_assembly', ?1)",
            params![dataset.assembly.as_str()],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('dataset_id', ?1)",
            params![dataset.canonical_string()],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('created_by', ?1)",
            params![format!(
                "{}@{}",
                crate::CRATE_NAME,
                env!("CARGO_PKG_VERSION")
            )],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('input_hashes', ?1)",
            params![format!(
                "gff3={gff3_sha256};fasta={fasta_sha256};fai={fai_sha256}"
            )],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('fasta_sha256', ?1)",
            params![fasta_sha256],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('fai_sha256', ?1)",
            params![fai_sha256],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('analyze_completed', 'false')",
            [],
        )
        .map_err(|e| IngestError(e.to_string()))?;
        tx.execute(
            "INSERT INTO atlas_meta (k, v) VALUES ('vacuum_completed', 'false')",
            [],
        )
        .map_err(|e| IngestError(e.to_string()))?;

        tx.execute_batch(
            "
            INSERT INTO dataset_stats (dimension, value, gene_count)
            SELECT 'biotype', biotype, COUNT(*) FROM gene_summary GROUP BY biotype;
            INSERT INTO dataset_stats (dimension, value, gene_count)
            SELECT 'seqid', seqid, COUNT(*) FROM gene_summary GROUP BY seqid;
            ",
        )
        .map_err(|e| IngestError(e.to_string()))?;
    }

    tx.commit().map_err(|e| IngestError(e.to_string()))?;
    assert_region_query_plan_uses_rtree(&conn)?;
    conn.execute_batch("ANALYZE;")
        .map_err(|e| IngestError(e.to_string()))?;
    conn.execute(
        "UPDATE atlas_meta SET v='true' WHERE k='analyze_completed'",
        [],
    )
    .map_err(|e| IngestError(e.to_string()))?;
    conn.execute_batch("VACUUM;")
        .map_err(|e| IngestError(e.to_string()))?;
    conn.execute(
        "UPDATE atlas_meta SET v='true' WHERE k='vacuum_completed'",
        [],
    )
    .map_err(|e| IngestError(e.to_string()))?;
    Ok(())
}

fn assert_region_query_plan_uses_rtree(conn: &Connection) -> Result<(), IngestError> {
    let mut stmt = conn
        .prepare("EXPLAIN QUERY PLAN SELECT g.gene_id FROM gene_summary g JOIN gene_summary_rtree r ON r.gene_rowid = g.id WHERE g.seqid=?1 AND r.start<=?2 AND r.end>=?3 ORDER BY g.seqid,g.start,g.gene_id LIMIT 10")
        .map_err(|e| IngestError(e.to_string()))?;
    let rows = stmt
        .query_map(params!["chr1", 1000_i64, 900_i64], |row| {
            row.get::<_, String>(3)
        })
        .map_err(|e| IngestError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IngestError(e.to_string()))?;
    let joined = rows.join("\n").to_ascii_lowercase();
    if joined.contains("index") || joined.contains("rtree") {
        return Ok(());
    }
    Err(IngestError(format!(
        "ingest-time EXPLAIN check failed: expected index/rtree usage in plan: {joined}"
    )))
}

pub fn write_sharded_sqlite_catalog(
    derived_dir: &Path,
    dataset: &DatasetId,
    genes: &[GeneRecord],
    transcripts: &[TranscriptRecord],
    sharding_plan: ShardingPlan,
    shard_partitions: usize,
    max_shards: usize,
) -> Result<(std::path::PathBuf, ShardCatalog), IngestError> {
    let mut buckets: BTreeMap<String, Vec<GeneRecord>> = BTreeMap::new();
    match sharding_plan {
        ShardingPlan::None => {
            return Err(IngestError(
                "sharding plan none cannot emit shards".to_string(),
            ))
        }
        ShardingPlan::Contig => {
            if shard_partitions == 0 {
                for g in genes {
                    buckets.entry(g.seqid.clone()).or_default().push(g.clone());
                }
            } else {
                for g in genes {
                    let shard = (canonical::stable_hash_hex(g.seqid.as_bytes())
                        .bytes()
                        .fold(0_u64, |acc, b| acc.wrapping_add(b as u64))
                        % shard_partitions as u64) as usize;
                    buckets
                        .entry(format!("p{:03}", shard))
                        .or_default()
                        .push(g.clone());
                }
            }
        }
        ShardingPlan::RegionGrid => {
            return Err(IngestError(
                "region_grid sharding plan is reserved for future implementation".to_string(),
            ))
        }
        _ => return Err(IngestError("unsupported sharding plan variant".to_string())),
    }

    if buckets.len() > max_shards {
        return Err(IngestError(format!(
            "shard count {} exceeds configured max_shards {}",
            buckets.len(),
            max_shards
        )));
    }

    let mut shards = Vec::new();
    for (bucket, mut rows) in buckets {
        rows.sort_by(|a, b| {
            a.seqid
                .cmp(&b.seqid)
                .then(a.start.cmp(&b.start))
                .then(a.end.cmp(&b.end))
                .then(a.gene_id.cmp(&b.gene_id))
        });
        let file_name = format!("gene_summary.{bucket}.sqlite");
        let sqlite_path = derived_dir.join(&file_name);
        let seqids = {
            let mut s: Vec<String> = rows.iter().map(|g| g.seqid.clone()).collect();
            s.sort();
            s.dedup();
            s
        };
        let tx_rows: Vec<TranscriptRecord> = transcripts
            .iter()
            .filter(|tx| seqids.contains(&tx.seqid))
            .cloned()
            .collect();
        let ex_rows: Vec<ExonRecord> = Vec::new();
        let empty_contigs = BTreeMap::new();
        write_sqlite(WriteSqliteInput {
            path: &sqlite_path,
            dataset,
            genes: &rows,
            transcripts: &tx_rows,
            exons: &ex_rows,
            contigs: &empty_contigs,
            gff3_sha256: "",
            fasta_sha256: "",
            fai_sha256: "",
        })?;
        shards.push(ShardEntry::new(
            bijux_atlas_model::ShardId::parse(
                &bucket
                    .chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() {
                            c.to_ascii_lowercase()
                        } else if c == '-' || c == '_' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>(),
            )
            .map_err(|e| IngestError(e.to_string()))?,
            seqids
                .iter()
                .map(|s| bijux_atlas_model::SeqId::parse(s).map_err(|e| IngestError(e.to_string())))
                .collect::<Result<Vec<_>, _>>()?,
            file_name,
            sha256_hex(&fs::read(&sqlite_path).map_err(|e| IngestError(e.to_string()))?),
        ));
    }
    shards.sort();
    let mode = if shard_partitions == 0 {
        "contig".to_string()
    } else {
        "contig_partitioned".to_string()
    };
    let catalog = ShardCatalog::new(dataset.clone(), mode, shards);
    catalog
        .validate_sorted()
        .map_err(|e| IngestError(e.to_string()))?;
    let catalog_path = derived_dir.join("catalog_shards.json");
    let bytes = canonical::stable_json_bytes(&catalog).map_err(|e| IngestError(e.to_string()))?;
    fs::write(&catalog_path, bytes).map_err(|e| IngestError(e.to_string()))?;
    Ok((catalog_path, catalog))
}

pub fn explain_plan_for_region_query(path: &Path) -> Result<Vec<String>, IngestError> {
    let conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    let mut stmt = conn
        .prepare("EXPLAIN QUERY PLAN SELECT g.gene_id FROM gene_summary g JOIN gene_summary_rtree r ON r.gene_rowid = g.id WHERE g.seqid=?1 AND r.start<=?2 AND r.end>=?3 ORDER BY g.seqid,g.start,g.gene_id LIMIT 10")
        .map_err(|e| IngestError(e.to_string()))?;
    let rows = stmt
        .query_map(params!["chr1", 1000_i64, 900_i64], |row| {
            let detail: String = row.get(3)?;
            Ok(detail)
        })
        .map_err(|e| IngestError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IngestError(e.to_string()))?;
    Ok(rows)
}

#[allow(dead_code)] // ATLAS-EXC-0001
pub fn explain_plan_for_gene_id_query(path: &Path) -> Result<Vec<String>, IngestError> {
    let conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    let mut stmt = conn
        .prepare("EXPLAIN QUERY PLAN SELECT gene_id FROM gene_summary WHERE gene_id=?1 ORDER BY seqid,start,gene_id LIMIT 10")
        .map_err(|e| IngestError(e.to_string()))?;
    let rows = stmt
        .query_map(params!["GENE1"], |row| row.get::<_, String>(3))
        .map_err(|e| IngestError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IngestError(e.to_string()))?;
    Ok(rows)
}

#[allow(dead_code)] // ATLAS-EXC-0001
pub fn explain_plan_for_name_query(path: &Path) -> Result<Vec<String>, IngestError> {
    let conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    let mut stmt = conn
        .prepare("EXPLAIN QUERY PLAN SELECT gene_id FROM gene_summary WHERE name=?1 ORDER BY seqid,start,gene_id LIMIT 10")
        .map_err(|e| IngestError(e.to_string()))?;
    let rows = stmt
        .query_map(params!["Gene1"], |row| row.get::<_, String>(3))
        .map_err(|e| IngestError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IngestError(e.to_string()))?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_atlas_core::sha256_hex;

    #[test]
    fn forward_only_migration_rejects_downgrade() {
        let conn = Connection::open_in_memory().expect("conn");
        conn.execute_batch("PRAGMA user_version=5;")
            .expect("set user_version");
        let err = migrate_forward_schema(&conn, 4).expect_err("downgrade must fail");
        assert!(err
            .to_string()
            .contains("forward-only schema migration violation"));
    }

    #[test]
    fn forward_migration_from_legacy_v2_adds_schema_version_table() {
        let conn = Connection::open_in_memory().expect("conn");
        conn.execute_batch(
            "
            PRAGMA user_version=2;
            CREATE TABLE atlas_meta (k TEXT PRIMARY KEY, v TEXT NOT NULL) WITHOUT ROWID;
            INSERT INTO atlas_meta (k, v) VALUES ('schema_version', '2');
            ",
        )
        .expect("legacy schema");
        migrate_forward_schema(&conn, 4).expect("migrate");
        let v: i64 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .expect("schema_version row");
        assert_eq!(v, 4);
    }

    #[test]
    fn schema_ssot_hash_is_stable() {
        assert_eq!(
            sha256_hex(SQLITE_SCHEMA_SSOT.as_bytes()),
            SQLITE_SCHEMA_SSOT_SHA256
        );
    }

    #[test]
    fn index_drift_gate_required_indexes_exist() {
        let conn = Connection::open_in_memory().expect("conn");
        conn.execute_batch(SQLITE_SCHEMA_SSOT)
            .expect("apply schema");
        for idx in SQLITE_REQUIRED_INDEXES {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                    params![idx],
                    |r| r.get(0),
                )
                .expect("index lookup");
            assert_eq!(count, 1, "missing required index: {idx}");
        }
    }

    #[test]
    fn schema_drift_gate_sqlite_master_digest_is_stable() {
        let conn = Connection::open_in_memory().expect("conn");
        conn.execute_batch(SQLITE_SCHEMA_SSOT)
            .expect("apply schema");
        let mut stmt = conn
            .prepare(
                "SELECT type, name, COALESCE(sql, '') FROM sqlite_master WHERE type IN ('table','index','trigger','view') ORDER BY type, name",
            )
            .expect("prepare");
        let rows = stmt
            .query_map([], |row| {
                let typ: String = row.get(0)?;
                let name: String = row.get(1)?;
                let sql: String = row.get(2)?;
                Ok(format!("{typ}|{name}|{sql}"))
            })
            .expect("query")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect");
        let digest = sha256_hex(rows.join("\n").as_bytes());
        assert_eq!(
            digest,
            "996a3e9bdbb5c4e65e9ef3f659a94bfe9bf4282cbd042934e6a60193cf3af41a"
        );
    }
}
