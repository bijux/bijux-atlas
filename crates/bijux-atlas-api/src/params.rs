use crate::errors::ApiError;
use bijux_atlas_query::RegionFilter;
use std::collections::{BTreeMap, BTreeSet};

pub const ALLOWED_FIELDS: [&str; 6] = [
    "gene_id",
    "name",
    "coords",
    "biotype",
    "transcript_count",
    "sequence_length",
];
pub const MAX_CURSOR_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListGenesParams {
    pub release: String,
    pub species: String,
    pub assembly: String,
    pub limit: usize,
    pub cursor: Option<String>,
    pub gene_id: Option<String>,
    pub name: Option<String>,
    pub name_prefix: Option<String>,
    pub biotype: Option<String>,
    pub region: Option<String>,
    pub fields: Option<Vec<String>>,
    pub pretty: bool,
}

pub fn parse_list_genes_params(
    query: &BTreeMap<String, String>,
) -> Result<ListGenesParams, ApiError> {
    parse_list_genes_params_with_limit(query, 100, 500)
}

pub fn parse_list_genes_params_with_limit(
    query: &BTreeMap<String, String>,
    default_limit: usize,
    max_limit: usize,
) -> Result<ListGenesParams, ApiError> {
    let release = query
        .get("release")
        .cloned()
        .ok_or_else(|| ApiError::missing_dataset_dim("release"))?;
    let species = query
        .get("species")
        .cloned()
        .ok_or_else(|| ApiError::missing_dataset_dim("species"))?;
    let assembly = query
        .get("assembly")
        .cloned()
        .ok_or_else(|| ApiError::missing_dataset_dim("assembly"))?;

    let limit = if let Some(raw) = query.get("limit") {
        let value = raw
            .parse::<usize>()
            .map_err(|_| ApiError::invalid_param("limit", raw))?;
        if value == 0 || value > max_limit {
            return Err(ApiError::invalid_param("limit", raw));
        }
        value
    } else {
        default_limit
    };

    let cursor = query.get("cursor").cloned();
    if let Some(value) = &cursor {
        if value.len() > MAX_CURSOR_BYTES {
            return Err(ApiError::invalid_cursor(value));
        }
    }

    let fields = if let Some(raw_fields) = query.get("fields") {
        Some(parse_fields(raw_fields)?)
    } else {
        None
    };

    Ok(ListGenesParams {
        release,
        species,
        assembly,
        limit,
        cursor,
        gene_id: query.get("gene_id").cloned(),
        name: query.get("name").cloned(),
        name_prefix: query.get("name_prefix").cloned(),
        biotype: query.get("biotype").cloned(),
        region: query.get("region").cloned(),
        fields,
        pretty: query
            .get("pretty")
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true")),
    })
}

pub fn parse_region_filter(raw: Option<String>) -> Result<Option<RegionFilter>, ApiError> {
    let Some(value) = raw else {
        return Ok(None);
    };
    let (seqid, coords) = value
        .split_once(':')
        .ok_or_else(|| ApiError::invalid_param("region", &value))?;
    let (start, end) = coords
        .split_once('-')
        .ok_or_else(|| ApiError::invalid_param("region", &value))?;
    let start = start
        .parse::<u64>()
        .map_err(|_| ApiError::invalid_param("region", &value))?;
    let end = end
        .parse::<u64>()
        .map_err(|_| ApiError::invalid_param("region", &value))?;
    if seqid.is_empty() || start == 0 || end < start {
        return Err(ApiError::invalid_param("region", &value));
    }
    Ok(Some(RegionFilter {
        seqid: seqid.to_string(),
        start,
        end,
    }))
}

fn parse_fields(raw_fields: &str) -> Result<Vec<String>, ApiError> {
    let mut ordered_fields = Vec::new();
    let mut seen = BTreeSet::new();
    for raw in raw_fields.split(',') {
        let field = raw.trim();
        if field.is_empty() || !ALLOWED_FIELDS.contains(&field) {
            return Err(ApiError::invalid_param("fields", raw_fields));
        }
        if seen.insert(field.to_string()) {
            ordered_fields.push(field.to_string());
        }
    }
    Ok(ordered_fields)
}
