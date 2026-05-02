// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::handlers;
use crate::domain::dataset::DatasetId;
use crate::domain::query::{BiotypePolicy, GeneNamePolicy, GeneQueryRequest, QueryClass};
use serde_json::{json, Value};

pub(super) fn build_success_payload(
    dataset: &DatasetId,
    req: &GeneQueryRequest,
    class: QueryClass,
    resp: bijux_atlas::domain::query::GeneQueryResponse,
    explain_mode: bool,
    provenance: serde_json::Value,
) -> serde_json::Value {
    let next_cursor = resp.next_cursor;
    let has_more = next_cursor.is_some();
    let mut warnings = Vec::new();
    if req.fields.sequence_length {
        warnings.push(json!({
            "code": "expensive_include_length",
            "message": "length include may increase response cost"
        }));
    }
    let page = next_cursor
        .as_ref()
        .map(|c| json!({ "next_cursor": c }))
        .unwrap_or_else(|| json!({ "next_cursor": null }));
    let mut payload = handlers::json_envelope(
        Some(json!(dataset)),
        Some(page),
        json!({
            "provenance": provenance,
            "class": format!("{class:?}").to_lowercase(),
            "rows": resp.rows
        }),
        next_cursor.clone().map(|c| json!({ "next_cursor": c })),
        Some(warnings),
    );
    if explain_mode {
        let name_policy = GeneNamePolicy::default();
        let biotype_policy = BiotypePolicy::default();
        let first_row = resp.rows.first();
        let last_row = resp.rows.last();
        payload["data"]["explain"] = json!({
            "gene_identifier_policy": "gff3_id_first",
            "gene_name_attribute_priority": name_policy.attribute_keys,
            "biotype_attribute_priority": biotype_policy.attribute_keys,
            "biotype_unknown_value": biotype_policy.unknown_value,
            "dataset_target": dataset.canonical_string(),
            "effective_filters": {
                "gene_id": req.filter.gene_id.as_deref(),
                "name": req.filter.name.as_deref(),
                "name_prefix": req.filter.name_prefix.as_deref(),
                "biotype": req.filter.biotype.as_deref(),
                "region": req.filter.region.as_ref(),
                "sort": req.filter.sort,
                "interval": req.filter.interval,
                "strand": req.filter.strand
            },
            "result_bounds": {
                "limit": req.limit,
                "returned_rows": resp.rows.len(),
                "has_more": has_more,
                "first_row": explain_row_bounds(first_row),
                "last_row": explain_row_bounds(last_row)
            }
        });
    }
    payload
}

fn explain_row_bounds(row: Option<&bijux_atlas::domain::query::GeneRow>) -> Value {
    let Some(row) = row else {
        return Value::Null;
    };
    json!({
        "gene_id": row.gene_id,
        "seqid": row.seqid,
        "start": row.start,
        "end": row.end
    })
}
