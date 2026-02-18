use crate::extract::{GeneRecord, TranscriptRecord};
use crate::IngestError;
use bijux_atlas_core::{canonical, sha256_hex};
use bijux_atlas_model::{DatasetId, ShardCatalog, ShardEntry};
use rusqlite::{params, Connection};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const SQLITE_SCHEMA_VERSION: i64 = 3;
const INGEST_JOURNAL_MODE: &str = "WAL";
const INGEST_LOCKING_MODE: &str = "EXCLUSIVE";
const INGEST_PAGE_SIZE: i64 = 4096;
const INGEST_MMAP_SIZE: i64 = 268_435_456;

#[allow(dead_code)]
pub fn migrate_forward_schema(conn: &Connection, target_version: i64) -> Result<i64, IngestError> {
    let current = detect_schema_version(conn)?;
    if current > target_version {
        return Err(IngestError(format!(
            "forward-only schema migration violation: current={current}, target={target_version}"
        )));
    }
    if current < 3 && target_version >= 3 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_version (
              version INTEGER PRIMARY KEY
            ) WITHOUT ROWID;
            DELETE FROM schema_version;
            INSERT INTO schema_version (version) VALUES (3);
            PRAGMA user_version=3;
            ",
        )
        .map_err(|e| IngestError(e.to_string()))?;
        let _ = conn.execute("UPDATE atlas_meta SET v='3' WHERE k='schema_version'", []);
    }
    Ok(target_version.max(current))
}

#[allow(dead_code)]
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

pub fn write_sqlite(
    path: &Path,
    dataset: &DatasetId,
    genes: &[GeneRecord],
    transcripts: &[TranscriptRecord],
) -> Result<(), IngestError> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| IngestError(e.to_string()))?;
    }
    let mut conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=OFF;
        PRAGMA locking_mode=EXCLUSIVE;
        PRAGMA temp_store=MEMORY;
        PRAGMA cache_size=-32000;
        PRAGMA page_size=4096;
        PRAGMA mmap_size=268435456;
        CREATE TABLE gene_summary (
          id INTEGER PRIMARY KEY,
          gene_id TEXT NOT NULL,
          name TEXT NOT NULL,
          name_normalized TEXT NOT NULL,
          biotype TEXT NOT NULL,
          seqid TEXT NOT NULL,
          start INTEGER NOT NULL,
          end INTEGER NOT NULL,
          transcript_count INTEGER NOT NULL,
          exon_count INTEGER NOT NULL DEFAULT 0,
          total_exon_span INTEGER NOT NULL DEFAULT 0,
          cds_present INTEGER NOT NULL DEFAULT 0,
          sequence_length INTEGER NOT NULL
        ) WITHOUT ROWID;
        CREATE TABLE transcript_summary (
          id INTEGER PRIMARY KEY,
          transcript_id TEXT NOT NULL UNIQUE,
          parent_gene_id TEXT NOT NULL,
          transcript_type TEXT NOT NULL,
          biotype TEXT,
          seqid TEXT NOT NULL,
          start INTEGER NOT NULL,
          end INTEGER NOT NULL,
          exon_count INTEGER NOT NULL DEFAULT 0,
          total_exon_span INTEGER NOT NULL DEFAULT 0,
          cds_present INTEGER NOT NULL DEFAULT 0
        ) WITHOUT ROWID;
        CREATE TABLE atlas_meta (
          k TEXT PRIMARY KEY,
          v TEXT NOT NULL
        ) WITHOUT ROWID;
        CREATE TABLE schema_version (
          version INTEGER PRIMARY KEY
        ) WITHOUT ROWID;
        CREATE TABLE dataset_stats (
          dimension TEXT NOT NULL,
          value TEXT NOT NULL,
          gene_count INTEGER NOT NULL,
          PRIMARY KEY (dimension, value)
        ) WITHOUT ROWID;
        CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(
          gene_rowid,
          start,
          end
        );
        ",
    )
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
            rtree_stmt
                .execute(params![rowid, g.start as f64, g.end as f64])
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

    tx.execute_batch(
        "
        CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
        CREATE INDEX idx_gene_summary_name ON gene_summary(name);
        CREATE INDEX idx_gene_summary_name_normalized ON gene_summary(name_normalized);
        CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
        CREATE INDEX idx_gene_summary_cover_lookup ON gene_summary(gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length);
        CREATE INDEX idx_gene_summary_cover_region ON gene_summary(seqid, start, gene_id, end, name, biotype, transcript_count, sequence_length);
        CREATE INDEX idx_transcript_summary_transcript_id ON transcript_summary(transcript_id);
        CREATE INDEX idx_transcript_summary_parent_gene_id ON transcript_summary(parent_gene_id);
        CREATE INDEX idx_transcript_summary_biotype ON transcript_summary(biotype);
        CREATE INDEX idx_transcript_summary_type ON transcript_summary(transcript_type);
        CREATE INDEX idx_transcript_summary_region ON transcript_summary(seqid, start, end);
        ",
    )
    .map_err(|e| IngestError(e.to_string()))?;

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
    shard_partitions: usize,
) -> Result<(std::path::PathBuf, ShardCatalog), IngestError> {
    let mut buckets: BTreeMap<String, Vec<GeneRecord>> = BTreeMap::new();
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
        let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
        write_sqlite(&sqlite_path, &dataset, &rows, &tx_rows)?;
        shards.push(ShardEntry::new(
            bucket,
            seqids,
            file_name,
            sha256_hex(&fs::read(&sqlite_path).map_err(|e| IngestError(e.to_string()))?),
        ));
    }
    shards.sort();
    let mode = if shard_partitions == 0 {
        "per-seqid".to_string()
    } else {
        format!("partitioned-{shard_partitions}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_only_migration_rejects_downgrade() {
        let conn = Connection::open_in_memory().expect("conn");
        conn.execute_batch("PRAGMA user_version=4;")
            .expect("set user_version");
        let err = migrate_forward_schema(&conn, 3).expect_err("downgrade must fail");
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
        migrate_forward_schema(&conn, 3).expect("migrate");
        let v: i64 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .expect("schema_version row");
        assert_eq!(v, 3);
    }
}
