#![forbid(unsafe_code)]

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use rusqlite::{params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CRATE_NAME: &str = "bijux-atlas-query";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneFields {
    pub gene_id: bool,
    pub name: bool,
    pub coords: bool,
    pub biotype: bool,
    pub transcript_count: bool,
    pub sequence_length: bool,
}

impl Default for GeneFields {
    fn default() -> Self {
        Self {
            gene_id: true,
            name: true,
            coords: true,
            biotype: true,
            transcript_count: true,
            sequence_length: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionFilter {
    pub seqid: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GeneFilter {
    pub gene_id: Option<String>,
    pub name: Option<String>,
    pub name_prefix: Option<String>,
    pub biotype: Option<String>,
    pub region: Option<RegionFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryLimits {
    pub max_limit: usize,
    pub max_region_span: u64,
    pub max_prefix_len: usize,
    pub max_work_units: u64,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_limit: 500,
            max_region_span: 5_000_000,
            max_prefix_len: 64,
            max_work_units: 2_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneQueryRequest {
    pub fields: GeneFields,
    pub filter: GeneFilter,
    pub limit: usize,
    pub cursor: Option<String>,
    pub allow_full_scan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneRow {
    pub gene_id: String,
    pub name: Option<String>,
    pub seqid: Option<String>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub biotype: Option<String>,
    pub transcript_count: Option<u64>,
    pub sequence_length: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneQueryResponse {
    pub rows: Vec<GeneRow>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryClass {
    Cheap,
    Medium,
    Heavy,
}

#[derive(Debug)]
pub struct QueryError(pub String);

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for QueryError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CursorPayload {
    order: String,
    last_seqid: Option<String>,
    last_start: Option<u64>,
    last_gene_id: String,
    query_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderMode {
    Region,
    GeneId,
}

pub fn query_genes(
    conn: &Connection,
    req: &GeneQueryRequest,
    limits: &QueryLimits,
    cursor_secret: &[u8],
) -> Result<GeneQueryResponse, QueryError> {
    validate_request(req, limits)?;

    let order_mode = if req.filter.region.is_some() {
        OrderMode::Region
    } else {
        OrderMode::GeneId
    };
    let query_hash = request_hash(req)?;

    let decoded_cursor = if let Some(token) = &req.cursor {
        Some(decode_cursor(
            token,
            cursor_secret,
            &query_hash,
            order_mode,
        )?)
    } else {
        None
    };

    let (sql, mut params) = build_sql(req, order_mode, decoded_cursor.as_ref())?;
    params.push(Value::Integer((req.limit as i64) + 1));
    assert_index_usage(conn, &sql, &params, req.allow_full_scan)?;

    let mut stmt = conn.prepare(&sql).map_err(|e| QueryError(e.to_string()))?;
    let mapped = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            Ok(GeneRow {
                gene_id: row.get::<_, String>(0)?,
                name: row.get::<_, Option<String>>(1)?,
                seqid: row.get::<_, Option<String>>(2)?,
                start: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                end: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                biotype: row.get::<_, Option<String>>(5)?,
                transcript_count: row.get::<_, Option<i64>>(6)?.map(|v| v as u64),
                sequence_length: row.get::<_, Option<i64>>(7)?.map(|v| v as u64),
            })
        })
        .map_err(|e| QueryError(e.to_string()))?;

    let mut rows: Vec<GeneRow> = mapped
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| QueryError(e.to_string()))?;

    let has_more = rows.len() > req.limit;
    if has_more {
        rows.truncate(req.limit);
    }

    let next_cursor = if has_more {
        let last = rows
            .last()
            .ok_or_else(|| QueryError("pagination invariant violated".to_string()))?;
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
        Some(encode_cursor(&payload, cursor_secret)?)
    } else {
        None
    };

    Ok(GeneQueryResponse { rows, next_cursor })
}

fn validate_request(req: &GeneQueryRequest, limits: &QueryLimits) -> Result<(), QueryError> {
    if req.limit == 0 || req.limit > limits.max_limit {
        return Err(QueryError(format!(
            "limit must be between 1 and {}",
            limits.max_limit
        )));
    }
    if let Some(prefix) = &req.filter.name_prefix {
        if prefix.len() > limits.max_prefix_len {
            return Err(QueryError(format!(
                "name_prefix length exceeds {}",
                limits.max_prefix_len
            )));
        }
    }
    if let Some(region) = &req.filter.region {
        if region.start == 0 || region.end < region.start {
            return Err(QueryError("invalid region span".to_string()));
        }
        let span = region.end - region.start + 1;
        if span > limits.max_region_span {
            return Err(QueryError(format!(
                "region span exceeds {}",
                limits.max_region_span
            )));
        }
    }
    let has_any_filter = req.filter.gene_id.is_some()
        || req.filter.name.is_some()
        || req.filter.name_prefix.is_some()
        || req.filter.biotype.is_some()
        || req.filter.region.is_some();
    if !has_any_filter && !req.allow_full_scan {
        return Err(QueryError(
            "full table scan is forbidden without explicit allow_full_scan=true".to_string(),
        ));
    }

    let work = estimate_work_units(req);
    if work > limits.max_work_units {
        return Err(QueryError(format!(
            "estimated query cost {} exceeds max_work_units {}",
            work, limits.max_work_units
        )));
    }
    Ok(())
}

#[must_use]
pub fn classify_query(req: &GeneQueryRequest) -> QueryClass {
    if req.filter.gene_id.is_some() {
        QueryClass::Cheap
    } else if req.filter.region.is_some() || req.filter.name_prefix.is_some() {
        QueryClass::Heavy
    } else {
        QueryClass::Medium
    }
}

#[must_use]
pub fn estimate_work_units(req: &GeneQueryRequest) -> u64 {
    let base = match classify_query(req) {
        QueryClass::Cheap => 20_u64,
        QueryClass::Medium => 200_u64,
        QueryClass::Heavy => 1200_u64,
    };
    let region_cost = req
        .filter
        .region
        .as_ref()
        .map_or(0_u64, |r| (r.end.saturating_sub(r.start) + 1) / 10_000);
    base + (req.limit as u64) + region_cost
}

fn request_hash(req: &GeneQueryRequest) -> Result<String, QueryError> {
    let mut no_cursor = req.clone();
    no_cursor.cursor = None;
    let bytes = serde_json::to_vec(&no_cursor).map_err(|e| QueryError(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn encode_cursor(payload: &CursorPayload, secret: &[u8]) -> Result<String, QueryError> {
    let payload_bytes = serde_json::to_vec(payload).map_err(|e| QueryError(e.to_string()))?;
    let payload_part = URL_SAFE_NO_PAD.encode(payload_bytes);
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| QueryError(e.to_string()))?;
    mac.update(payload_part.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_part = URL_SAFE_NO_PAD.encode(sig);
    Ok(format!("{}.{}", payload_part, sig_part))
}

fn decode_cursor(
    token: &str,
    secret: &[u8],
    expected_hash: &str,
    mode: OrderMode,
) -> Result<CursorPayload, QueryError> {
    let (payload_part, sig_part) = token
        .split_once('.')
        .ok_or_else(|| QueryError("invalid cursor format".to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| QueryError(e.to_string()))?;
    mac.update(payload_part.as_bytes());
    let expected = URL_SAFE_NO_PAD
        .decode(sig_part)
        .map_err(|e| QueryError(e.to_string()))?;
    mac.verify_slice(&expected)
        .map_err(|_| QueryError("cursor signature mismatch".to_string()))?;

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_part)
        .map_err(|e| QueryError(e.to_string()))?;
    let payload: CursorPayload =
        serde_json::from_slice(&payload_bytes).map_err(|e| QueryError(e.to_string()))?;

    if payload.query_hash != expected_hash {
        return Err(QueryError("cursor query hash mismatch".to_string()));
    }
    match mode {
        OrderMode::Region if payload.order != "region" => {
            return Err(QueryError(
                "cursor order mismatch for region query".to_string(),
            ));
        }
        OrderMode::GeneId if payload.order != "gene_id" => {
            return Err(QueryError(
                "cursor order mismatch for gene_id query".to_string(),
            ));
        }
        _ => {}
    }
    Ok(payload)
}

fn build_sql(
    req: &GeneQueryRequest,
    order_mode: OrderMode,
    cursor: Option<&CursorPayload>,
) -> Result<(String, Vec<Value>), QueryError> {
    let select = compile_field_projection(&req.fields);

    let mut sql = format!("SELECT {} FROM gene_summary g", select.join(", "));
    let mut where_parts: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();

    if let Some(region) = &req.filter.region {
        sql.push_str(" JOIN gene_summary_rtree r ON r.gene_rowid = g.id");
        where_parts.push("g.seqid = ?".to_string());
        params.push(Value::Text(region.seqid.clone()));
        where_parts.push("r.start <= ?".to_string());
        params.push(Value::Real(region.end as f64));
        where_parts.push("r.end >= ?".to_string());
        params.push(Value::Real(region.start as f64));
    }
    if let Some(gene_id) = &req.filter.gene_id {
        where_parts.push("g.gene_id = ?".to_string());
        params.push(Value::Text(gene_id.clone()));
    }
    if let Some(name) = &req.filter.name {
        where_parts.push("g.name = ?".to_string());
        params.push(Value::Text(name.clone()));
    }
    if let Some(prefix) = &req.filter.name_prefix {
        where_parts.push("g.name LIKE ? ESCAPE '!'".to_string());
        params.push(Value::Text(format!("{}%", escape_like_prefix(prefix))));
    }
    if let Some(biotype) = &req.filter.biotype {
        where_parts.push("g.biotype = ?".to_string());
        params.push(Value::Text(biotype.clone()));
    }

    if let Some(c) = cursor {
        match order_mode {
            OrderMode::GeneId => {
                where_parts.push("g.gene_id > ?".to_string());
                params.push(Value::Text(c.last_gene_id.clone()));
            }
            OrderMode::Region => {
                let seqid = c
                    .last_seqid
                    .clone()
                    .ok_or_else(|| QueryError("region cursor missing seqid".to_string()))?;
                let start = c
                    .last_start
                    .ok_or_else(|| QueryError("region cursor missing start".to_string()))?;
                where_parts.push(
                    "(g.seqid > ? OR (g.seqid = ? AND g.start > ?) OR (g.seqid = ? AND g.start = ? AND g.gene_id > ?))"
                        .to_string(),
                );
                params.push(Value::Text(seqid.clone()));
                params.push(Value::Text(seqid.clone()));
                params.push(Value::Integer(start as i64));
                params.push(Value::Text(seqid));
                params.push(Value::Integer(start as i64));
                params.push(Value::Text(c.last_gene_id.clone()));
            }
        }
    }

    if !where_parts.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_parts.join(" AND "));
    }

    match order_mode {
        OrderMode::Region => sql.push_str(" ORDER BY g.seqid ASC, g.start ASC, g.gene_id ASC"),
        OrderMode::GeneId => sql.push_str(" ORDER BY g.gene_id ASC"),
    }
    sql.push_str(" LIMIT ?");

    Ok((sql, params))
}

#[must_use]
pub fn compile_field_projection(fields: &GeneFields) -> Vec<String> {
    let mut select = vec!["g.gene_id".to_string()];
    select.push(if fields.name {
        "g.name".to_string()
    } else {
        "NULL AS name".to_string()
    });
    select.push(if fields.coords {
        "g.seqid".to_string()
    } else {
        "NULL AS seqid".to_string()
    });
    select.push(if fields.coords {
        "g.start".to_string()
    } else {
        "NULL AS start".to_string()
    });
    select.push(if fields.coords {
        "g.end".to_string()
    } else {
        "NULL AS end".to_string()
    });
    select.push(if fields.biotype {
        "g.biotype".to_string()
    } else {
        "NULL AS biotype".to_string()
    });
    select.push(if fields.transcript_count {
        "g.transcript_count".to_string()
    } else {
        "NULL AS transcript_count".to_string()
    });
    select.push(if fields.sequence_length {
        "g.sequence_length".to_string()
    } else {
        "NULL AS sequence_length".to_string()
    });
    select
}

fn escape_like_prefix(prefix: &str) -> String {
    let mut out = String::with_capacity(prefix.len());
    for c in prefix.chars() {
        match c {
            '!' | '%' | '_' => {
                out.push('!');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

pub fn explain_query_plan(
    conn: &Connection,
    req: &GeneQueryRequest,
    limits: &QueryLimits,
    cursor_secret: &[u8],
) -> Result<Vec<String>, QueryError> {
    validate_request(req, limits)?;
    let order_mode = if req.filter.region.is_some() {
        OrderMode::Region
    } else {
        OrderMode::GeneId
    };
    let query_hash = request_hash(req)?;
    let decoded_cursor = if let Some(token) = &req.cursor {
        Some(decode_cursor(
            token,
            cursor_secret,
            &query_hash,
            order_mode,
        )?)
    } else {
        None
    };
    let (sql, mut params) = build_sql(req, order_mode, decoded_cursor.as_ref())?;
    params.push(Value::Integer((req.limit as i64) + 1));
    let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let mut stmt = conn
        .prepare(&explain_sql)
        .map_err(|e| QueryError(e.to_string()))?;
    let mut lines = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            row.get::<_, String>(3)
        })
        .map_err(|e| QueryError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| QueryError(e.to_string()))?;
    lines.sort();
    Ok(lines)
}

fn assert_index_usage(
    conn: &Connection,
    sql: &str,
    params: &[Value],
    allow_full_scan: bool,
) -> Result<(), QueryError> {
    let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let mut stmt = conn
        .prepare(&explain_sql)
        .map_err(|e| QueryError(e.to_string()))?;
    let plan = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            row.get::<_, String>(3)
        })
        .map_err(|e| QueryError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| QueryError(e.to_string()))?
        .join("\n");

    if !allow_full_scan && plan.contains("SCAN gene_summary") {
        return Err(QueryError(
            "query plan indicates full table scan; forbidden by policy".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod query_tests;
