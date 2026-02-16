use crate::extract::GeneRecord;
use crate::IngestError;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

pub const SQLITE_SCHEMA_VERSION: i64 = 1;

pub fn write_sqlite(path: &Path, genes: &[GeneRecord]) -> Result<(), IngestError> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| IngestError(e.to_string()))?;
    }
    let mut conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    conn.execute_batch(
        "
        PRAGMA journal_mode=DELETE;
        PRAGMA synchronous=FULL;
        PRAGMA temp_store=MEMORY;
        PRAGMA cache_size=-32000;
        PRAGMA page_size=4096;
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
    conn.execute_batch(&format!("PRAGMA user_version={};", SQLITE_SCHEMA_VERSION))
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

pub fn explain_plan_for_region_query(path: &Path) -> Result<Vec<String>, IngestError> {
    let conn = Connection::open(path).map_err(|e| IngestError(e.to_string()))?;
    let mut stmt = conn
        .prepare("EXPLAIN QUERY PLAN SELECT gene_id FROM gene_summary WHERE seqid=?1 AND start<=?2 AND end>=?3 ORDER BY seqid,start,gene_id LIMIT 10")
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
