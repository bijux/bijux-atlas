use crate::cost::estimate_prefix_match_cost;
use crate::filters::GeneQueryRequest;
use crate::limits::QueryLimits;
use bijux_atlas_model::ShardCatalog;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum QueryClass {
    Cheap,
    Medium,
    Heavy,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct QueryCost {
    pub work_units: u64,
}

impl QueryCost {
    #[must_use]
    pub const fn new(work_units: u64) -> Self {
        Self { work_units }
    }
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
    estimate_query_cost(req).work_units
}

#[must_use]
pub fn estimate_query_cost(req: &GeneQueryRequest) -> QueryCost {
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
    QueryCost::new(base + (req.limit as u64) + region_cost)
}

pub fn validate_request(req: &GeneQueryRequest, limits: &QueryLimits) -> Result<(), String> {
    if req.limit == 0 || req.limit > limits.max_limit {
        return Err(format!("limit must be between 1 and {}", limits.max_limit));
    }

    if let Some(prefix) = &req.filter.name_prefix {
        if prefix.len() < limits.min_prefix_len {
            return Err(format!(
                "name_prefix length must be >= {}",
                limits.min_prefix_len
            ));
        }
        if prefix.len() > limits.max_prefix_len {
            return Err(format!(
                "name_prefix length exceeds {}",
                limits.max_prefix_len
            ));
        }
        let prefix_cost = estimate_prefix_match_cost(req);
        if prefix_cost > limits.max_prefix_cost_units {
            return Err(format!(
                "name_prefix estimated cost {} exceeds {}",
                prefix_cost, limits.max_prefix_cost_units
            ));
        }
    }

    if let Some(region) = &req.filter.region {
        if region.start == 0 || region.end < region.start {
            return Err("invalid region span".to_string());
        }
        let span = region.end - region.start + 1;
        if span > limits.max_region_span {
            return Err(format!("region span exceeds {}", limits.max_region_span));
        }
    }

    let has_any_filter = req.filter.gene_id.is_some()
        || req.filter.name.is_some()
        || req.filter.name_prefix.is_some()
        || req.filter.biotype.is_some()
        || req.filter.region.is_some();
    if !has_any_filter && !req.allow_full_scan {
        return Err(
            "full table scan is forbidden without explicit allow_full_scan=true".to_string(),
        );
    }

    let cost = estimate_query_cost(req);
    // Exact gene_id lookups are contractually "cheap" and always allowed.
    if req.filter.gene_id.is_none() && cost.work_units > limits.max_work_units {
        return Err(format!(
            "estimated query cost {} exceeds max_work_units {}",
            cost.work_units, limits.max_work_units
        ));
    }
    Ok(())
}

#[must_use]
pub fn select_shards_for_request(req: &GeneQueryRequest, catalog: &ShardCatalog) -> Vec<String> {
    if let Some(region) = &req.filter.region {
        let mut selected = BTreeSet::new();
        for shard in &catalog.shards {
            if shard.seqids.iter().any(|x| x == &region.seqid) {
                selected.insert(shard.sqlite_path.clone());
            }
        }
        if !selected.is_empty() {
            return selected.into_iter().collect();
        }
    }
    vec!["gene_summary.sqlite".to_string()]
}
