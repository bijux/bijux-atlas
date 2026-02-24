use crate::filters::GeneQueryRequest;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    GeneId(String),
    NameEquals(String),
    NamePrefix(String),
    Biotype(String),
    Region { seqid: String, start: u64, end: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    GeneId,
    Region,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneQueryAst {
    pub predicates: Vec<Predicate>,
    pub limit: usize,
    pub dataset_key: Option<String>,
    pub allow_full_scan: bool,
    pub has_cursor: bool,
    pub sort_key: SortKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidLimit,
    InvalidRegionSpan,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimit => f.write_str("limit must be > 0"),
            Self::InvalidRegionSpan => f.write_str("region start must be <= end and >= 1"),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_gene_query(req: &GeneQueryRequest) -> Result<GeneQueryAst, ParseError> {
    if req.limit == 0 {
        return Err(ParseError::InvalidLimit);
    }

    let mut predicates = Vec::new();

    if let Some(v) = &req.filter.gene_id {
        predicates.push(Predicate::GeneId(v.clone()));
    }
    if let Some(v) = &req.filter.name {
        predicates.push(Predicate::NameEquals(v.clone()));
    }
    if let Some(v) = &req.filter.name_prefix {
        predicates.push(Predicate::NamePrefix(v.clone()));
    }
    if let Some(v) = &req.filter.biotype {
        predicates.push(Predicate::Biotype(v.clone()));
    }
    if let Some(v) = &req.filter.region {
        if v.start == 0 || v.end < v.start {
            return Err(ParseError::InvalidRegionSpan);
        }
        predicates.push(Predicate::Region {
            seqid: v.seqid.clone(),
            start: v.start,
            end: v.end,
        });
    }

    let sort_key = if req.filter.region.is_some() {
        SortKey::Region
    } else {
        SortKey::GeneId
    };

    Ok(GeneQueryAst {
        predicates,
        limit: req.limit,
        dataset_key: req.dataset_key.clone(),
        allow_full_scan: req.allow_full_scan,
        has_cursor: req.cursor.is_some(),
        sort_key,
    })
}
