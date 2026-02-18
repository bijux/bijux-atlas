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
    let release_filter = params.get("release").map(std::string::String::as_str);
    let species_filter = params.get("species").map(std::string::String::as_str);
    let assembly_filter = params.get("assembly").map(std::string::String::as_str);
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .map_or(100, |v| v.clamp(1, 500));
    let cursor = params.get("cursor").cloned();

    let mut rows = catalog
        .datasets
        .iter()
        .filter(|entry| {
            release_filter.is_none_or(|v| entry.dataset.release.as_str() == v)
                && species_filter.is_none_or(|v| entry.dataset.species.as_str() == v)
                && assembly_filter.is_none_or(|v| entry.dataset.assembly.as_str() == v)
        })
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.dataset.canonical_string().cmp(&b.dataset.canonical_string()));

    let start_idx = cursor
        .as_deref()
        .and_then(|c| rows.iter().position(|r| r.dataset.canonical_string() == c))
        .map_or(0, |idx| idx.saturating_add(1));
    let page = rows.into_iter().skip(start_idx).take(limit + 1).collect::<Vec<_>>();
    let has_more = page.len() > limit;
    let page = if has_more {
        page[..limit].to_vec()
    } else {
        page
    };
    let next_cursor = if has_more {
        page.last().map(|entry| entry.dataset.canonical_string())
    } else {
        None
    };

    let datasets_payload = if include_bom {
        let mut with_bom = Vec::with_capacity(page.len());
        for entry in &page {
            let bom = match state.cache.fetch_manifest_summary(&entry.dataset).await {
                Ok(manifest) => json!({
                    "manifest_version": manifest.manifest_version,
                    "db_schema_version": manifest.db_schema_version,
                    "checksums": manifest.checksums,
                    "stats": manifest.stats
                }),
                Err(_) => Value::Null,
            };
            with_bom.push(json!({
                "dataset": entry.dataset,
                "manifest_path": entry.manifest_path,
                "sqlite_path": entry.sqlite_path,
                "bill_of_materials": bom
            }));
        }
        Value::Array(with_bom)
    } else {
        serde_json::to_value(&page).unwrap_or(Value::Array(Vec::new()))
    };
    let payload = json_envelope(
        None,
        Some(json!({"next_cursor": next_cursor})),
        json!({
            "items": datasets_payload,
            "stats": {
                "limit": limit,
                "returned": page.len()
            }
        }),
        next_cursor.map(|cursor| json!({"next_cursor": cursor})),
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

pub(crate) async fn dataset_identity_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path((release, species, assembly)): axum::extract::Path<(
        String,
        String,
        String,
    )>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
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
                    "/v1/datasets/{release}/{species}/{assembly}",
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
                "/v1/datasets/{release}/{species}/{assembly}",
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
                    "/v1/datasets/{release}/{species}/{assembly}",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let endpoints = json!([
        "/v1/genes",
        "/v1/genes/count",
        "/v1/genes/{gene_id}/transcripts",
        "/v1/genes/{gene_id}/sequence",
        "/v1/transcripts/{tx_id}",
        "/v1/sequence/region"
    ]);
    let query = format!(
        "release={}&species={}&assembly={}",
        dataset.release.as_str(),
        dataset.species.as_str(),
        dataset.assembly.as_str()
    );
    let item = json!({
        "dataset": dataset,
        "artifact_hash": manifest.artifact_hash,
        "artifact_db_hash": manifest.db_hash,
        "checksums": {
            "sqlite_sha256": manifest.checksums.sqlite_sha256
        },
        "shard_info": {
            "plan": manifest.sharding_plan,
            "router": manifest.sharding_plan != bijux_atlas_model::ShardingPlan::None
        },
        "metadata": {
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
        },
        "available_endpoints": endpoints,
        "links": {
            "genes": format!("/v1/genes?{query}"),
            "genes_count": format!("/v1/genes/count?{query}"),
            "sequence_region": format!("/v1/sequence/region?{query}&region=chr1:1-10")
        }
    });
    let mut data = json!({ "item": item });
    if include_bom {
        data["item"]["bill_of_materials"] = json!({
            "checksums": manifest.checksums,
            "manifest_version": manifest.manifest_version,
            "db_schema_version": manifest.db_schema_version
        });
    }
    let payload = json_envelope(Some(json!(dataset)), None, data, None, None);
    let etag = format!(
        "\"{}\"",
        sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(
            resp.headers_mut(),
            state.api.immutable_gene_ttl,
            &etag,
            CachePolicy::ImmutableDataset,
        );
        state
            .metrics
            .observe_request(
                "/v1/datasets/{release}/{species}/{assembly}",
                StatusCode::NOT_MODIFIED,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let mut resp = Json(payload).into_response();
    put_cache_headers(
        resp.headers_mut(),
        state.api.immutable_gene_ttl,
        &etag,
        CachePolicy::ImmutableDataset,
    );
    state
        .metrics
        .observe_request(
            "/v1/datasets/{release}/{species}/{assembly}",
            StatusCode::OK,
            started.elapsed(),
        )
        .await;
    with_request_id(resp, &request_id)
}

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
    let (dataset, req) = match crate::http::genes_support::build_dataset_query(&params, 500) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
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
            let count = query_gene_count_with_filters(&c.conn, &req);
            match count {
                Ok(v) => {
                    let epoch = state.cache.catalog_epoch().await;
                    let resp = Json(json!({
                        "dataset": dataset.canonical_string(),
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

fn query_gene_count_with_filters(
    conn: &rusqlite::Connection,
    req: &bijux_atlas_query::GeneQueryRequest,
) -> Result<i64, rusqlite::Error> {
    let mut sql = "SELECT COUNT(*) FROM gene_summary g".to_string();
    let mut where_parts: Vec<String> = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(region) = &req.filter.region {
        sql.push_str(" JOIN gene_summary_rtree r ON r.gene_rowid = g.id");
        where_parts.push("g.seqid = ?".to_string());
        params.push(rusqlite::types::Value::Text(region.seqid.clone()));
        where_parts.push("r.start <= ?".to_string());
        params.push(rusqlite::types::Value::Real(region.end as f64));
        where_parts.push("r.end >= ?".to_string());
        params.push(rusqlite::types::Value::Real(region.start as f64));
    }
    if let Some(gene_id) = &req.filter.gene_id {
        where_parts.push("g.gene_id = ?".to_string());
        params.push(rusqlite::types::Value::Text(gene_id.clone()));
    }
    if let Some(name) = &req.filter.name {
        where_parts.push("g.name_normalized = ?".to_string());
        params.push(rusqlite::types::Value::Text(
            bijux_atlas_query::normalize_name_lookup(name),
        ));
    }
    if let Some(prefix) = &req.filter.name_prefix {
        where_parts.push("g.name_normalized LIKE ? ESCAPE '!'".to_string());
        params.push(rusqlite::types::Value::Text(format!(
            "{}%",
            bijux_atlas_query::escape_like_prefix(&bijux_atlas_query::normalize_name_lookup(prefix))
        )));
    }
    if let Some(biotype) = &req.filter.biotype {
        where_parts.push("g.biotype = ?".to_string());
        params.push(rusqlite::types::Value::Text(biotype.clone()));
    }
    if !where_parts.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_parts.join(" AND "));
    }

    conn.query_row(&sql, rusqlite::params_from_iter(params.iter()), |row| {
        row.get::<_, i64>(0)
    })
}
