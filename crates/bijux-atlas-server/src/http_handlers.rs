use super::*;

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
            "config_schema_version": crate::api_config::CONFIG_SCHEMA_VERSION,
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
    crate::metrics_endpoint::metrics_handler(State(state)).await
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
        .unwrap_or(Catalog { datasets: vec![] });
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

    let normalized = normalize_query(&params);
    let work = async {
        info!(request_id = %request_id, dataset = ?dataset, "dataset resolve");
        let c = state.cache.open_dataset_connection(&dataset).await?;
        let deadline = Instant::now() + state.api.sql_timeout;
        c.conn
            .progress_handler(1_000, Some(move || Instant::now() > deadline));
        let query_started = Instant::now();
        let query_span = info_span!("sqlite_query", class = %format!("{class:?}").to_lowercase());
        let result = query_span.in_scope(|| {
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

    let bytes = info_span!("serialize_response").in_scope(|| {
        if wants_pretty(&params) {
            serde_json::to_vec_pretty(&payload).unwrap_or_default()
        } else {
            serde_json::to_vec(&payload).unwrap_or_default()
        }
    });
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

    if wants_text(&headers) {
        let text = String::from_utf8_lossy(&bytes).to_string();
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
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    resp.headers_mut()
        .insert("content-type", HeaderValue::from_static("application/json"));
    put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
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
