#![deny(clippy::redundant_clone)]

use crate::*;
use bijux_atlas_query::query_gene_by_id_fast;
use brotli::CompressorWriter;
use flate2::{write::GzEncoder, Compression};
use serde_json::json;
use serde_json::Value;
use std::io::Write;
use tracing::{info, info_span, warn};

fn api_error_response(status: StatusCode, err: ApiError) -> Response {
    let body = Json(json!({"error": err}));
    (status, body).into_response()
}

fn error_json(code: ApiErrorCode, message: &str, details: Value) -> ApiError {
    ApiError {
        code,
        message: message.to_string(),
        details,
    }
}

fn normalize_query(params: &HashMap<String, String>) -> String {
    let mut kv: Vec<(&String, &String)> = params.iter().collect();
    kv.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(b.1)));
    kv.into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get("if-none-match")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string)
}

fn put_cache_headers(headers: &mut HeaderMap, ttl: Duration, etag: &str) {
    if let Ok(value) = HeaderValue::from_str(&format!("public, max-age={}", ttl.as_secs())) {
        headers.insert("cache-control", value);
    }
    if let Ok(value) = HeaderValue::from_str(etag) {
        headers.insert("etag", value);
    }
}

fn wants_pretty(params: &HashMap<String, String>) -> bool {
    params
        .get("pretty")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn wants_text(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/plain"))
}

fn accepted_encoding(headers: &HeaderMap) -> Option<&'static str> {
    let accept = headers
        .get("accept-encoding")
        .and_then(|v| v.to_str().ok())?;
    if accept.contains("br") {
        Some("br")
    } else if accept.contains("gzip") {
        Some("gzip")
    } else {
        None
    }
}

fn serialize_payload_with_capacity(
    payload: &Value,
    pretty: bool,
    capacity_hint: usize,
) -> Result<Vec<u8>, ApiError> {
    let mut out = Vec::with_capacity(capacity_hint);
    if pretty {
        serde_json::to_writer_pretty(&mut out, payload).map_err(|e| {
            error_json(
                ApiErrorCode::Internal,
                "json serialization failed",
                json!({"message": e.to_string()}),
            )
        })?;
    } else {
        serde_json::to_writer(&mut out, payload).map_err(|e| {
            error_json(
                ApiErrorCode::Internal,
                "json serialization failed",
                json!({"message": e.to_string()}),
            )
        })?;
    }
    Ok(out)
}

fn maybe_compress_response(
    headers: &HeaderMap,
    state: &AppState,
    bytes: Vec<u8>,
) -> Result<(Vec<u8>, Option<&'static str>), ApiError> {
    if !state.api.enable_response_compression || bytes.len() < state.api.compression_min_bytes {
        return Ok((bytes, None));
    }
    match accepted_encoding(headers) {
        Some("gzip") => {
            let mut encoder = GzEncoder::new(
                Vec::with_capacity((bytes.len() / 2).max(256)),
                Compression::fast(),
            );
            encoder.write_all(&bytes).map_err(|e| {
                error_json(
                    ApiErrorCode::Internal,
                    "gzip encoding failed",
                    json!({"message": e.to_string()}),
                )
            })?;
            let compressed = encoder.finish().map_err(|e| {
                error_json(
                    ApiErrorCode::Internal,
                    "gzip finalize failed",
                    json!({"message": e.to_string()}),
                )
            })?;
            Ok((compressed, Some("gzip")))
        }
        Some("br") => {
            let mut compressed = Vec::with_capacity((bytes.len() / 2).max(256));
            {
                let mut writer = CompressorWriter::new(&mut compressed, 4096, 4, 22);
                writer.write_all(&bytes).map_err(|e| {
                    error_json(
                        ApiErrorCode::Internal,
                        "brotli encoding failed",
                        json!({"message": e.to_string()}),
                    )
                })?;
            }
            Ok((compressed, Some("br")))
        }
        _ => Ok((bytes, None)),
    }
}

fn make_request_id(state: &AppState) -> String {
    let id = state.request_id_seed.fetch_add(1, Ordering::Relaxed);
    format!("req-{id:016x}")
}

fn propagated_request_id(headers: &HeaderMap, state: &AppState) -> String {
    if let Some(raw) = headers.get("x-request-id").and_then(|v| v.to_str().ok()) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(raw) = headers.get("traceparent").and_then(|v| v.to_str().ok()) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return format!("trace-{trimmed}");
        }
    }
    make_request_id(state)
}

fn with_request_id(mut response: Response, request_id: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", v);
    }
    response
}

pub(crate) async fn healthz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let resp = (StatusCode::OK, "ok").into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/healthz",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn version_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let payload = json!({
        "plugin": {
            "name": "bijux-atlas",
            "version": env!("CARGO_PKG_VERSION"),
            "compatible_umbrella": ">=0.1.0,<0.2.0",
            "build_hash": option_env!("BIJUX_BUILD_HASH").unwrap_or("dev"),
        },
        "server": {
            "crate": CRATE_NAME,
            "config_schema_version": crate::config::CONFIG_SCHEMA_VERSION,
        }
    });
    let mut response = Json(payload).into_response();
    if let Ok(value) = HeaderValue::from_str("public, max-age=30") {
        response.headers_mut().insert("cache-control", value);
    }
    state
        .metrics
        .observe_request("/v1/version", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn readyz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let catalog_ready = if state.api.readiness_requires_catalog {
        state.cache.current_catalog().await.is_some()
    } else {
        true
    };
    if state.ready.load(Ordering::Relaxed) && catalog_ready {
        let resp = (StatusCode::OK, "ready").into_response();
        state
            .metrics
            .observe_request_with_trace(
                "/readyz",
                StatusCode::OK,
                started.elapsed(),
                Some(&request_id),
            )
            .await;
        with_request_id(resp, &request_id)
    } else {
        let resp = (StatusCode::SERVICE_UNAVAILABLE, "not-ready").into_response();
        state
            .metrics
            .observe_request_with_trace(
                "/readyz",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
                Some(&request_id),
            )
            .await;
        with_request_id(resp, &request_id)
    }
}

pub(crate) async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    crate::telemetry::metrics_endpoint::metrics_handler(State(state)).await
}

pub(crate) async fn datasets_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
    info!(request_id = %request_id, route = "/v1/datasets", "request start");
    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| Catalog::new(vec![]));
    let payload = json!({"datasets": catalog.datasets});
    let etag = format!(
        "\"{}\"",
        sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(resp.headers_mut(), state.api.discovery_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/datasets", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let mut response = Json(payload).into_response();
    put_cache_headers(response.headers_mut(), state.api.discovery_ttl, &etag);
    state
        .metrics
        .observe_request("/v1/datasets", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn debug_datasets_handler(State(state): State<AppState>) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    if !state.api.enable_debug_datasets {
        let resp = api_error_response(
            StatusCode::NOT_FOUND,
            error_json(
                ApiErrorCode::InvalidQueryParameter,
                "debug endpoint disabled",
                json!({}),
            ),
        );
        state
            .metrics
            .observe_request("/debug/datasets", StatusCode::NOT_FOUND, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let items = state.cache.cached_datasets_debug().await;
    let resp = Json(json!({"datasets": items, "catalog_epoch": state.cache.catalog_epoch().await}))
        .into_response();
    state
        .metrics
        .observe_request("/debug/datasets", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

fn parse_fields(fields: Option<Vec<String>>) -> GeneFields {
    let mut out = GeneFields {
        gene_id: false,
        name: false,
        coords: false,
        biotype: false,
        transcript_count: false,
        sequence_length: false,
    };
    if let Some(list) = fields {
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
        let (seqid, span) = value
            .split_once(':')
            .ok_or_else(|| ApiError::invalid_param("region", &value))?;
        let (start, end) = span
            .split_once('-')
            .ok_or_else(|| ApiError::invalid_param("region", &value))?;
        let start = start
            .parse::<u64>()
            .map_err(|_| ApiError::invalid_param("region", &value))?;
        let end = end
            .parse::<u64>()
            .map_err(|_| ApiError::invalid_param("region", &value))?;
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
        error_json(
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
    let request_id = propagated_request_id(&headers, &state);
    info!(request_id = %request_id, "request start");
    if let Some(ip) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if !state
            .ip_limiter
            .allow(ip, &state.api.rate_limit_per_ip)
            .await
        {
            let resp = api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                error_json(
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
            return with_request_id(resp, &request_id);
        }
    }
    if state.api.enable_api_key_rate_limit {
        if let Some(key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
            if !state
                .api_key_limiter
                .allow(key, &state.api.rate_limit_per_api_key)
                .await
            {
                let resp = api_error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    error_json(
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
                return with_request_id(resp, &request_id);
            }
        }
    }

    let parse_map: std::collections::BTreeMap<String, String> =
        params.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let parsed = match parse_list_genes_params_with_limit(&parse_map, 100, state.limits.max_limit) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let dataset = match DatasetId::new(&parsed.release, &parsed.species, &parsed.assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                ApiError::invalid_param("dataset", &e.to_string()),
            );
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let req = match parse_region(parsed.region) {
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
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let class = classify_query(&req);
    if class == QueryClass::Heavy
        && state.api.shed_load_enabled
        && state
            .metrics
            .should_shed_heavy(
                state.api.shed_latency_min_samples,
                state.api.shed_latency_p95_threshold_ms,
            )
            .await
    {
        let resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
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
        return with_request_id(resp, &request_id);
    }
    let _class_permit = match acquire_class_permit(&state, class).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::TOO_MANY_REQUESTS, e);
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let _heavy_worker_permit = if class == QueryClass::Heavy {
        match state.heavy_workers.clone().try_acquire_owned() {
            Ok(permit) => Some(permit),
            Err(_) => {
                let resp = api_error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    error_json(
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
                return with_request_id(resp, &request_id);
            }
        }
    } else {
        None
    };

    let normalized = normalize_query(&params);
    let coalesce_key = format!(
        "{}|{}|{}|{}",
        dataset.canonical_string(),
        format!("{class:?}").to_lowercase(),
        normalized,
        wants_pretty(&params)
    );
    if class == QueryClass::Heavy {
        let cache = state.coalesced_cache.lock().await;
        if let Some(entry) = cache.get(&coalesce_key) {
            if entry.created_at.elapsed() <= state.api.query_coalesce_ttl {
                let mut resp = Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(entry.body.clone()))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                resp.headers_mut()
                    .insert("content-type", HeaderValue::from_static("application/json"));
                put_cache_headers(
                    resp.headers_mut(),
                    state.api.immutable_gene_ttl,
                    &entry.etag,
                );
                state
                    .metrics
                    .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
                    .await;
                return with_request_id(resp, &request_id);
            }
        }
        drop(cache);
    }
    let _coalesce_guard = if class == QueryClass::Heavy {
        let lock = {
            let mut inflight = state.coalesced_inflight.lock().await;
            Arc::clone(
                inflight
                    .entry(coalesce_key.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            )
        };
        Some(lock.lock_owned().await)
    } else {
        None
    };

    let stage_dataset_resolve_started = Instant::now();
    let work = async {
        info!(request_id = %request_id, dataset = ?dataset, "dataset resolve");
        let c = state.cache.open_dataset_connection(&dataset).await?;
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
            if let Some(gene_id) = req.filter.gene_id.as_ref() {
                if req.filter.name.is_none()
                    && req.filter.name_prefix.is_none()
                    && req.filter.biotype.is_none()
                    && req.filter.region.is_none()
                    && req.cursor.is_none()
                    && req.limit <= 1
                {
                    let row = query_gene_by_id_fast(&c.conn, gene_id, &req.fields)
                        .map_err(|e| CacheError(e.to_string()))?;
                    return Ok(bijux_atlas_query::GeneQueryResponse {
                        rows: row.into_iter().collect(),
                        next_cursor: None,
                    });
                }
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
                normalized_query = %normalize_query(&params),
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
                let resp = api_error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    error_json(
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
                return with_request_id(resp, &request_id);
            }
            if req.cursor.is_some() {
                let resp = api_error_response(
                    StatusCode::BAD_REQUEST,
                    error_json(
                        ApiErrorCode::InvalidCursor,
                        "invalid cursor",
                        json!({"message": msg}),
                    ),
                );
                state
                    .metrics
                    .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                    .await;
                return with_request_id(resp, &request_id);
            }
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
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
            return with_request_id(resp, &request_id);
        }
        Err(_) => {
            let resp = api_error_response(
                StatusCode::GATEWAY_TIMEOUT,
                error_json(ApiErrorCode::Timeout, "request timed out", json!({})),
            );
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::GATEWAY_TIMEOUT, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let serialize_started = Instant::now();
    let bytes = match info_span!("serialize_response").in_scope(|| {
        serialize_payload_with_capacity(
            &payload,
            wants_pretty(&params),
            state.api.response_max_bytes / 4,
        )
    }) {
        Ok(v) => v,
        Err(err) => {
            let resp = api_error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    state
        .metrics
        .observe_stage("serialize", serialize_started.elapsed())
        .await;
    if bytes.len() > state.api.response_max_bytes {
        let resp = api_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            error_json(
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
        return with_request_id(resp, &request_id);
    }

    let etag = format!(
        "\"{}\"",
        sha256_hex(format!("{normalized}|{}", String::from_utf8_lossy(&bytes)).as_bytes())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }

    let (response_bytes, content_encoding) = match maybe_compress_response(&headers, &state, bytes)
    {
        Ok(v) => v,
        Err(err) => {
            let resp = api_error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    if wants_text(&headers) {
        let text = String::from_utf8_lossy(&response_bytes).to_string();
        let mut resp = (StatusCode::OK, text).into_response();
        put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
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
    put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
    if class == QueryClass::Heavy {
        let mut cache = state.coalesced_cache.lock().await;
        cache.retain(|_, v| v.created_at.elapsed() <= state.api.query_coalesce_ttl);
        cache.insert(
            coalesce_key,
            CachedResponse {
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
    with_request_id(resp, &request_id)
}

pub(crate) async fn genes_count_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    let release = params.get("release").cloned().unwrap_or_default();
    let species = params.get("species").cloned().unwrap_or_default();
    let assembly = params.get("assembly").cloned().unwrap_or_default();
    let dataset = match DatasetId::new(&release, &species, &assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
            state
                .metrics
                .observe_request(
                    "/v1/genes/count",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    match state.cache.open_dataset_connection(&dataset).await {
        Ok(c) => {
            let count: Result<i64, _> =
                c.conn
                    .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0));
            match count {
                Ok(v) => {
                    let epoch = state.cache.catalog_epoch().await;
                    let resp = Json(json!({
                        "dataset": format!("{}/{}/{}", release, species, assembly),
                        "gene_count": v,
                        "catalog_epoch": epoch
                    }))
                    .into_response();
                    state
                        .metrics
                        .observe_request("/v1/genes/count", StatusCode::OK, started.elapsed())
                        .await;
                    with_request_id(resp, &request_id)
                }
                Err(e) => {
                    let resp = api_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_json(
                            ApiErrorCode::Internal,
                            "query failed",
                            json!({"message": e.to_string()}),
                        ),
                    );
                    state
                        .metrics
                        .observe_request(
                            "/v1/genes/count",
                            StatusCode::INTERNAL_SERVER_ERROR,
                            started.elapsed(),
                        )
                        .await;
                    with_request_id(resp, &request_id)
                }
            }
        }
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "dataset unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes/count",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            with_request_id(resp, &request_id)
        }
    }
}
