use crate::errors::ApiError;
use bijux_atlas_query::RegionFilter;
use std::collections::{BTreeMap, BTreeSet};

pub const ALLOWED_INCLUDE: [&str; 4] = [
    "coords",
    "biotype",
    "counts",
    "length",
];
pub const MAX_CURSOR_BYTES: usize = 4096;
pub const MAX_FILTER_COUNT: usize = 6;
pub const MAX_RANGE_SPAN: u64 = 5_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IncludeField {
    Coords,
    Biotype,
    Counts,
    Length,
}

impl IncludeField {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "coords" => Some(Self::Coords),
            "biotype" => Some(Self::Biotype),
            "counts" => Some(Self::Counts),
            "length" => Some(Self::Length),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListGenesParams {
    pub release: String,
    pub species: String,
    pub assembly: String,
    pub limit: usize,
    pub cursor: Option<String>,
    pub gene_id: Option<String>,
    pub name: Option<String>,
    pub name_like: Option<String>,
    pub biotype: Option<String>,
    pub contig: Option<String>,
    pub range: Option<String>,
    pub min_transcripts: Option<u64>,
    pub max_transcripts: Option<u64>,
    pub include: Option<Vec<IncludeField>>,
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
    validate_known_filters(query)?;
    if query.contains_key("fields") {
        return Err(ApiError::invalid_param(
            "fields",
            "unsupported; use include=coords,biotype,counts,length",
        ));
    }
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

    let include = if let Some(raw_include) = query.get("include") {
        Some(parse_include(raw_include)?)
    } else {
        None
    };
    let name_like = query.get("name_like").cloned();
    if let Some(pattern) = &name_like {
        if pattern.starts_with('*')
            || pattern.contains('%')
            || pattern.contains('?')
            || (!pattern.ends_with('*') && pattern.contains('*'))
        {
            return Err(ApiError::invalid_param(
                "name_like",
                "only prefix wildcard is supported (example: BRCA*)",
            ));
        }
    }
    let min_transcripts = parse_u64_opt(query, "min_transcripts")?;
    let max_transcripts = parse_u64_opt(query, "max_transcripts")?;
    if let (Some(min), Some(max)) = (min_transcripts, max_transcripts) {
        if min > max {
            return Err(ApiError::invalid_param(
                "min_transcripts",
                "must be <= max_transcripts",
            ));
        }
    }

    let active_filters = [
        query.get("gene_id").is_some(),
        query.get("name").is_some(),
        name_like.is_some(),
        query.get("biotype").is_some(),
        query.get("contig").is_some(),
        query.get("range").is_some() || query.get("region").is_some(),
        min_transcripts.is_some(),
        max_transcripts.is_some(),
    ]
    .into_iter()
    .filter(|active| *active)
    .count();
    if active_filters > MAX_FILTER_COUNT {
        return Err(ApiError::invalid_param(
            "filters",
            &format!("too many filters; max {MAX_FILTER_COUNT}"),
        ));
    }
    let range = query
        .get("range")
        .cloned()
        .or_else(|| query.get("region").cloned());
    let parsed_range = parse_range_filter(range.clone())?;
    if let Some(contig) = query.get("contig") {
        let Some(region) = parsed_range.as_ref() else {
            return Err(ApiError::invalid_param(
                "contig",
                "contig requires range=contig:start-end",
            ));
        };
        if region.seqid != *contig {
            return Err(ApiError::invalid_param(
                "contig",
                "contig must match range contig",
            ));
        }
    }

    Ok(ListGenesParams {
        release,
        species,
        assembly,
        limit,
        cursor,
        gene_id: query.get("gene_id").cloned(),
        name: query.get("name").cloned(),
        name_like,
        biotype: query.get("biotype").cloned(),
        contig: query.get("contig").cloned(),
        range,
        min_transcripts,
        max_transcripts,
        include,
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

pub fn parse_range_filter(raw: Option<String>) -> Result<Option<RegionFilter>, ApiError> {
    let Some(value) = raw else {
        return Ok(None);
    };
    let (seqid, coords) = value
        .split_once(':')
        .ok_or_else(|| ApiError::invalid_param("range", "expected chr:start-end"))?;
    let (start, end) = coords
        .split_once('-')
        .ok_or_else(|| ApiError::invalid_param("range", "expected chr:start-end"))?;
    let start = start
        .parse::<u64>()
        .map_err(|_| ApiError::invalid_param("range", "start must be an integer"))?;
    let end = end
        .parse::<u64>()
        .map_err(|_| ApiError::invalid_param("range", "end must be an integer"))?;
    if seqid.is_empty() {
        return Err(ApiError::invalid_param("range", "contig is required"));
    }
    if start == 0 {
        return Err(ApiError::invalid_param("range", "start must be >= 1"));
    }
    if end < start {
        return Err(ApiError::invalid_param("range", "end must be >= start"));
    }
    let span = end - start + 1;
    if span > MAX_RANGE_SPAN {
        return Err(ApiError::invalid_param(
            "range",
            &format!("span exceeds {MAX_RANGE_SPAN} bases"),
        ));
    }
    Ok(Some(RegionFilter {
        seqid: seqid.to_string(),
        start,
        end,
    }))
}

fn parse_include(raw_include: &str) -> Result<Vec<IncludeField>, ApiError> {
    let mut ordered_fields = Vec::new();
    let mut seen = BTreeSet::new();
    for raw in raw_include.split(',') {
        let field = raw.trim();
        if field.is_empty() || !ALLOWED_INCLUDE.contains(&field) {
            return Err(ApiError::invalid_param("include", raw_include));
        }
        let parsed = IncludeField::parse(field)
            .ok_or_else(|| ApiError::invalid_param("include", raw_include))?;
        if seen.insert(parsed) {
            ordered_fields.push(parsed);
        }
    }
    Ok(ordered_fields)
}

fn parse_u64_opt(
    query: &BTreeMap<String, String>,
    key: &'static str,
) -> Result<Option<u64>, ApiError> {
    let Some(raw) = query.get(key) else {
        return Ok(None);
    };
    let value = raw
        .parse::<u64>()
        .map_err(|_| ApiError::invalid_param(key, raw))?;
    Ok(Some(value))
}

fn validate_known_filters(query: &BTreeMap<String, String>) -> Result<(), ApiError> {
    const ALLOWED_PARAMS: [&str; 18] = [
        "release",
        "species",
        "assembly",
        "limit",
        "cursor",
        "gene_id",
        "name",
        "name_like",
        "biotype",
        "contig",
        "range",
        "region",
        "min_transcripts",
        "max_transcripts",
        "include",
        "pretty",
        "explain",
        "fields",
    ];
    let mut unknown = query
        .keys()
        .filter(|k| !ALLOWED_PARAMS.contains(&k.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if unknown.is_empty() {
        return Ok(());
    }
    unknown.sort();
    Err(ApiError::invalid_param(
        "filter",
        &format!(
            "unknown filter(s): {}; allowed: gene_id,name,name_like,biotype,contig,range,min_transcripts,max_transcripts",
            unknown.join(",")
        ),
    ))
}
