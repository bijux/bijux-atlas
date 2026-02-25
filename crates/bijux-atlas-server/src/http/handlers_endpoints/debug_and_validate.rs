pub(crate) async fn release_dataset_handler(
    State(state): State<AppState>,
    axum::extract::Path((release, species, assembly)): axum::extract::Path<(
        String,
        String,
        String,
    )>,
    uri: axum::extract::OriginalUri,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    let mut location = format!("/v1/datasets/{release}/{species}/{assembly}");
    if let Some(raw_query) = uri.0.query() {
        location.push('?');
        location.push_str(raw_query);
    }

    let mut resp = StatusCode::PERMANENT_REDIRECT.into_response();
    if let Ok(v) = HeaderValue::from_str(&location) {
        resp.headers_mut().insert("location", v);
    }
    if let Ok(v) = HeaderValue::from_str(&format!("<{location}>; rel=\"canonical\"")) {
        resp.headers_mut().insert("link", v);
    }
    resp.headers_mut()
        .insert("deprecation", HeaderValue::from_static("true"));

    state
        .metrics
        .observe_request(
            "/v1/releases/{release}/species/{species}/assemblies/{assembly}",
            StatusCode::PERMANENT_REDIRECT,
            started.elapsed(),
        )
        .await;
    with_request_id(resp, &request_id)
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
    let metrics = &state.cache.metrics;
    let resp = Json(json!({
        "datasets": items,
        "catalog_epoch": state.cache.catalog_epoch().await,
        "cache_stats": {
            "hit": metrics.dataset_hits.load(std::sync::atomic::Ordering::Relaxed),
            "miss": metrics.dataset_misses.load(std::sync::atomic::Ordering::Relaxed),
            "bytes_used": metrics.disk_usage_bytes.load(std::sync::atomic::Ordering::Relaxed),
            "evictions": metrics.cache_evictions_total.load(std::sync::atomic::Ordering::Relaxed),
            "download_failures": metrics.store_download_failures.load(std::sync::atomic::Ordering::Relaxed)
        }
    }))
    .into_response();
    state
        .metrics
        .observe_request("/debug/datasets", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn dataset_health_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
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
            .observe_request(
                "/debug/dataset-health",
                StatusCode::NOT_FOUND,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let release = params.get("release").cloned().unwrap_or_default();
    let species = params.get("species").cloned().unwrap_or_default();
    let assembly = params.get("assembly").cloned().unwrap_or_default();
    let dataset = match DatasetId::new(&release, &species, &assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::InvalidQueryParameter,
                    "invalid dataset dimensions",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/debug/dataset-health",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let snapshot = match state.cache.dataset_health_snapshot(&dataset).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "dataset health check failed",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/debug/dataset-health",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let resp = Json(json!({
        "dataset": dataset,
        "health": {
            "cached": snapshot.cached,
            "checksum_verified": snapshot.checksum_verified,
            "last_open_seconds_ago": snapshot.last_open_seconds_ago,
            "size_bytes": snapshot.size_bytes,
            "open_failures": snapshot.open_failures,
            "quarantined": snapshot.quarantined
        },
        "catalog_epoch": state.cache.catalog_epoch().await
    }))
    .into_response();
    state
        .metrics
        .observe_request("/debug/dataset-health", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn debug_echo_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let request_id = propagated_request_id(&headers, &state);
    if !state.api.enable_debug_datasets {
        let resp = api_error_response(
            StatusCode::NOT_FOUND,
            error_json(
                ApiErrorCode::InvalidQueryParameter,
                "debug endpoint disabled",
                json!({}),
            ),
        );
        return with_request_id(resp, &request_id);
    }
    let payload = json!({
        "dataset": Value::Null,
        "page": Value::Null,
        "data": {
            "query": params,
        },
        "links": Value::Null
    });
    with_request_id(Json(payload).into_response(), &request_id)
}

pub(crate) async fn query_validate_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(params): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
    let (dataset, req) =
        match crate::http::genes_support::build_dataset_query(&params, state.limits.max_limit) {
            Ok(v) => v,
            Err(e) => {
                let resp = api_error_response(StatusCode::BAD_REQUEST, e);
                state
                    .metrics
                    .observe_request_with_method(
                        "/v1/query/validate",
                        "POST",
                        StatusCode::BAD_REQUEST,
                        started.elapsed(),
                    )
                    .await;
                return with_request_id(resp, &request_id);
            }
        };
    let class = classify_query(&req);
    let cost = estimate_query_cost(&req);
    tracing::info!(
        request_id = %request_id,
        route = "/v1/query/validate",
        query_class = ?class,
        policy_mode = %state.runtime_policy_mode.as_str(),
        max_page_size = state.limits.max_limit,
        max_region_span = state.limits.max_region_span,
        "policy_applied"
    );
    let reasons = [
        req.filter.gene_id.as_ref().map(|_| "gene_id"),
        req.filter.name.as_ref().map(|_| "name"),
        req.filter.name_prefix.as_ref().map(|_| "name_like"),
        req.filter.biotype.as_ref().map(|_| "biotype"),
        req.filter.region.as_ref().map(|_| "range"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let data = json!({
        "dataset": dataset,
        "query_class": format!("{:?}", class).to_ascii_lowercase(),
        "work_units": cost.work_units,
        "limits": {
            "max_limit": state.limits.max_limit,
            "max_range_span": state.limits.max_region_span,
            "max_name_prefix_len": state.limits.max_prefix_len,
            "max_serialization_bytes": state.limits.max_serialization_bytes
        },
        "reasons": reasons
    });
    let payload = json_envelope(None, None, data, None, None);
    let resp = with_query_class(Json(payload).into_response(), class);
    state
        .metrics
        .observe_request_with_method(
            "/v1/query/validate",
            "POST",
            StatusCode::OK,
            started.elapsed(),
        )
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn registry_health_handler(State(state): State<AppState>) -> impl IntoResponse {
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
            .observe_request(
                "/debug/registry-health",
                StatusCode::NOT_FOUND,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let health = state.cache.registry_health().await;
    let resp = Json(json!({
        "registry_freeze_mode": state.cache.registry_freeze_mode(),
        "registry_ttl_seconds": state.cache.registry_ttl_seconds(),
        "sources": health,
        "catalog_epoch": state.cache.catalog_epoch().await
    }))
    .into_response();
    state
        .metrics
        .observe_request("/debug/registry-health", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

