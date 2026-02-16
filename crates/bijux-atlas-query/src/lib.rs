#![forbid(unsafe_code)]

mod cursor;
mod filters;
mod limits;
mod planner;
mod sql;

use cursor::{decode_cursor, encode_cursor, CursorPayload, OrderMode};
use planner::validate_request;
use rusqlite::{params_from_iter, types::Value, Connection};
use sql::{
    assert_index_usage, build_sql, normalized_query_hash, order_mode_for, parse_row_from_sql,
};

pub const CRATE_NAME: &str = "bijux-atlas-query";

pub use cursor::{CursorError, CursorErrorCode};
pub use filters::{
    compile_field_projection, escape_like_prefix, GeneFields, GeneFilter, GeneRow, RegionFilter,
};
pub use limits::QueryLimits as QueryLimitsExport;
pub use planner::{classify_query, estimate_work_units, QueryClass};
pub use sql::explain_query_plan as explain_query_plan_internal;

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
            decode_cursor(token, cursor_secret, &query_hash, order_mode)
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
        .prepare(&sql)
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
        let last = rows
            .last()
            .ok_or_else(|| QueryError::new(QueryErrorCode::Sql, "pagination invariant violated"))?;
        let payload = match order_mode {
            OrderMode::Region => CursorPayload {
                order: "region".to_string(),
                last_seqid: last.seqid.clone(),
                last_start: last.start,
                last_gene_id: last.gene_id.clone(),
                query_hash,
            },
            OrderMode::GeneId => CursorPayload {
                order: "gene_id".to_string(),
                last_seqid: None,
                last_start: None,
                last_gene_id: last.gene_id.clone(),
                query_hash,
            },
        };
        Some(
            encode_cursor(&payload, cursor_secret)
                .map_err(|e| QueryError::new(QueryErrorCode::Cursor, e.to_string()))?,
        )
    } else {
        None
    };

    Ok(GeneQueryResponse { rows, next_cursor })
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
            decode_cursor(token, cursor_secret, &query_hash, order_mode)
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

pub use filters::{GeneQueryRequest, GeneQueryResponse};
pub use limits::QueryLimits;

#[cfg(test)]
mod query_tests;
