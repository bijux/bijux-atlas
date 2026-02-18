#![deny(clippy::redundant_clone)]

use crate::http::handlers::{
    api_error_response, error_json, if_none_match, maybe_compress_response, normalize_query,
    put_cache_headers, serialize_payload_with_capacity, with_request_id,
};
use crate::*;
use bijux_atlas_model::{DiffPage, DiffRecord, DiffScope, DiffStatus, ReleaseGeneIndexEntry};
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

struct QueueGuard {
    counter: Arc<AtomicU64>,
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

fn parse_dataset_dims(params: &HashMap<String, String>) -> Result<(String, String), ApiError> {
    let species = params
        .get("species")
        .ok_or_else(|| ApiError::missing_dataset_dim("species"))?
        .to_string();
    let assembly = params
        .get("assembly")
        .ok_or_else(|| ApiError::missing_dataset_dim("assembly"))?
        .to_string();
    Ok((species, assembly))
}

fn parse_region(params: &HashMap<String, String>) -> Result<Option<RegionFilter>, ApiError> {
    let Some(raw) = params.get("region") else {
        return Ok(None);
    };
    let (seqid, span) = raw.split_once(':').ok_or_else(|| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"value": raw}),
        )
    })?;
    let (start, end) = span.split_once('-').ok_or_else(|| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"value": raw}),
        )
    })?;
    let start = start.parse::<u64>().map_err(|_| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"value": raw}),
        )
    })?;
    let end = end.parse::<u64>().map_err(|_| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"value": raw}),
        )
    })?;
    if start == 0 || end < start {
        return Err(error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"value": raw}),
        ));
    }
    Ok(Some(RegionFilter {
        seqid: seqid.to_string(),
        start,
        end,
    }))
}

fn overlaps(entry: &ReleaseGeneIndexEntry, region: &RegionFilter) -> bool {
    entry.seqid == region.seqid && entry.start <= region.end && region.start <= entry.end
}

fn load_index(path: &std::path::Path) -> Result<Vec<ReleaseGeneIndexEntry>, ApiError> {
    let bytes = crate::http::effects_adapters::read_bytes(path).map_err(|e| {
        error_json(
            ApiErrorCode::Internal,
            "release index read failed",
            json!({"message": e.0}),
        )
    })?;
    let idx: bijux_atlas_model::ReleaseGeneIndex = serde_json::from_slice(&bytes).map_err(|e| {
        error_json(
            ApiErrorCode::Internal,
            "release index parse failed",
            json!({"message": e.to_string()}),
        )
    })?;
    Ok(idx.entries)
}

fn resolve_explicit_release_alias(
    release: &str,
    species: &str,
    assembly: &str,
    catalog: &bijux_atlas_model::Catalog,
) -> Option<String> {
    if release != "latest" {
        return Some(release.to_string());
    }
    let mut candidates: Vec<String> = catalog
        .datasets
        .iter()
        .filter(|x| {
            x.dataset.species.as_str() == species && x.dataset.assembly.as_str() == assembly
        })
        .map(|x| x.dataset.release.as_str().to_string())
        .collect();
    if candidates.is_empty() {
        return None;
    }
    candidates.sort();
    candidates.last().cloned()
}

async fn diff_common(
    State(state): State<AppState>,
    headers: HeaderMap,
    params: HashMap<String, String>,
    scope: DiffScope,
) -> Response {
    let started = Instant::now();
    let request_id = crate::http::handlers::propagated_request_id(&headers, &state);
    let route = match scope {
        DiffScope::Genes => "/v1/diff/genes",
        DiffScope::Region => "/v1/diff/region",
        _ => "/v1/diff/genes",
    };
    info!(request_id = %request_id, route = route, "request start");
    let queue_depth = state
        .queued_requests
        .fetch_add(1, Ordering::Relaxed)
        .saturating_add(1);
    if queue_depth as usize > state.api.max_request_queue_depth {
        state.queued_requests.fetch_sub(1, Ordering::Relaxed);
        let resp = api_error_response(
            StatusCode::TOO_MANY_REQUESTS,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "request queue depth exceeded",
                json!({"depth": queue_depth, "max": state.api.max_request_queue_depth}),
            ),
        );
        return with_request_id(resp, &request_id);
    }
    let _queue_guard = QueueGuard {
        counter: Arc::clone(&state.queued_requests),
    };

    let (species, assembly) = match parse_dataset_dims(&params) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            return with_request_id(resp, &request_id);
        }
    };
    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| bijux_atlas_model::Catalog::new(Vec::new()));

    let from_release_raw = params.get("from_release").map(String::as_str).unwrap_or("");
    let to_release_raw = params.get("to_release").map(String::as_str).unwrap_or("");
    if from_release_raw.is_empty() || to_release_raw.is_empty() {
        let resp = api_error_response(
            StatusCode::BAD_REQUEST,
            ApiError::invalid_param("from_release/to_release", "missing"),
        );
        return with_request_id(resp, &request_id);
    }
    let Some(from_release) =
        resolve_explicit_release_alias(from_release_raw, &species, &assembly, &catalog)
    else {
        let resp = api_error_response(
            StatusCode::NOT_FOUND,
            error_json(
                ApiErrorCode::DatasetNotFound,
                "from_release alias unresolved",
                json!({"from_release": from_release_raw}),
            ),
        );
        return with_request_id(resp, &request_id);
    };
    let Some(to_release) =
        resolve_explicit_release_alias(to_release_raw, &species, &assembly, &catalog)
    else {
        let resp = api_error_response(
            StatusCode::NOT_FOUND,
            error_json(
                ApiErrorCode::DatasetNotFound,
                "to_release alias unresolved",
                json!({"to_release": to_release_raw}),
            ),
        );
        return with_request_id(resp, &request_id);
    };

    let from_dataset = match DatasetId::new(&from_release, &species, &assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                ApiError::invalid_param("from_release", &e.to_string()),
            );
            return with_request_id(resp, &request_id);
        }
    };
    let to_dataset = match DatasetId::new(&to_release, &species, &assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                ApiError::invalid_param("to_release", &e.to_string()),
            );
            return with_request_id(resp, &request_id);
        }
    };

    let mut limit = params
        .get("limit")
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(100)
        .min(state.limits.max_limit);
    let class = QueryClass::Heavy;
    let overloaded = crate::middleware::shedding::overloaded(&state).await;
    if crate::middleware::shedding::should_shed_noncheap(&state, class).await
        || (state.api.shed_load_enabled && overloaded)
    {
        let backoff = crate::middleware::shedding::heavy_backoff_ms(&state);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
        let mut resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server is shedding non-cheap query load",
                json!({"class":"Heavy","retry_after_ms": backoff}),
            ),
        );
        if let Ok(v) = HeaderValue::from_str(&(backoff / 1000).max(1).to_string()) {
            resp.headers_mut().insert("retry-after", v);
        }
        return with_request_id(resp, &request_id);
    }
    if overloaded {
        let adaptive_max = ((state.limits.max_limit as f64) * state.api.adaptive_heavy_limit_factor)
            .max(1.0) as usize;
        limit = limit.min(adaptive_max);
    }
    let _class_permit = match state.class_heavy.clone().try_acquire_owned() {
        Ok(v) => v,
        Err(_) => {
            let resp = api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                error_json(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "heavy query concurrency limit exceeded",
                    json!({"class":"Heavy"}),
                ),
            );
            return with_request_id(resp, &request_id);
        }
    };

    let query_hash = sha256_hex(normalize_query(&params).as_bytes());
    let coalesce_key = format!("{route}:{scope:?}:{query_hash}");
    let _coalesce_guard = state.coalescer.acquire(&coalesce_key).await;
    let cursor_gene = if let Some(token) = params.get("cursor") {
        match decode_cursor(
            token,
            b"atlas-diff-cursor",
            &query_hash,
            OrderMode::GeneId,
            None,
        ) {
            Ok(c) => Some(c.last_gene_id),
            Err(e) => {
                let message = e.to_string();
                let reason_code = if message.contains("UnsupportedVersion") {
                    "CURSOR_VERSION_UNSUPPORTED"
                } else if message.contains("DatasetMismatch") {
                    "CURSOR_DATASET_MISMATCH"
                } else {
                    "CURSOR_INVALID"
                };
                let resp = api_error_response(
                    StatusCode::BAD_REQUEST,
                    error_json(
                        ApiErrorCode::InvalidCursor,
                        "invalid cursor",
                        json!({"message": message, "reason_code": reason_code}),
                    ),
                );
                return with_request_id(resp, &request_id);
            }
        }
    } else {
        None
    };
    let region = match scope {
        DiffScope::Genes => None,
        DiffScope::Region => match parse_region(&params) {
            Ok(v) => v,
            Err(e) => {
                let resp = api_error_response(StatusCode::BAD_REQUEST, e);
                return with_request_id(resp, &request_id);
            }
        },
        _ => None,
    };

    let from_index_path = match state
        .cache
        .ensure_release_gene_index_cached(&from_dataset)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "from_release index unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            return with_request_id(resp, &request_id);
        }
    };
    let to_index_path = match state
        .cache
        .ensure_release_gene_index_cached(&to_dataset)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "to_release index unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            return with_request_id(resp, &request_id);
        }
    };
    let from_entries = match load_index(&from_index_path) {
        Ok(v) => v,
        Err(e) => {
            return with_request_id(
                api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e),
                &request_id,
            )
        }
    };
    let to_entries = match load_index(&to_index_path) {
        Ok(v) => v,
        Err(e) => {
            return with_request_id(
                api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e),
                &request_id,
            )
        }
    };

    let mut i = 0usize;
    let mut j = 0usize;
    let mut rows: Vec<DiffRecord> = Vec::new();
    while i < from_entries.len() || j < to_entries.len() {
        let next = match (from_entries.get(i), to_entries.get(j)) {
            (Some(a), Some(b)) => match a.gene_id.cmp(&b.gene_id) {
                std::cmp::Ordering::Less => {
                    i += 1;
                    Some((a.gene_id.clone(), DiffStatus::Removed, Some(a)))
                }
                std::cmp::Ordering::Greater => {
                    j += 1;
                    Some((b.gene_id.clone(), DiffStatus::Added, Some(b)))
                }
                std::cmp::Ordering::Equal => {
                    i += 1;
                    j += 1;
                    if a.signature_sha256 != b.signature_sha256 {
                        Some((a.gene_id.clone(), DiffStatus::Changed, Some(b)))
                    } else {
                        None
                    }
                }
            },
            (Some(a), None) => {
                i += 1;
                Some((a.gene_id.clone(), DiffStatus::Removed, Some(a)))
            }
            (None, Some(b)) => {
                j += 1;
                Some((b.gene_id.clone(), DiffStatus::Added, Some(b)))
            }
            (None, None) => None,
        };
        let Some((gene_id, status, coord_src)) = next else {
            continue;
        };
        if cursor_gene.as_ref().is_some_and(|c| &gene_id <= c) {
            continue;
        }
        if let Some(r) = &region {
            let Some(src) = coord_src else {
                continue;
            };
            if !overlaps(src, r) {
                continue;
            }
        }
        rows.push(DiffRecord::new(
            gene_id.clone(),
            status,
            coord_src.map(|x| x.seqid.clone()),
            coord_src.map(|x| x.start),
            coord_src.map(|x| x.end),
        ));
        if rows.len() > limit {
            break;
        }
    }

    let has_more = rows.len() > limit;
    if has_more {
        rows.truncate(limit);
    }
    let next_cursor = if has_more {
        rows.last().and_then(|x| {
            encode_cursor(
                &CursorPayload {
                    cursor_version: "v1".to_string(),
                    dataset_id: None,
                    sort_key: Some("gene_id".to_string()),
                    last_seen: Some(bijux_atlas_query::CursorLastSeen {
                        gene_id: x.gene_id.clone(),
                        seqid: None,
                        start: None,
                    }),
                    order: "gene_id".to_string(),
                    last_seqid: None,
                    last_start: None,
                    last_gene_id: x.gene_id.clone(),
                    query_hash: query_hash.clone(),
                    depth: 0,
                },
                b"atlas-diff-cursor",
            )
            .ok()
        })
    } else {
        None
    };

    let page = DiffPage::new(
        from_release.clone(),
        to_release.clone(),
        species.clone(),
        assembly.clone(),
        scope,
        rows,
        next_cursor,
    );
    let added = page
        .rows
        .iter()
        .filter(|x| matches!(x.status, DiffStatus::Added))
        .count();
    let removed = page
        .rows
        .iter()
        .filter(|x| matches!(x.status, DiffStatus::Removed))
        .count();
    let changed = page
        .rows
        .iter()
        .filter(|x| matches!(x.status, DiffStatus::Changed))
        .count();
    let qc = json!({
        "added": added,
        "removed": removed,
        "changed": changed,
        "count_consistent": (added + removed + changed) == page.rows.len(),
        "from_release": from_release,
        "to_release": to_release
    });
    let provenance = crate::http::handlers::dataset_provenance(&state, &to_dataset).await;
    let payload = crate::http::handlers::json_envelope(
        Some(json!(to_dataset)),
        Some(json!({ "next_cursor": page.next_cursor.clone() })),
        json!({"diff": page.rows, "qc": qc, "provenance": provenance}),
        page.next_cursor.map(|c| json!({ "next_cursor": c })),
        None,
    );
    let etag = format!(
        "\"{}\"",
        sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
        state
            .metrics
            .observe_request(route, StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let body =
        match serialize_payload_with_capacity(&payload, false, payload.to_string().len() + 64) {
            Ok(v) => v,
            Err(e) => {
                return with_request_id(
                    api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e),
                    &request_id,
                )
            }
        };
    let (encoded, encoding) = match maybe_compress_response(&headers, &state, body) {
        Ok(v) => v,
        Err(e) => {
            return with_request_id(
                api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e),
                &request_id,
            )
        }
    };
    if encoded.len() > state.api.response_max_bytes {
        let resp = api_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "response size exceeds configured limit",
                json!({"size_bytes": encoded.len(), "max": state.api.response_max_bytes}),
            ),
        );
        return with_request_id(resp, &request_id);
    }
    let mut out_headers = HeaderMap::new();
    put_cache_headers(&mut out_headers, state.api.immutable_gene_ttl, &etag);
    out_headers.insert(
        "content-type",
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    if let Some(enc) = encoding {
        out_headers.insert("content-encoding", HeaderValue::from_static(enc));
    }
    let resp = (StatusCode::OK, out_headers, encoded).into_response();
    state
        .metrics
        .observe_request(route, StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn diff_genes_handler(
    state: State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    diff_common(state, headers, params, DiffScope::Genes).await
}

pub(crate) async fn diff_region_handler(
    state: State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    diff_common(state, headers, params, DiffScope::Region).await
}
