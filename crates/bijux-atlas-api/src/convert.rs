use crate::dto::{DatasetKeyDto, GeneRowsDto, LinkCursorDto, ListGenesResponseDto, PageCursorDto};
use crate::errors::ApiError;
use crate::params::IncludeField;
use bijux_atlas_query::{GeneQueryResponse, GeneRow};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

pub fn list_genes_response_dto(
    page: GeneQueryResponse,
    dataset: DatasetKeyDto,
    requested: Option<&BTreeSet<IncludeField>>,
) -> Result<ListGenesResponseDto, ApiError> {
    let mut rows = Vec::with_capacity(page.rows.len());
    for row in &page.rows {
        rows.push(gene_row_dto(row, requested)?);
    }
    let links = page.next_cursor.as_ref().map(|cursor| LinkCursorDto {
        next_cursor: cursor.clone(),
    });

    Ok(ListGenesResponseDto {
        api_version: "v1".to_string(),
        contract_version: "v1".to_string(),
        dataset,
        page: PageCursorDto {
            next_cursor: page.next_cursor,
        },
        data: GeneRowsDto { rows },
        links,
    })
}

fn gene_row_dto(
    row: &GeneRow,
    requested: Option<&BTreeSet<IncludeField>>,
) -> Result<Value, ApiError> {
    if row.gene_id.trim().is_empty() {
        return Err(ApiError::validation_failed(json!([
            {"field":"gene_id", "reason":"must be non-empty"}
        ])));
    }

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
    Ok(Value::Object(map))
}

fn include_field(requested: Option<&BTreeSet<IncludeField>>, field: IncludeField) -> bool {
    requested.is_some_and(|set| set.contains(&field))
}
