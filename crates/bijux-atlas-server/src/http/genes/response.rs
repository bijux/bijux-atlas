use crate::*;
use crate::http::handlers;
use serde_json::json;

pub(super) fn build_success_payload(
    dataset: &DatasetId,
    req: &GeneQueryRequest,
    class: QueryClass,
    resp: bijux_atlas_query::GeneQueryResponse,
    explain_mode: bool,
    provenance: serde_json::Value,
) -> serde_json::Value {
    let mut warnings = Vec::new();
    if req.fields.sequence_length {
        warnings.push(json!({
            "code": "expensive_include_length",
            "message": "length include may increase response cost"
        }));
    }
    let mut payload = handlers::json_envelope(
        Some(json!(dataset)),
        Some(json!({ "next_cursor": resp.next_cursor.clone() })),
        json!({
            "provenance": provenance,
            "class": format!("{class:?}").to_lowercase(),
            "rows": resp.rows
        }),
        resp.next_cursor.map(|c| json!({ "next_cursor": c })),
        Some(warnings),
    );
    if explain_mode {
        let name_policy = bijux_atlas_model::GeneNamePolicy::default();
        let biotype_policy = bijux_atlas_model::BiotypePolicy::default();
        payload["data"]["explain"] = json!({
            "gene_identifier_policy": "gff3_id_first",
            "gene_name_attribute_priority": name_policy.attribute_keys,
            "biotype_attribute_priority": biotype_policy.attribute_keys,
            "biotype_unknown_value": biotype_policy.unknown_value
        });
    }
    payload
}
