use crate::errors::ApiError;
use crate::params::{IncludeField, ListGenesParams};
use bijux_atlas_query::{GeneQueryResponse, GeneRow};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

pub trait QueryAdapter {
    fn list_genes(&self, params: &ListGenesParams) -> Result<GeneQueryResponse, ApiError>;
}

pub fn list_genes_v1<A: QueryAdapter>(
    adapter: &A,
    params: &ListGenesParams,
) -> Result<Value, ApiError> {
    let page = adapter.list_genes(params)?;
    let requested = params
        .include
        .as_ref()
        .map(|v| v.iter().cloned().collect::<BTreeSet<_>>());
    let rows = page
        .rows
        .iter()
        .map(|row| shape_row(row, requested.as_ref()))
        .collect::<Vec<_>>();
    Ok(json!({
        "rows": rows,
        "next_cursor": page.next_cursor
    }))
}

fn include_field(requested: Option<&BTreeSet<IncludeField>>, field: IncludeField) -> bool {
    requested.is_some_and(|set| set.contains(&field))
}

fn shape_row(row: &GeneRow, requested: Option<&BTreeSet<IncludeField>>) -> Value {
    // Policy: omitted when field is not requested; null when requested but value is absent.
    let mut map = Map::new();
    map.insert("gene_id".to_string(), Value::String(row.gene_id.clone()));
    map.insert(
        "name".to_string(),
        row.name
            .as_ref()
            .map_or(Value::Null, |x| Value::String(x.clone())),
    );
    if include_field(requested, IncludeField::Coords) {
        map.insert(
            "seqid".to_string(),
            row.seqid
                .as_ref()
                .map_or(Value::Null, |x| Value::String(x.clone())),
        );
        map.insert(
            "start".to_string(),
            row.start.map_or(Value::Null, |x| Value::Number(x.into())),
        );
        map.insert(
            "end".to_string(),
            row.end.map_or(Value::Null, |x| Value::Number(x.into())),
        );
    }
    if include_field(requested, IncludeField::Biotype) {
        map.insert(
            "biotype".to_string(),
            row.biotype
                .as_ref()
                .map_or(Value::Null, |x| Value::String(x.clone())),
        );
    }
    if include_field(requested, IncludeField::Counts) {
        map.insert(
            "transcript_count".to_string(),
            row.transcript_count
                .map_or(Value::Null, |x| Value::Number(x.into())),
        );
    }
    if include_field(requested, IncludeField::Length) {
        map.insert(
            "sequence_length".to_string(),
            row.sequence_length
                .map_or(Value::Null, |x| Value::Number(x.into())),
        );
    }
    Value::Object(map)
}
