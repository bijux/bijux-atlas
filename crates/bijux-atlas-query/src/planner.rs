use crate::filters::GeneQueryRequest;
use crate::limits::QueryLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QueryClass {
    Cheap,
    Medium,
    Heavy,
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
    base + (req.limit as u64) + region_cost
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

    let work = estimate_work_units(req);
    if work > limits.max_work_units {
        return Err(format!(
            "estimated query cost {} exceeds max_work_units {}",
            work, limits.max_work_units
        ));
    }
    Ok(())
}
