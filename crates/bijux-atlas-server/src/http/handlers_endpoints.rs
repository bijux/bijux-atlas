pub(crate) async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    crate::telemetry::metrics_endpoint::metrics_handler(State(state)).await
}

pub(crate) async fn datasets_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
    if is_draining(&state) {
        let resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server draining; refusing new requests",
                json!({}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/datasets",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    info!(request_id = %request_id, route = "/v1/datasets", "request start");
    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| Catalog::new(vec![]));
    let include_bom = bool_query_flag(&params, "include_bom");
    let datasets_payload = if include_bom {
        let mut rows = Vec::with_capacity(catalog.datasets.len());
        for entry in &catalog.datasets {
            let bom = match state.cache.fetch_manifest_summary(&entry.dataset).await {
                Ok(manifest) => json!({
                    "manifest_version": manifest.manifest_version,
                    "db_schema_version": manifest.db_schema_version,
                    "checksums": manifest.checksums,
                    "stats": manifest.stats
                }),
                Err(_) => Value::Null,
            };
            rows.push(json!({
                "dataset": entry.dataset,
                "manifest_path": entry.manifest_path,
                "sqlite_path": entry.sqlite_path,
                "bill_of_materials": bom
            }));
        }
        Value::Array(rows)
    } else {
        serde_json::to_value(&catalog.datasets).unwrap_or(Value::Array(Vec::new()))
    };
    let payload = json_envelope(
        None,
        None,
        json!({ "datasets": datasets_payload }),
        None,
        None,
    );
    let etag = format!(
        "\"{}\"",
        sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(
            resp.headers_mut(),
            state.api.discovery_ttl,
            &etag,
            CachePolicy::CatalogDiscovery,
        );
        state
            .metrics
            .observe_request("/v1/datasets", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let mut response = Json(payload).into_response();
    put_cache_headers(
        response.headers_mut(),
        state.api.discovery_ttl,
        &etag,
        CachePolicy::CatalogDiscovery,
    );
    state
        .metrics
        .observe_request("/v1/datasets", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn release_dataset_handler(
    State(state): State<AppState>,
    axum::extract::Path((release, species, assembly)): axum::extract::Path<(
        String,
        String,
        String,
    )>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = make_request_id(&state);
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
                    "/v1/releases/{release}/species/{species}/assemblies/{assembly}",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| Catalog::new(vec![]));
    let entry = catalog
        .datasets
        .iter()
        .find(|e| e.dataset == dataset)
        .cloned();
    if entry.is_none() {
        let resp = api_error_response(
            StatusCode::NOT_FOUND,
            error_json(
                ApiErrorCode::DatasetNotFound,
                "dataset not found in catalog",
                json!({
                    "release": release,
                    "species": species,
                    "assembly": assembly
                }),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/releases/{release}/species/{species}/assemblies/{assembly}",
                StatusCode::NOT_FOUND,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let include_bom = bool_query_flag(&params, "include_bom");
    let manifest = match state.cache.fetch_manifest_summary(&dataset).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::UpstreamStoreUnavailable,
                    "dataset manifest unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/releases/{release}/species/{species}/assemblies/{assembly}",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let mut data = json!({
        "provenance": dataset_provenance(&state, &dataset).await,
        "catalog_entry": entry,
        "manifest_summary": {
            "manifest_version": manifest.manifest_version,
            "db_schema_version": manifest.db_schema_version,
            "stats": manifest.stats
        },
        "qc_summary": {
            "gene_count": manifest.stats.gene_count,
            "transcript_count": manifest.stats.transcript_count,
            "contig_count": manifest.stats.contig_count
        }
    });
    if include_bom {
        data["bill_of_materials"] = json!({
            "checksums": manifest.checksums,
            "manifest_version": manifest.manifest_version,
            "db_schema_version": manifest.db_schema_version
        });
    }
    let payload = json_envelope(Some(json!(dataset)), None, data, None, None);
    let resp = Json(payload).into_response();
    state
        .metrics
        .observe_request(
            "/v1/releases/{release}/species/{species}/assemblies/{assembly}",
            StatusCode::OK,
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
                    .observe_request(
                        "/v1/query/validate",
                        StatusCode::BAD_REQUEST,
                        started.elapsed(),
                    )
                    .await;
                return with_request_id(resp, &request_id);
            }
        };
    let class = classify_query(&req);
    let cost = estimate_query_cost(&req);
    let data = json!({
        "dataset": dataset,
        "query_class": format!("{:?}", class).to_ascii_lowercase(),
        "work_units": cost.work_units
    });
    let payload = json_envelope(None, None, data, None, None);
    let resp = with_query_class(Json(payload).into_response(), class);
    state
        .metrics
        .observe_request("/v1/query/validate", StatusCode::OK, started.elapsed())
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

pub(crate) async fn genes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    super::genes::genes_handler(State(state), headers, axum::extract::Query(params)).await
}

pub(crate) async fn genes_count_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    if is_draining(&state) {
        let resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server draining; refusing new requests",
                json!({}),
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
        return with_request_id(resp, &request_id);
    }
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

include!("transcript_endpoints.rs");
