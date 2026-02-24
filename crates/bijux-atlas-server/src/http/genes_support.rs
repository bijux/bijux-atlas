// SPDX-License-Identifier: Apache-2.0

use crate::*;
use bijux_atlas_api::params::{IncludeField, SortKey};
use serde_json::json;

pub(super) struct QueueGuard {
    counter: Arc<AtomicU64>,
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(super) fn try_enter_queue(state: &AppState) -> Result<QueueGuard, ApiError> {
    let depth = state
        .queued_requests
        .fetch_add(1, Ordering::Relaxed)
        .saturating_add(1);
    if depth as usize > state.api.max_request_queue_depth {
        state.queued_requests.fetch_sub(1, Ordering::Relaxed);
        state
            .cache
            .metrics
            .policy_violations_total
            .fetch_add(1, Ordering::Relaxed);
        return Err(super::handlers::error_json(
            ApiErrorCode::QueryRejectedByPolicy,
            "request queue depth exceeded",
            json!({"depth": depth, "max": state.api.max_request_queue_depth}),
        ));
    }
    Ok(QueueGuard {
        counter: Arc::clone(&state.queued_requests),
    })
}

pub(super) fn parse_include(include: Option<Vec<IncludeField>>) -> GeneFields {
    if let Some(list) = include {
        let mut out = GeneFields {
            gene_id: true,
            name: true,
            coords: false,
            biotype: false,
            transcript_count: false,
            sequence_length: false,
        };
        for field in list {
            match field {
                IncludeField::Coords => out.coords = true,
                IncludeField::Biotype => out.biotype = true,
                IncludeField::Counts => out.transcript_count = true,
                IncludeField::Length => out.sequence_length = true,
            }
        }
        out
    } else {
        GeneFields {
            gene_id: true,
            name: true,
            coords: false,
            biotype: false,
            transcript_count: false,
            sequence_length: false,
        }
    }
}

pub(super) fn parse_region(raw: Option<String>) -> Result<Option<RegionFilter>, ApiError> {
    if let Some(value) = raw {
        let (seqid, span) = value.split_once(':').ok_or_else(|| {
            super::handlers::error_json(
                ApiErrorCode::InvalidQueryParameter,
                "invalid region",
                json!({"value": value}),
            )
        })?;
        let (start, end) = span.split_once('-').ok_or_else(|| {
            super::handlers::error_json(
                ApiErrorCode::InvalidQueryParameter,
                "invalid region",
                json!({"value": value}),
            )
        })?;
        let start = start.parse::<u64>().map_err(|_| {
            super::handlers::error_json(
                ApiErrorCode::InvalidQueryParameter,
                "invalid region",
                json!({"value": value}),
            )
        })?;
        let end = end.parse::<u64>().map_err(|_| {
            super::handlers::error_json(
                ApiErrorCode::InvalidQueryParameter,
                "invalid region",
                json!({"value": value}),
            )
        })?;
        return Ok(Some(RegionFilter {
            seqid: seqid.to_string(),
            start,
            end,
        }));
    }
    Ok(None)
}

pub(super) async fn acquire_class_permit(
    state: &AppState,
    class: QueryClass,
) -> Result<tokio::sync::OwnedSemaphorePermit, ApiError> {
    let sem = match class {
        QueryClass::Cheap => state.class_cheap.clone(),
        QueryClass::Medium => state.class_medium.clone(),
        QueryClass::Heavy => state.class_heavy.clone(),
        _ => state.class_heavy.clone(),
    };
    sem.try_acquire_owned().map_err(|_| {
        state
            .cache
            .metrics
            .policy_violations_total
            .fetch_add(1, Ordering::Relaxed);
        super::handlers::error_json(
            ApiErrorCode::QueryRejectedByPolicy,
            "concurrency limit reached",
            json!({"class": format!("{class:?}")}),
        )
    })
}

pub(super) fn check_serialization_budget(
    req: &GeneQueryRequest,
    limits: &QueryLimits,
) -> Option<ApiError> {
    let selected_fields = [
        req.fields.gene_id,
        req.fields.name,
        req.fields.coords,
        req.fields.biotype,
        req.fields.transcript_count,
        req.fields.sequence_length,
    ]
    .into_iter()
    .filter(|x| *x)
    .count();
    let estimated_serialized = req
        .limit
        .saturating_mul(32 + selected_fields.saturating_mul(32));
    if estimated_serialized > limits.max_serialization_bytes {
        return Some(super::handlers::error_json(
            ApiErrorCode::QueryRejectedByPolicy,
            "serialization budget exceeded",
            json!({"estimated_bytes": estimated_serialized, "max": limits.max_serialization_bytes}),
        ));
    }
    None
}

pub(super) fn build_dataset_query(
    params: &HashMap<String, String>,
    max_limit: usize,
) -> Result<(DatasetId, GeneQueryRequest), ApiError> {
    let parse_map: std::collections::BTreeMap<String, String> =
        params.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let parsed =
        bijux_atlas_api::params::parse_list_genes_params_with_limit(&parse_map, 100, max_limit)?;
    let dataset = DatasetId::new(&parsed.release, &parsed.species, &parsed.assembly)
        .map_err(|e| ApiError::invalid_param("dataset", &e.to_string()))?;
    if parsed.min_transcripts.is_some() || parsed.max_transcripts.is_some() {
        return Err(super::handlers::error_json(
            ApiErrorCode::InvalidQueryParameter,
            "min_transcripts/max_transcripts are not yet supported",
            json!({}),
        ));
    }
    if let Some(sort) = parsed.sort {
        match sort {
            SortKey::GeneIdAsc => {}
            SortKey::RegionAsc => {
                if parsed.range.is_none() {
                    return Err(super::handlers::error_json(
                        ApiErrorCode::InvalidQueryParameter,
                        "sort=region:asc requires range filter",
                        json!({}),
                    ));
                }
            }
        }
    }
    let region = parse_region(parsed.range)?;
    let name_prefix = parsed.name_like.as_ref().map(|v| v.trim_end_matches('*'));
    let req = GeneQueryRequest {
        fields: parse_include(parsed.include),
        filter: GeneFilter {
            gene_id: parsed.gene_id,
            name: parsed.name,
            name_prefix: name_prefix.map(ToString::to_string),
            biotype: parsed.biotype,
            region,
        },
        limit: parsed.limit,
        cursor: parsed.cursor,
        dataset_key: Some(dataset.canonical_string()),
        allow_full_scan: false,
    };
    Ok((dataset, req))
}

pub(super) fn exact_lookup_cache_keys(
    dataset: &DatasetId,
    req: &GeneQueryRequest,
) -> (Option<String>, Option<String>) {
    let exact_gene_id = super::handlers::is_gene_id_exact_query(req).map(ToString::to_string);
    let redis_cache_key = exact_gene_id.as_ref().map(|gene_id| {
        let dataset_hash = sha256_hex(dataset.canonical_string().as_bytes());
        format!(
            "{dataset_hash}:{gene_id}:{}",
            super::handlers::gene_fields_key(&req.fields)
        )
    });
    (exact_gene_id, redis_cache_key)
}

pub(super) fn record_overload_cheap(state: &AppState, class: QueryClass, overloaded: bool) {
    if overloaded && class == QueryClass::Cheap {
        state
            .cache
            .metrics
            .cheap_queries_served_while_overloaded_total
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

pub(super) fn adaptive_rl_factor(state: &AppState, overloaded_early: bool) -> f64 {
    if overloaded_early {
        state.api.adaptive_rate_limit_factor
    } else {
        1.0
    }
}

pub(super) fn cap_heavy_limit(
    req: &mut GeneQueryRequest,
    state: &AppState,
    class: QueryClass,
    overloaded: bool,
) {
    if overloaded && class == QueryClass::Heavy {
        let adaptive_max = ((state.limits.heavy_projection_limit as f64)
            * state.api.adaptive_heavy_limit_factor)
            .max(1.0) as usize;
        req.limit = req.limit.min(adaptive_max);
    }
}
