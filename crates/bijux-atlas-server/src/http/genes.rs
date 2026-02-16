#![deny(clippy::redundant_clone)]

use crate::*;
use bijux_atlas_query::{
    prepared_sql_for_class_export, query_gene_by_id_fast, query_gene_id_name_json_minimal_fast,
};
use serde_json::json;
use tracing::{info, info_span, warn};

fn parse_fields(fields: Option<Vec<String>>) -> GeneFields {
    if let Some(list) = fields {
        let mut out = GeneFields {
            gene_id: false,
            name: false,
            coords: false,
            biotype: false,
            transcript_count: false,
            sequence_length: false,
        };
        for field in list {
            match field.as_str() {
                "gene_id" => out.gene_id = true,
                "name" => out.name = true,
                "coords" => out.coords = true,
                "biotype" => out.biotype = true,
                "transcript_count" => out.transcript_count = true,
                "sequence_length" => out.sequence_length = true,
                _ => {}
            }
        }
        out
    } else {
        GeneFields::default()
    }
}

fn parse_region(raw: Option<String>) -> Result<Option<RegionFilter>, ApiError> {
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

async fn acquire_class_permit(
    state: &AppState,
    class: QueryClass,
) -> Result<tokio::sync::OwnedSemaphorePermit, ApiError> {
    let sem = match class {
        QueryClass::Cheap => state.class_cheap.clone(),
        QueryClass::Medium => state.class_medium.clone(),
        QueryClass::Heavy => state.class_heavy.clone(),
    };
    sem.try_acquire_owned().map_err(|_| {
        super::handlers::error_json(
            ApiErrorCode::QueryRejectedByPolicy,
            "concurrency limit reached",
            json!({"class": format!("{class:?}")}),
        )
    })
}

pub(crate) async fn genes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = super::handlers::propagated_request_id(&headers, &state);
    info!(request_id = %request_id, "request start");

    if let Some(ip) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if !state
            .ip_limiter
            .allow(ip, &state.api.rate_limit_per_ip)
            .await
        {
            let resp = super::handlers::api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                super::handlers::error_json(
                    ApiErrorCode::RateLimited,
                    "rate limit exceeded",
                    json!({"scope":"ip"}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    }

    if state.api.enable_api_key_rate_limit {
        if let Some(key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
            if !state
                .api_key_limiter
                .allow(key, &state.api.rate_limit_per_api_key)
                .await
            {
                let resp = super::handlers::api_error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    super::handlers::error_json(
                        ApiErrorCode::RateLimited,
                        "rate limit exceeded",
                        json!({"scope":"api_key"}),
                    ),
                );
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::TOO_MANY_REQUESTS,
                        started.elapsed(),
                    )
                    .await;
                return super::handlers::with_request_id(resp, &request_id);
            }
        }
    }

    let parse_map: std::collections::BTreeMap<String, String> =
        params.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let parsed = match parse_list_genes_params_with_limit(&parse_map, 100, state.limits.max_limit) {
        Ok(v) => v,
        Err(e) => {
            let resp = super::handlers::api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    };

    let dataset = match DatasetId::new(&parsed.release, &parsed.species, &parsed.assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = super::handlers::api_error_response(
                StatusCode::BAD_REQUEST,
                ApiError::invalid_param("dataset", &e.to_string()),
            );
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    };

    let mut req = match parse_region(parsed.region) {
        Ok(region) => GeneQueryRequest {
            fields: parse_fields(parsed.fields),
            filter: GeneFilter {
                gene_id: parsed.gene_id,
                name: parsed.name,
                name_prefix: parsed.name_prefix,
                biotype: parsed.biotype,
                region,
            },
            limit: parsed.limit,
            cursor: parsed.cursor,
            allow_full_scan: false,
        },
        Err(e) => {
            let resp = super::handlers::api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    };

    let exact_gene_id = super::handlers::is_gene_id_exact_query(&req).map(ToString::to_string);
    let redis_cache_key = exact_gene_id.as_ref().map(|gene_id| {
        let dataset_hash = sha256_hex(dataset.canonical_string().as_bytes());
        format!(
            "{dataset_hash}:{gene_id}:{}",
            super::handlers::gene_fields_key(&req.fields)
        )
    });

    let class = classify_query(&req);
    let overloaded = state
        .metrics
        .should_shed_heavy(
            state.api.shed_latency_min_samples,
            state.api.shed_latency_p95_threshold_ms,
        )
        .await;
    if overloaded
        && state.api.allow_min_viable_response
        && super::handlers::wants_min_viable_response(&params)
    {
        req.fields = GeneFields {
            gene_id: true,
            name: true,
            coords: false,
            biotype: false,
            transcript_count: false,
            sequence_length: false,
        };
    }

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
    if estimated_serialized > state.limits.max_serialization_bytes {
        let resp = super::handlers::api_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            super::handlers::error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "serialization budget exceeded",
                json!({"estimated_bytes": estimated_serialized, "max": state.limits.max_serialization_bytes}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/genes",
                StatusCode::UNPROCESSABLE_ENTITY,
                started.elapsed(),
            )
            .await;
        return super::handlers::with_request_id(resp, &request_id);
    }

    if class == QueryClass::Heavy && req.limit > state.limits.heavy_projection_limit {
        let resp = super::handlers::api_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            super::handlers::error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "heavy projection limit exceeded",
                json!({"limit": req.limit, "max": state.limits.heavy_projection_limit}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/genes",
                StatusCode::UNPROCESSABLE_ENTITY,
                started.elapsed(),
            )
            .await;
        return super::handlers::with_request_id(resp, &request_id);
    }

    if (class == QueryClass::Heavy && state.api.shed_load_enabled && overloaded)
        || crate::middleware::shedding::should_shed_noncheap(&state, class).await
    {
        let resp = super::handlers::api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            super::handlers::error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server is shedding heavy query load",
                json!({"class":"heavy"}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/genes",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
            )
            .await;
        return super::handlers::with_request_id(resp, &request_id);
    }

    let _class_permit = match acquire_class_permit(&state, class).await {
        Ok(v) => v,
        Err(e) => {
            let resp = super::handlers::api_error_response(StatusCode::TOO_MANY_REQUESTS, e);
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    };

    let _heavy_worker_permit = if class == QueryClass::Heavy {
        match state.heavy_workers.clone().try_acquire_owned() {
            Ok(permit) => Some(permit),
            Err(_) => {
                let resp = super::handlers::api_error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    super::handlers::error_json(
                        ApiErrorCode::QueryRejectedByPolicy,
                        "heavy worker pool is saturated",
                        json!({"class":"heavy"}),
                    ),
                );
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::TOO_MANY_REQUESTS,
                        started.elapsed(),
                    )
                    .await;
                return super::handlers::with_request_id(resp, &request_id);
            }
        }
    } else {
        None
    };

    let normalized = super::handlers::normalize_query(&params);
    if state.api.enable_redis_response_cache {
        if let (Some(redis), Some(cache_key)) = (&state.redis_backend, &redis_cache_key) {
            match redis.get_gene_cache(cache_key).await {
                Ok(Some(cached_bytes)) => {
                    let etag = format!(
                        "\"{}\"",
                        sha256_hex(
                            format!("{normalized}|{}", String::from_utf8_lossy(&cached_bytes))
                                .as_bytes(),
                        )
                    );
                    if super::handlers::if_none_match(&headers).as_deref() == Some(etag.as_str()) {
                        let mut resp = StatusCode::NOT_MODIFIED.into_response();
                        super::handlers::put_cache_headers(
                            resp.headers_mut(),
                            state.api.immutable_gene_ttl,
                            &etag,
                        );
                        state
                            .metrics
                            .observe_request(
                                "/v1/genes",
                                StatusCode::NOT_MODIFIED,
                                started.elapsed(),
                            )
                            .await;
                        return super::handlers::with_request_id(resp, &request_id);
                    }
                    let mut resp = Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from(cached_bytes))
                        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                    resp.headers_mut()
                        .insert("content-type", HeaderValue::from_static("application/json"));
                    super::handlers::put_cache_headers(
                        resp.headers_mut(),
                        state.api.immutable_gene_ttl,
                        &etag,
                    );
                    if let Ok(v) = HeaderValue::from_str("redis-hit") {
                        resp.headers_mut().insert("x-atlas-cache", v);
                    }
                    state
                        .metrics
                        .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
                        .await;
                    return super::handlers::with_request_id(resp, &request_id);
                }
                Ok(None) => {}
                Err(e) => warn!("redis cache read fallback: {e}"),
            }
        }
    }

    let coalesce_key = format!(
        "{}|{}|{}|{}",
        dataset.canonical_string(),
        format!("{class:?}").to_lowercase(),
        normalized,
        super::handlers::wants_pretty(&params)
    );
    if class == QueryClass::Heavy || class == QueryClass::Cheap {
        let mut cache = state.hot_query_cache.lock().await;
        if let Some(entry) = cache.get(&coalesce_key) {
            let mut resp = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(entry.body))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
            resp.headers_mut()
                .insert("content-type", HeaderValue::from_static("application/json"));
            super::handlers::put_cache_headers(
                resp.headers_mut(),
                state.api.immutable_gene_ttl,
                &entry.etag,
            );
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    }

    let _coalesce_guard = if class == QueryClass::Heavy || class == QueryClass::Cheap {
        Some(state.coalescer.acquire(&coalesce_key).await)
    } else {
        None
    };

    let stage_dataset_resolve_started = Instant::now();
    let work = async {
        info!(request_id = %request_id, dataset = ?dataset, "dataset resolve");
        let c = state.cache.open_dataset_connection(&dataset).await?;
        let _ = c.conn.prepare_cached(prepared_sql_for_class_export(class));
        state
            .metrics
            .observe_stage("dataset_open", stage_dataset_resolve_started.elapsed())
            .await;
        let deadline = Instant::now() + state.api.sql_timeout;
        c.conn
            .progress_handler(1_000, Some(move || Instant::now() > deadline));
        let query_started = Instant::now();
        let query_span = info_span!("sqlite_query", class = %format!("{class:?}").to_lowercase());
        let result = query_span.in_scope(|| {
            if let Some(gene_id) = exact_gene_id.as_ref() {
                if req.fields.gene_id
                    && req.fields.name
                    && !req.fields.coords
                    && !req.fields.biotype
                    && !req.fields.transcript_count
                    && !req.fields.sequence_length
                {
                    if let Some(bytes) = query_gene_id_name_json_minimal_fast(&c.conn, gene_id)
                        .map_err(|e| CacheError(e.to_string()))?
                    {
                        let value: serde_json::Value = serde_json::from_slice(&bytes)
                            .map_err(|e| CacheError(e.to_string()))?;
                        return Ok(bijux_atlas_query::GeneQueryResponse {
                            rows: vec![bijux_atlas_query::GeneRow {
                                gene_id: value
                                    .get("gene_id")
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or_default()
                                    .to_string(),
                                name: value
                                    .get("name")
                                    .and_then(serde_json::Value::as_str)
                                    .map(ToString::to_string),
                                seqid: None,
                                start: None,
                                end: None,
                                biotype: None,
                                transcript_count: None,
                                sequence_length: None,
                            }],
                            next_cursor: None,
                        });
                    }
                }
                let row = query_gene_by_id_fast(&c.conn, gene_id, &req.fields)
                    .map_err(|e| CacheError(e.to_string()))?;
                return Ok(bijux_atlas_query::GeneQueryResponse {
                    rows: row.into_iter().collect(),
                    next_cursor: None,
                });
            }
            query_genes(&c.conn, &req, &state.limits, b"atlas-server-cursor-secret")
                .map_err(|e| CacheError(e.to_string()))
        })?;
        let query_elapsed = query_started.elapsed();
        if query_elapsed > state.api.slow_query_threshold {
            warn!(
                request_id = %request_id,
                dataset = %format!("{}/{}/{}", dataset.release, dataset.species, dataset.assembly),
                class = %format!("{class:?}").to_lowercase(),
                normalized_query = %super::handlers::normalize_query(&params),
                "slow query detected"
            );
        }
        c.conn.progress_handler(1_000, None::<fn() -> bool>);
        Ok::<_, CacheError>((result, query_elapsed))
    };

    let result = timeout(state.api.request_timeout, work).await;
    let payload = match result {
        Ok(Ok((resp, query_elapsed))) => {
            state
                .metrics
                .observe_sqlite_query(&format!("{class:?}").to_lowercase(), query_elapsed)
                .await;
            state.metrics.observe_stage("query", query_elapsed).await;
            json!({"dataset": dataset, "class": format!("{class:?}").to_lowercase(), "response": resp})
        }
        Ok(Err(err)) => {
            let msg = err.to_string();
            if msg.contains("limit") || msg.contains("span") || msg.contains("scan") {
                let resp = super::handlers::api_error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    super::handlers::error_json(
                        ApiErrorCode::QueryRejectedByPolicy,
                        "query rejected",
                        json!({"message": msg}),
                    ),
                );
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::UNPROCESSABLE_ENTITY,
                        started.elapsed(),
                    )
                    .await;
                return super::handlers::with_request_id(resp, &request_id);
            }
            if req.cursor.is_some() {
                let resp = super::handlers::api_error_response(
                    StatusCode::BAD_REQUEST,
                    super::handlers::error_json(
                        ApiErrorCode::InvalidCursor,
                        "invalid cursor",
                        json!({"message": msg}),
                    ),
                );
                state
                    .metrics
                    .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                    .await;
                return super::handlers::with_request_id(resp, &request_id);
            }
            let resp = super::handlers::api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                super::handlers::error_json(
                    ApiErrorCode::Internal,
                    "query failed",
                    json!({"message": msg}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
        Err(_) => {
            let resp = super::handlers::api_error_response(
                StatusCode::GATEWAY_TIMEOUT,
                super::handlers::error_json(ApiErrorCode::Timeout, "request timed out", json!({})),
            );
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::GATEWAY_TIMEOUT, started.elapsed())
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    };

    let serialize_started = Instant::now();
    let bytes = match info_span!("serialize_response").in_scope(|| {
        super::handlers::serialize_payload_with_capacity(
            &payload,
            super::handlers::wants_pretty(&params),
            state.api.response_max_bytes / 4,
        )
    }) {
        Ok(v) => v,
        Err(err) => {
            let resp = super::handlers::api_error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    started.elapsed(),
                )
                .await;
            return super::handlers::with_request_id(resp, &request_id);
        }
    };
    state
        .metrics
        .observe_stage("serialize", serialize_started.elapsed())
        .await;

    if bytes.len() > state.api.response_max_bytes {
        let resp = super::handlers::api_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            super::handlers::error_json(
                ApiErrorCode::ResponseTooLarge,
                "response exceeds configured size guard",
                json!({"bytes": bytes.len(), "max": state.api.response_max_bytes}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/genes",
                StatusCode::PAYLOAD_TOO_LARGE,
                started.elapsed(),
            )
            .await;
        return super::handlers::with_request_id(resp, &request_id);
    }

    let etag = format!(
        "\"{}\"",
        sha256_hex(format!("{normalized}|{}", String::from_utf8_lossy(&bytes)).as_bytes())
    );
    if super::handlers::if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        super::handlers::put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return super::handlers::with_request_id(resp, &request_id);
    }

    if state.api.enable_redis_response_cache {
        if let (Some(redis), Some(cache_key), Some(_)) =
            (&state.redis_backend, &redis_cache_key, &exact_gene_id)
        {
            if let Err(e) = redis
                .set_gene_cache(cache_key, &bytes, state.api.redis_response_cache_ttl_secs)
                .await
            {
                warn!("redis cache write fallback: {e}");
            }
        }
    }

    let (response_bytes, content_encoding) =
        match super::handlers::maybe_compress_response(&headers, &state, bytes) {
            Ok(v) => v,
            Err(err) => {
                let resp =
                    super::handlers::api_error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        started.elapsed(),
                    )
                    .await;
                return super::handlers::with_request_id(resp, &request_id);
            }
        };

    if super::handlers::wants_text(&headers) {
        let text = String::from_utf8_lossy(&response_bytes).to_string();
        let mut resp = (StatusCode::OK, text).into_response();
        super::handlers::put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
            .await;
        return super::handlers::with_request_id(resp, &request_id);
    }

    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(response_bytes.clone()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    resp.headers_mut()
        .insert("content-type", HeaderValue::from_static("application/json"));
    if let Some(encoding) = content_encoding {
        resp.headers_mut()
            .insert("content-encoding", HeaderValue::from_static(encoding));
    }
    super::handlers::put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
    if class == QueryClass::Heavy || class == QueryClass::Cheap {
        let mut cache = state.hot_query_cache.lock().await;
        cache.insert(
            coalesce_key,
            crate::cache::hot::HotEntry {
                body: response_bytes,
                etag: etag.clone(),
                created_at: Instant::now(),
            },
        );
    }
    state
        .metrics
        .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
        .await;
    info!(request_id = %request_id, status = 200_u16, "request complete");
    super::handlers::with_request_id(resp, &request_id)
}
