// SPDX-License-Identifier: Apache-2.0

use rusqlite::{params_from_iter, types::Value, Connection};

use crate::cursor::{
    decode_cursor as decode_cursor_inner, encode_cursor as encode_cursor_inner,
    CursorPayload as CursorPayloadInner, OrderMode as OrderModeInner,
};
use crate::db::{assert_index_usage, build_sql, order_mode_for, parse_row_from_sql};
use crate::filters::{self, GeneQueryRequest, GeneQueryResponse};
use crate::planner::QueryPlan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    Cursor(String),
    Sql(String),
    Policy(String),
    Validation(String),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cursor(msg) | Self::Sql(msg) | Self::Policy(msg) | Self::Validation(msg) => {
                f.write_str(msg)
            }
        }
    }
}

impl std::error::Error for ExecError {}

pub fn execute_gene_query(
    conn: &Connection,
    req: &GeneQueryRequest,
    plan: &QueryPlan,
    query_hash: &str,
    cursor_secret: &[u8],
) -> Result<GeneQueryResponse, ExecError> {
    let order_mode = order_mode_for(req);

    let decoded_cursor = if let Some(token) = &req.cursor {
        Some(
            decode_cursor_inner(
                token,
                cursor_secret,
                query_hash,
                order_mode,
                req.dataset_key.as_deref(),
            )
            .map_err(|e| ExecError::Cursor(e.to_string()))?,
        )
    } else {
        None
    };

    let (sql, mut params) =
        build_sql(req, order_mode, decoded_cursor.as_ref()).map_err(ExecError::Sql)?;
    params.push(Value::Integer((req.limit as i64) + 1));
    assert_index_usage(conn, &sql, &params, req.allow_full_scan).map_err(ExecError::Policy)?;

    let mut stmt = conn
        .prepare_cached(&sql)
        .map_err(|e| ExecError::Sql(e.to_string()))?;
    let mapped = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            parse_row_from_sql(row, &req.fields)
        })
        .map_err(|e| ExecError::Sql(e.to_string()))?;

    let mut rows: Vec<filters::GeneRow> = mapped
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ExecError::Sql(e.to_string()))?;

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
            .ok_or_else(|| ExecError::Sql("pagination invariant violated".to_string()))?;
        let payload = match order_mode {
            OrderModeInner::Region => CursorPayloadInner {
                cursor_version: "v1".to_string(),
                dataset_id: req.dataset_key.clone(),
                sort_key: Some("region".to_string()),
                last_seen: Some(crate::cursor::CursorLastSeen {
                    gene_id: last.gene_id.clone(),
                    seqid: last.seqid.clone(),
                    start: last.start,
                }),
                order: "region".to_string(),
                last_seqid: last.seqid.clone(),
                last_start: last.start,
                last_gene_id: last.gene_id.clone(),
                query_hash: query_hash.to_string(),
                depth: next_depth,
            },
            OrderModeInner::GeneId => CursorPayloadInner {
                cursor_version: "v1".to_string(),
                dataset_id: req.dataset_key.clone(),
                sort_key: Some("gene_id".to_string()),
                last_seen: Some(crate::cursor::CursorLastSeen {
                    gene_id: last.gene_id.clone(),
                    seqid: None,
                    start: None,
                }),
                order: "gene_id".to_string(),
                last_seqid: None,
                last_start: None,
                last_gene_id: last.gene_id.clone(),
                query_hash: query_hash.to_string(),
                depth: next_depth,
            },
        };
        Some(
            encode_cursor_inner(&payload, cursor_secret)
                .map_err(|e| ExecError::Cursor(e.to_string()))?,
        )
    } else {
        None
    };

    let _ = plan;
    Ok(GeneQueryResponse { rows, next_cursor })
}
