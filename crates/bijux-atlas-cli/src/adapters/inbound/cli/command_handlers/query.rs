// SPDX-License-Identifier: Apache-2.0

use super::super::commands::ExportFormat;
use super::super::{output, OutputMode};
use bijux_atlas_query::{
    classify_query, estimate_work_units, explain_query_plan, query_genes, GeneFields, GeneFilter,
    GeneQueryRequest, IntervalSemantics, QueryLimits, QuerySort, RegionFilter, StrandMode,
};
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

pub(crate) struct ExplainQueryArgs {
    pub(crate) db: PathBuf,
    pub(crate) gene_id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) name_prefix: Option<String>,
    pub(crate) biotype: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) limit: usize,
    pub(crate) allow_full_scan: bool,
}

pub(crate) fn explain_query_from_query_text(
    db: PathBuf,
    query_text: &str,
    limit: usize,
    allow_full_scan: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    let parsed = parse_query_text(query_text);
    explain_query(
        ExplainQueryArgs {
            db,
            gene_id: parsed.get("gene_id").cloned(),
            name: parsed.get("name").cloned(),
            name_prefix: parsed.get("name_prefix").cloned(),
            biotype: parsed.get("biotype").cloned(),
            region: parsed.get("region").cloned(),
            limit,
            allow_full_scan,
        },
        output_mode,
    )
}

pub(crate) fn run_query(args: ExplainQueryArgs, output_mode: OutputMode) -> Result<(), String> {
    let conn = Connection::open(args.db.clone()).map_err(|e| e.to_string())?;
    let req = build_query_request(args)?;
    let query_class = classify_query(&req);
    let cost_units = estimate_work_units(&req);
    let resp = query_genes(&conn, &req, &QueryLimits::default(), b"atlas-cli")
        .map_err(|e| e.to_string())?;
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas query run",
            "query_class": format!("{query_class:?}"),
            "estimated_cost_units": cost_units,
            "runtime_query_evidence": {
                "query_class": format!("{query_class:?}"),
                "estimated_cost_units": cost_units,
                "cursor_secret_owner": "atlas-cli",
                "engine": "sqlite",
                "coordinate_system": "1-based-closed"
            },
            "rows": resp.rows,
            "next_cursor": resp.next_cursor,
        }),
    )?;
    Ok(())
}

pub(crate) fn explain_query(args: ExplainQueryArgs, output_mode: OutputMode) -> Result<(), String> {
    let conn = Connection::open(args.db.clone()).map_err(|e| e.to_string())?;
    let req = build_query_request(args)?;
    let query_class = classify_query(&req);
    let cost_units = estimate_work_units(&req);
    let lines = explain_query_plan(&conn, &req, &QueryLimits::default(), b"atlas-cli")
        .map_err(|e| e.to_string())?;
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas query explain",
            "query_class": format!("{query_class:?}"),
            "estimated_cost_units": cost_units,
            "runtime_query_evidence": {
                "query_class": format!("{query_class:?}"),
                "estimated_cost_units": cost_units,
                "cursor_secret_owner": "atlas-cli",
                "engine": "sqlite",
                "coordinate_system": "1-based-closed"
            },
            "plan": lines
        }),
    )?;
    Ok(())
}

pub(crate) fn export_query_rows(
    args: ExplainQueryArgs,
    out: PathBuf,
    format: ExportFormat,
    output_mode: OutputMode,
) -> Result<(), String> {
    let conn = Connection::open(args.db.clone()).map_err(|e| e.to_string())?;
    let req = build_query_request(args)?;
    let resp = query_genes(&conn, &req, &QueryLimits::default(), b"atlas-cli")
        .map_err(|e| e.to_string())?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    match format {
        ExportFormat::Json => {
            fs::write(
                &out,
                serde_json::to_vec_pretty(&resp.rows).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
        ExportFormat::Jsonl => {
            let mut buf = String::new();
            for row in &resp.rows {
                buf.push_str(&serde_json::to_string(row).map_err(|e| e.to_string())?);
                buf.push('\n');
            }
            fs::write(&out, buf).map_err(|e| e.to_string())?;
        }
        ExportFormat::Csv => {
            let mut writer = csv::Writer::from_path(&out).map_err(|e| e.to_string())?;
            writer
                .write_record([
                    "gene_id",
                    "name",
                    "seqid",
                    "start",
                    "end",
                    "biotype",
                    "transcript_count",
                    "sequence_length",
                ])
                .map_err(|e| e.to_string())?;
            for row in &resp.rows {
                writer
                    .write_record([
                        row.gene_id.to_string(),
                        row.name.clone().unwrap_or_default(),
                        row.seqid.clone().unwrap_or_default(),
                        row.start.map(|x| x.to_string()).unwrap_or_default(),
                        row.end.map(|x| x.to_string()).unwrap_or_default(),
                        row.biotype.clone().unwrap_or_default(),
                        row.transcript_count
                            .map(|x| x.to_string())
                            .unwrap_or_default(),
                        row.sequence_length
                            .map(|x| x.to_string())
                            .unwrap_or_default(),
                    ])
                    .map_err(|e| e.to_string())?;
            }
            writer.flush().map_err(|e| e.to_string())?;
        }
    }
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas export query",
            "status":"ok",
            "out": out,
            "format": match format {
                ExportFormat::Json => "json",
                ExportFormat::Jsonl => "jsonl",
                ExportFormat::Csv => "csv",
            },
            "rows": resp.rows.len()
        }),
    )?;
    Ok(())
}

fn parse_query_text(query_text: &str) -> std::collections::HashMap<String, String> {
    query_text
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k.is_empty() {
                return None;
            }
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

fn build_query_request(args: ExplainQueryArgs) -> Result<GeneQueryRequest, String> {
    let region_filter = if let Some(raw) = args.region {
        let (seqid, span) = raw
            .split_once(':')
            .ok_or_else(|| "region must be seqid:start-end".to_string())?;
        let (start, end) = span
            .split_once('-')
            .ok_or_else(|| "region must be seqid:start-end".to_string())?;
        Some(RegionFilter {
            seqid: seqid.to_string(),
            start: start.parse::<u64>().map_err(|e| e.to_string())?,
            end: end.parse::<u64>().map_err(|e| e.to_string())?,
        })
    } else {
        None
    };
    Ok(GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: args.gene_id,
            name: args.name,
            name_prefix: args.name_prefix,
            biotype: args.biotype,
            region: region_filter,
            sort: QuerySort::Auto,
            interval: IntervalSemantics::Overlap,
            strand: StrandMode::Any,
        },
        limit: args.limit,
        cursor: None,
        dataset_key: None,
        allow_full_scan: args.allow_full_scan,
    })
}
