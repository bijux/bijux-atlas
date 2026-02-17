#![forbid(unsafe_code)]

mod cost;
mod cursor;
mod filters;
mod limits;
mod planner;
mod row_decode;
mod sql;

use cursor::{
    decode_cursor as decode_cursor_inner, encode_cursor as encode_cursor_inner,
    CursorPayload as CursorPayloadInner, OrderMode as OrderModeInner,
};
use planner::validate_request;
use rusqlite::{params_from_iter, types::Value, Connection};
use sql::{
    assert_index_usage, build_sql, normalized_query_hash, order_mode_for, parse_row_from_sql,
    query_gene_id_name_json_minimal,
};

pub const CRATE_NAME: &str = "bijux-atlas-query";

pub use cost::estimate_prefix_match_cost;
pub use cursor::{
    decode_cursor, encode_cursor, CursorError, CursorErrorCode, CursorPayload, OrderMode,
};
pub use filters::{
    compile_field_projection, escape_like_prefix, GeneFields, GeneFilter, GeneRow, RegionFilter,
    TranscriptFilter, TranscriptQueryRequest, TranscriptQueryResponse, TranscriptRow,
};
pub use limits::QueryLimits as QueryLimitsExport;
pub use planner::{classify_query, estimate_work_units, select_shards_for_request, QueryClass};
pub use row_decode::RawGeneRow;
pub use sql::explain_query_plan as explain_query_plan_internal;
pub use sql::prepared_sql_for_class as prepared_sql_for_class_export;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryErrorCode {
    Validation,
    Cursor,
    Sql,
    Policy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError {
    pub code: QueryErrorCode,
    pub message: String,
}

impl QueryError {
    #[must_use]
    pub fn new(code: QueryErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
impl std::error::Error for QueryError {}

pub fn query_genes(
    conn: &Connection,
    req: &GeneQueryRequest,
    limits: &QueryLimits,
    cursor_secret: &[u8],
) -> Result<GeneQueryResponse, QueryError> {
    validate_request(req, limits).map_err(|e| QueryError::new(QueryErrorCode::Validation, e))?;
    reject_impossible_filter_fast(req, limits, conn)?;

    let order_mode = order_mode_for(req);
    let query_hash =
        normalized_query_hash(req).map_err(|e| QueryError::new(QueryErrorCode::Validation, e))?;

    let decoded_cursor = if let Some(token) = &req.cursor {
        Some(
            decode_cursor_inner(token, cursor_secret, &query_hash, order_mode)
                .map_err(|e| QueryError::new(QueryErrorCode::Cursor, e.to_string()))?,
        )
    } else {
        None
    };

    let (sql, mut params) = build_sql(req, order_mode, decoded_cursor.as_ref())
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e))?;
    params.push(Value::Integer((req.limit as i64) + 1));
    assert_index_usage(conn, &sql, &params, req.allow_full_scan)
        .map_err(|e| QueryError::new(QueryErrorCode::Policy, e))?;

    let mut stmt = conn
        .prepare_cached(&sql)
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let mapped = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            parse_row_from_sql(row, &req.fields)
        })
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;

    let mut rows: Vec<filters::GeneRow> = mapped
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;

    let has_more = rows.len() > req.limit;
    if has_more {
        rows.truncate(req.limit);
    }

    let next_cursor = if has_more {
        let next_depth = decoded_cursor
            .as_ref()
            .map_or(1_u32, |c| c.depth.saturating_add(1));
        let last = rows
            .last()
            .ok_or_else(|| QueryError::new(QueryErrorCode::Sql, "pagination invariant violated"))?;
        let payload = match order_mode {
            OrderModeInner::Region => CursorPayloadInner {
                order: "region".to_string(),
                last_seqid: last.seqid.clone(),
                last_start: last.start,
                last_gene_id: last.gene_id.clone(),
                query_hash,
                depth: next_depth,
            },
            OrderModeInner::GeneId => CursorPayloadInner {
                order: "gene_id".to_string(),
                last_seqid: None,
                last_start: None,
                last_gene_id: last.gene_id.clone(),
                query_hash,
                depth: next_depth,
            },
        };
        Some(
            encode_cursor_inner(&payload, cursor_secret)
                .map_err(|e| QueryError::new(QueryErrorCode::Cursor, e.to_string()))?,
        )
    } else {
        None
    };

    Ok(GeneQueryResponse { rows, next_cursor })
}

pub fn query_gene_by_id_fast(
    conn: &Connection,
    gene_id: &str,
    fields: &GeneFields,
) -> Result<Option<filters::GeneRow>, QueryError> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT g.gene_id, g.name, g.seqid, g.start, g.end, g.biotype, g.transcript_count, g.sequence_length
             FROM gene_summary g
             WHERE g.gene_id = ?1
             LIMIT 1",
        )
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let mut rows = stmt
        .query([gene_id])
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let Some(row) = rows
        .next()
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?
    else {
        return Ok(None);
    };
    let mut parsed = parse_row_from_sql(row, fields)
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    if !fields.name {
        parsed.name = None;
    }
    if !fields.coords {
        parsed.seqid = None;
        parsed.start = None;
        parsed.end = None;
    }
    if !fields.biotype {
        parsed.biotype = None;
    }
    if !fields.transcript_count {
        parsed.transcript_count = None;
    }
    if !fields.sequence_length {
        parsed.sequence_length = None;
    }
    Ok(Some(parsed))
}

pub fn query_gene_id_name_json_minimal_fast(
    conn: &Connection,
    gene_id: &str,
) -> Result<Option<Vec<u8>>, QueryError> {
    query_gene_id_name_json_minimal(conn, gene_id)
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e))
}

fn reject_impossible_filter_fast(
    req: &GeneQueryRequest,
    limits: &QueryLimits,
    conn: &Connection,
) -> Result<(), QueryError> {
    if let Some(biotype) = &req.filter.biotype {
        let count: i64 = conn
            .query_row(
                "SELECT COALESCE((SELECT gene_count FROM dataset_stats WHERE dimension='biotype' AND value=?1), 0)",
                [biotype],
                |r| r.get(0),
            )
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
        if count == 0 {
            return Err(QueryError::new(
                QueryErrorCode::Validation,
                "biotype does not exist in dataset",
            ));
        }
    }
    if let Some(region) = &req.filter.region {
        let seqid_count: i64 = conn
            .query_row(
                "SELECT COALESCE((SELECT gene_count FROM dataset_stats WHERE dimension='seqid' AND value=?1), 0)",
                [&region.seqid],
                |r| r.get(0),
            )
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
        if seqid_count == 0 {
            return Err(QueryError::new(
                QueryErrorCode::Validation,
                "region seqid does not exist in dataset",
            ));
        }
        let span = region.end.saturating_sub(region.start) + 1;
        let span_ratio = span as f64 / limits.max_region_span as f64;
        let estimated_rows = ((seqid_count as f64) * span_ratio).ceil() as u64;
        if estimated_rows > limits.max_region_estimated_rows {
            return Err(QueryError::new(
                QueryErrorCode::Validation,
                format!(
                    "estimated region rows {} exceeds {}",
                    estimated_rows, limits.max_region_estimated_rows
                ),
            ));
        }
    }
    Ok(())
}

pub fn explain_query_plan(
    conn: &Connection,
    req: &GeneQueryRequest,
    limits: &QueryLimits,
    cursor_secret: &[u8],
) -> Result<Vec<String>, QueryError> {
    validate_request(req, limits).map_err(|e| QueryError::new(QueryErrorCode::Validation, e))?;
    let order_mode = order_mode_for(req);
    let query_hash =
        normalized_query_hash(req).map_err(|e| QueryError::new(QueryErrorCode::Validation, e))?;
    let decoded_cursor = if let Some(token) = &req.cursor {
        Some(
            decode_cursor_inner(token, cursor_secret, &query_hash, order_mode)
                .map_err(|e| QueryError::new(QueryErrorCode::Cursor, e.to_string()))?,
        )
    } else {
        None
    };
    sql::explain_query_plan(conn, req, order_mode, decoded_cursor.as_ref())
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e))
}

pub fn query_normalization_hash(req: &GeneQueryRequest) -> Result<String, QueryError> {
    normalized_query_hash(req).map_err(|e| QueryError::new(QueryErrorCode::Validation, e))
}

pub fn query_genes_fanout(
    conns: &[&Connection],
    req: &GeneQueryRequest,
    limits: &QueryLimits,
    cursor_secret: &[u8],
) -> Result<GeneQueryResponse, QueryError> {
    if conns.is_empty() {
        return Err(QueryError::new(
            QueryErrorCode::Validation,
            "fanout requires at least one connection",
        ));
    }
    let order_mode = order_mode_for(req);
    let query_hash =
        normalized_query_hash(req).map_err(|e| QueryError::new(QueryErrorCode::Validation, e))?;
    let mut merged = Vec::new();
    for conn in conns {
        let mut req_per_shard = req.clone();
        req_per_shard.cursor = None;
        req_per_shard.limit = req.limit.saturating_add(1);
        let partial = query_genes(conn, &req_per_shard, limits, cursor_secret)?;
        merged.extend(partial.rows);
    }
    merged.sort_by(|a, b| {
        a.seqid
            .cmp(&b.seqid)
            .then(a.start.cmp(&b.start))
            .then(a.gene_id.cmp(&b.gene_id))
    });
    merged.dedup_by(|a, b| a.gene_id == b.gene_id);
    let has_more = merged.len() > req.limit;
    if has_more {
        merged.truncate(req.limit);
    }
    let next_cursor = if has_more {
        let last = merged
            .last()
            .ok_or_else(|| QueryError::new(QueryErrorCode::Sql, "pagination invariant violated"))?;
        let next_depth = req
            .cursor
            .as_ref()
            .and_then(|token| {
                decode_cursor_inner(token, cursor_secret, &query_hash, order_mode).ok()
            })
            .map_or(1_u32, |c| c.depth.saturating_add(1));
        let payload = CursorPayloadInner {
            order: "region".to_string(),
            last_seqid: last.seqid.clone(),
            last_start: last.start,
            last_gene_id: last.gene_id.clone(),
            query_hash,
            depth: next_depth,
        };
        Some(
            encode_cursor_inner(&payload, cursor_secret)
                .map_err(|e| QueryError::new(QueryErrorCode::Cursor, e.to_string()))?,
        )
    } else {
        None
    };
    Ok(GeneQueryResponse {
        rows: merged,
        next_cursor,
    })
}

pub fn query_transcripts(
    conn: &Connection,
    req: &TranscriptQueryRequest,
) -> Result<TranscriptQueryResponse, QueryError> {
    if req.limit == 0 || req.limit > 500 {
        return Err(QueryError::new(
            QueryErrorCode::Validation,
            "limit must be between 1 and 500",
        ));
    }
    let mut sql = String::from(
        "SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary",
    );
    let mut where_parts = Vec::<String>::new();
    let mut params = Vec::<Value>::new();
    if let Some(gene_id) = &req.filter.parent_gene_id {
        where_parts.push("parent_gene_id = ?".to_string());
        params.push(Value::Text(gene_id.clone()));
    }
    if let Some(biotype) = &req.filter.biotype {
        where_parts.push("biotype = ?".to_string());
        params.push(Value::Text(biotype.clone()));
    }
    if let Some(kind) = &req.filter.transcript_type {
        where_parts.push("transcript_type = ?".to_string());
        params.push(Value::Text(kind.clone()));
    }
    if let Some(region) = &req.filter.region {
        where_parts.push("seqid = ?".to_string());
        params.push(Value::Text(region.seqid.clone()));
        where_parts.push("start <= ?".to_string());
        params.push(Value::Integer(region.end as i64));
        where_parts.push("end >= ?".to_string());
        params.push(Value::Integer(region.start as i64));
    }
    if let Some(cursor) = &req.cursor {
        where_parts.push("transcript_id > ?".to_string());
        params.push(Value::Text(cursor.clone()));
    }
    if !where_parts.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_parts.join(" AND "));
    }
    sql.push_str(" ORDER BY seqid ASC, start ASC, transcript_id ASC LIMIT ?");
    params.push(Value::Integer(req.limit as i64 + 1));
    assert_index_usage(conn, &sql, &params, false)
        .map_err(|e| QueryError::new(QueryErrorCode::Policy, e))?;

    let mut stmt = conn
        .prepare_cached(&sql)
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let mapped = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            Ok(TranscriptRow {
                transcript_id: row.get::<_, String>(0)?,
                parent_gene_id: row.get::<_, String>(1)?,
                transcript_type: row.get::<_, String>(2)?,
                biotype: row.get::<_, Option<String>>(3)?,
                seqid: row.get::<_, String>(4)?,
                start: row.get::<_, i64>(5)? as u64,
                end: row.get::<_, i64>(6)? as u64,
                exon_count: row.get::<_, i64>(7)? as u64,
                total_exon_span: row.get::<_, i64>(8)? as u64,
                cds_present: row.get::<_, i64>(9)? != 0,
            })
        })
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let mut rows = mapped
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let has_more = rows.len() > req.limit;
    if has_more {
        rows.truncate(req.limit);
    }
    let next_cursor = if has_more {
        rows.last().map(|r| r.transcript_id.clone())
    } else {
        None
    };
    Ok(TranscriptQueryResponse { rows, next_cursor })
}

pub fn query_transcript_by_id(
    conn: &Connection,
    tx_id: &str,
) -> Result<Option<TranscriptRow>, QueryError> {
    let mut stmt = conn
        .prepare_cached("SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary WHERE transcript_id=?1 LIMIT 1")
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let mut rows = stmt
        .query([tx_id])
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let Some(row) = rows
        .next()
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?
    else {
        return Ok(None);
    };
    Ok(Some(TranscriptRow {
        transcript_id: row
            .get::<_, String>(0)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?,
        parent_gene_id: row
            .get::<_, String>(1)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?,
        transcript_type: row
            .get::<_, String>(2)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?,
        biotype: row
            .get::<_, Option<String>>(3)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?,
        seqid: row
            .get::<_, String>(4)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?,
        start: row
            .get::<_, i64>(5)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?
            as u64,
        end: row
            .get::<_, i64>(6)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))? as u64,
        exon_count: row
            .get::<_, i64>(7)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?
            as u64,
        total_exon_span: row
            .get::<_, i64>(8)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?
            as u64,
        cds_present: row
            .get::<_, i64>(9)
            .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?
            != 0,
    }))
}

pub fn explain_transcript_query_plan(
    conn: &Connection,
    req: &TranscriptQueryRequest,
) -> Result<Vec<String>, QueryError> {
    let mut sql = String::from("SELECT transcript_id FROM transcript_summary");
    let mut where_parts = Vec::<String>::new();
    let mut params = Vec::<Value>::new();
    if let Some(gene_id) = &req.filter.parent_gene_id {
        where_parts.push("parent_gene_id = ?".to_string());
        params.push(Value::Text(gene_id.clone()));
    }
    if let Some(biotype) = &req.filter.biotype {
        where_parts.push("biotype = ?".to_string());
        params.push(Value::Text(biotype.clone()));
    }
    if let Some(kind) = &req.filter.transcript_type {
        where_parts.push("transcript_type = ?".to_string());
        params.push(Value::Text(kind.clone()));
    }
    if let Some(region) = &req.filter.region {
        where_parts.push("seqid = ?".to_string());
        params.push(Value::Text(region.seqid.clone()));
        where_parts.push("start <= ?".to_string());
        params.push(Value::Integer(region.end as i64));
        where_parts.push("end >= ?".to_string());
        params.push(Value::Integer(region.start as i64));
    }
    if !where_parts.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_parts.join(" AND "));
    }
    sql.push_str(" ORDER BY seqid ASC, start ASC, transcript_id ASC LIMIT ?");
    params.push(Value::Integer(req.limit as i64 + 1));
    let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let mut stmt = conn
        .prepare_cached(&explain_sql)
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    let mut lines = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            row.get::<_, String>(3)
        })
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| QueryError::new(QueryErrorCode::Sql, e.to_string()))?;
    lines.sort();
    Ok(lines)
}

pub use filters::{GeneQueryRequest, GeneQueryResponse};
pub use filters::{
    TranscriptQueryRequest as TxQueryRequest, TranscriptQueryResponse as TxQueryResponse,
};
pub use limits::QueryLimits;

#[cfg(test)]
mod query_tests;
