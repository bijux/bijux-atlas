// SPDX-License-Identifier: Apache-2.0

use crate::cursor::{CursorPayload, OrderMode};
use crate::filters::{
    compile_field_projection, escape_like_prefix, normalize_name_lookup, GeneFields,
    GeneQueryRequest,
};
use crate::planner::QueryClass;
use crate::row_decode::RawGeneRow;
use rusqlite::{params_from_iter, types::Value, Connection};

pub fn build_sql(
    req: &GeneQueryRequest,
    order_mode: OrderMode,
    cursor: Option<&CursorPayload>,
) -> Result<(String, Vec<Value>), String> {
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
        where_parts.push("g.name_normalized = ?".to_string());
        params.push(Value::Text(normalize_name_lookup(name)));
    }
    if let Some(prefix) = &req.filter.name_prefix {
        where_parts.push("g.name_normalized LIKE ? ESCAPE '!'".to_string());
        params.push(Value::Text(format!(
            "{}%",
            escape_like_prefix(&normalize_name_lookup(prefix))
        )));
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
                    .ok_or_else(|| "region cursor missing seqid".to_string())?;
                let start = c
                    .last_start
                    .ok_or_else(|| "region cursor missing start".to_string())?;
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

pub fn assert_index_usage(
    conn: &Connection,
    sql: &str,
    params: &[Value],
    allow_full_scan: bool,
) -> Result<(), String> {
    let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let mut stmt = conn
        .prepare_cached(&explain_sql)
        .map_err(|e| e.to_string())?;
    let lines = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            row.get::<_, String>(3)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if !allow_full_scan {
        let has_forbidden_scan = lines.iter().any(|line| {
            let upper = line.to_ascii_uppercase();
            upper.contains("SCAN")
                && !upper.contains("USING INDEX")
                && !upper.contains("USING COVERING INDEX")
                && !upper.contains("USING INTEGER PRIMARY KEY")
                && !upper.contains("VIRTUAL TABLE INDEX")
                && !upper.contains("RTREE")
        });
        if has_forbidden_scan {
            return Err(format!(
                "query plan indicates full table scan; forbidden by policy: {}",
                lines.join(" | ")
            ));
        }
    }
    Ok(())
}

pub fn explain_query_plan(
    conn: &Connection,
    req: &GeneQueryRequest,
    order_mode: OrderMode,
    cursor: Option<&CursorPayload>,
) -> Result<Vec<String>, String> {
    let (sql, mut params) = build_sql(req, order_mode, cursor)?;
    params.push(Value::Integer((req.limit as i64) + 1));
    let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let mut stmt = conn
        .prepare_cached(&explain_sql)
        .map_err(|e| e.to_string())?;
    let mut lines = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            row.get::<_, String>(3)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    lines.sort();
    Ok(lines)
}

#[must_use]
pub fn order_mode_for(req: &GeneQueryRequest) -> OrderMode {
    if req.filter.region.is_some() {
        OrderMode::Region
    } else {
        OrderMode::GeneId
    }
}

pub fn parse_row_from_sql(
    row: &rusqlite::Row<'_>,
    fields: &GeneFields,
) -> rusqlite::Result<crate::filters::GeneRow> {
    let _ = fields;
    let raw = RawGeneRow::from_sql_row(row)?;
    Ok(crate::filters::GeneRow {
        gene_id: raw.gene_id,
        name: raw.name,
        seqid: raw.seqid,
        start: raw.start.map(|v| v as u64),
        end: raw.end.map(|v| v as u64),
        biotype: raw.biotype,
        transcript_count: raw.transcript_count.map(|v| v as u64),
        sequence_length: raw.sequence_length.map(|v| v as u64),
    })
}

pub fn query_gene_id_name_json_minimal(
    conn: &Connection,
    gene_id: &str,
) -> Result<Option<Vec<u8>>, String> {
    let mut stmt = conn
        .prepare_cached("SELECT gene_id, name FROM gene_summary WHERE gene_id = ?1 LIMIT 1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([gene_id]).map_err(|e| e.to_string())?;
    let Some(row) = rows.next().map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let gid: String = row.get(0).map_err(|e| e.to_string())?;
    let name: Option<String> = row.get(1).map_err(|e| e.to_string())?;
    let gid_json = serde_json::to_string(&gid).map_err(|e| e.to_string())?;
    let name_json = serde_json::to_string(&name).map_err(|e| e.to_string())?;
    let body = format!("{{\"gene_id\":{gid_json},\"name\":{name_json}}}");
    Ok(Some(body.into_bytes()))
}

#[must_use]
pub fn prepared_sql_for_class(class: QueryClass) -> &'static str {
    match class {
        QueryClass::Cheap => {
            "SELECT gene_id, name FROM gene_summary WHERE gene_id = ?1 LIMIT ?2"
        }
        QueryClass::Medium => {
            "SELECT gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length FROM gene_summary WHERE biotype = ?1 ORDER BY gene_id LIMIT ?2"
        }
        QueryClass::Heavy => {
            "SELECT g.gene_id, g.name, g.seqid, g.start, g.end, g.biotype, g.transcript_count, g.sequence_length FROM gene_summary g JOIN gene_summary_rtree r ON r.gene_rowid = g.id WHERE g.seqid = ?1 AND r.start <= ?2 AND r.end >= ?3 ORDER BY g.seqid, g.start, g.gene_id LIMIT ?4"
        }
    }
}
