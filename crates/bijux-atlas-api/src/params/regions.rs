// SPDX-License-Identifier: Apache-2.0

use super::types::MAX_RANGE_SPAN;
use crate::errors::ApiError;
use bijux_atlas_model::query::RegionFilter;

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
    if seqid.is_empty() {
        return Err(ApiError::invalid_param("region", "contig is required"));
    }
    if start == 0 {
        return Err(ApiError::invalid_param(
            "region",
            "start must be >= 1 (1-based closed coordinates)",
        ));
    }
    if end < start {
        return Err(ApiError::invalid_param(
            "region",
            "end must be >= start (1-based closed coordinates)",
        ));
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
        return Err(ApiError::invalid_param(
            "range",
            "start must be >= 1 (1-based closed coordinates)",
        ));
    }
    if end < start {
        return Err(ApiError::invalid_param(
            "range",
            "end must be >= start (1-based closed coordinates)",
        ));
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
