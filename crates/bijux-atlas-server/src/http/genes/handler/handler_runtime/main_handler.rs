// SPDX-License-Identifier: Apache-2.0

pub(crate) async fn genes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = super::handlers::propagated_request_id(&headers, &state);
    if !state.accepting_requests.load(Ordering::Relaxed) {
        crate::record_shed_reason(&state, "draining").await;
        let resp = super::handlers::api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            super::handlers::error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server draining; refusing new requests",
                json!({}),
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
    info!(request_id = %request_id, "request start");
    let overloaded_early = crate::middleware::shedding::overloaded(&state).await;
    let adaptive_rl = super::genes_support::adaptive_rl_factor(&state, overloaded_early);
    if let Some(resp) = async {
        genes_admission::enforce_ip_rate_limit(&state, &headers, adaptive_rl, started, &request_id)
            .await
    }
    .instrument(info_span!("admission_control", route = "/v1/genes"))
    .await
    {
        crate::record_shed_reason(&state, "ip_rate_limited").await;
        return resp;
    }
    if let Some(resp) = async {
        genes_admission::enforce_api_key_rate_limit(
            &state,
            &headers,
            adaptive_rl,
            started,
            &request_id,
        )
        .await
    }
    .instrument(info_span!("admission_control", route = "/v1/genes"))
    .await
    {
        crate::record_shed_reason(&state, "api_key_rate_limited").await;
        return resp;
    }
    let (dataset, mut req) =
        match async { super::genes_support::build_dataset_query(&params, state.limits.max_limit) }
            .instrument(info_span!("dataset_resolve", route = "/v1/genes"))
            .await
        {
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
    let class = classify_query(&req);
    let estimated_cost = estimate_work_units(&req);
    info!(
        request_id = %request_id,
        route = "/v1/genes",
        query_class = ?class,
        policy_mode = %state.runtime_policy_mode.as_str(),
        max_page_size = state.limits.max_limit,
        max_region_span = state.limits.max_region_span,
        max_response_bytes = state.limits.max_serialization_bytes,
        "policy_applied"
    );
    if estimated_cost >= 50 {
        info!(
            request_id = %request_id,
            route = "/v1/genes",
            dataset = %dataset.canonical_string(),
            query_class = ?class,
            query_cost = estimated_cost,
            "query_cost_sample"
        );
    }
    let overloaded = state
        .metrics
        .should_shed_heavy(
            state.api.shed_latency_min_samples,
            state.api.shed_latency_p95_threshold_ms,
        )
        .await;
    super::genes_support::record_overload_cheap(&state, class, overloaded);
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
    if let Some(error) = super::genes_support::check_serialization_budget(&req, &state.limits) {
        let resp = super::handlers::api_error_response(StatusCode::UNPROCESSABLE_ENTITY, error);
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
    super::genes_support::cap_heavy_limit(&mut req, &state, class, overloaded);
    let (exact_gene_id, redis_cache_key) =
        super::genes_support::exact_lookup_cache_keys(&dataset, &req);
    if (class == QueryClass::Heavy && state.api.shed_load_enabled && overloaded)
        || crate::middleware::shedding::should_shed_noncheap(&state, class).await
    {
        crate::record_shed_reason(&state, "bulkhead_shed_heavy").await;
        let backoff = crate::middleware::shedding::heavy_backoff_ms(&state);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
        let mut resp = super::handlers::api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            super::handlers::error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server is shedding heavy query load",
                json!({"class":"heavy","retry_after_ms": backoff}),
            ),
        );
        if let Ok(v) = HeaderValue::from_str(&(backoff / 1000).max(1).to_string()) {
            resp.headers_mut().insert("retry-after", v);
        }
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
    let _queue_guard = match genes_admission::try_enter_request_queue(&state) {
        Ok(g) => g,
        Err(e) => {
            crate::record_shed_reason(&state, "queue_depth_exceeded").await;
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
    let _class_permit = match super::genes_support::acquire_class_permit(&state, class).await {
        Ok(v) => v,
        Err(e) => {
            crate::record_shed_reason(&state, "class_permit_saturated").await;
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
    let _heavy_worker_permit =
        match genes_admission::acquire_heavy_worker_permit(&state, class, started, &request_id)
            .await
        {
            Ok(permit) => permit,
            Err(resp) => {
                crate::record_shed_reason(&state, "heavy_worker_saturated").await;
                return resp;
            }
        };
    let normalized = super::handlers::normalize_query(&params);
    let manifest_summary = state.cache.fetch_manifest_summary(&dataset).await.ok();
    let artifact_hash = super::handlers::dataset_artifact_hash(manifest_summary.as_ref(), &dataset);
    let etag = super::handlers::dataset_etag(&artifact_hash, "/v1/genes", &params);
    let cache_key_debug = format!("/v1/genes?{normalized}");
    state
        .metrics
        .observe_request_size("/v1/genes", cache_key_debug.len())
        .await;
    let explain_mode = super::handlers::bool_query_flag(&params, "explain");
    let mut redis_fill_guard = None;
    if state.api.enable_redis_response_cache {
        if let (Some(redis), Some(cache_key)) = (&state.redis_backend, &redis_cache_key) {
            match redis.get_gene_cache(cache_key).await {
                Ok(Some(cached_bytes)) => {
                    if super::handlers::if_none_match(&headers).as_deref() == Some(etag.as_str()) {
                        let mut resp = StatusCode::NOT_MODIFIED.into_response();
                        super::handlers::put_cache_headers(
                            resp.headers_mut(),
                            state.api.immutable_gene_ttl,
                            &etag,
                            super::handlers::CachePolicy::ImmutableDataset,
                        );
                        resp = super::handlers::with_query_class(resp, class);
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
                        super::handlers::CachePolicy::ImmutableDataset,
                    );
                    if let Ok(v) = HeaderValue::from_str("redis-hit") {
                        resp.headers_mut().insert("x-atlas-cache", v);
                    }
                    resp = super::handlers::with_query_class(resp, class);
                    state
                        .metrics
                        .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
                        .await;
                    return super::handlers::with_request_id(resp, &request_id);
                }
                Ok(None) => {
                    let guard = redis.acquire_fill_lock(cache_key).await;
                    match redis.get_gene_cache(cache_key).await {
                        Ok(Some(cached_bytes)) => {
                            if super::handlers::if_none_match(&headers).as_deref()
                                == Some(etag.as_str())
                            {
                                let mut resp = StatusCode::NOT_MODIFIED.into_response();
                                super::handlers::put_cache_headers(
                                    resp.headers_mut(),
                                    state.api.immutable_gene_ttl,
                                    &etag,
                                    super::handlers::CachePolicy::ImmutableDataset,
                                );
                                resp = super::handlers::with_query_class(resp, class);
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
                                .unwrap_or_else(|_| {
                                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                                });
                            resp.headers_mut().insert(
                                "content-type",
                                HeaderValue::from_static("application/json"),
                            );
                            super::handlers::put_cache_headers(
                                resp.headers_mut(),
                                state.api.immutable_gene_ttl,
                                &etag,
                                super::handlers::CachePolicy::ImmutableDataset,
                            );
                            if let Ok(v) = HeaderValue::from_str("redis-hit") {
                                resp.headers_mut().insert("x-atlas-cache", v);
                            }
                            resp = super::handlers::with_query_class(resp, class);
                            state
                                .metrics
                                .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
                                .await;
                            return super::handlers::with_request_id(resp, &request_id);
                        }
                        Ok(None) => {
                            redis_fill_guard = Some(guard);
                        }
                        Err(e) => {
                            warn!("redis cache read fallback after fill-lock: {e}");
                            redis_fill_guard = Some(guard);
                        }
                    }
                }
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
                super::handlers::CachePolicy::ImmutableDataset,
            );
            resp = super::handlers::with_query_class(resp, class);
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
        let shard_candidates = if req.filter.region.is_some() {
            Some(
                state
                    .cache
                    .selected_shards_for_region(
                        &dataset,
                        req.filter.region.as_ref().map(|r| r.seqid.as_str()),
                    )
                    .await?,
            )
        } else {
            None
        };
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
            if req.filter.region.is_some() {
                let catalog_path = bijux_atlas_model::artifact_paths(
                    state.cache.disk_root(),
                    &dataset,
                )
                .derived_dir
                .join("catalog_shards.json");
                if catalog_path.exists() {
                    let raw = crate::http::effects_adapters::read_bytes(&catalog_path)?;
                    if let Ok(catalog) = serde_json::from_slice::<ShardCatalog>(&raw) {
                        let selected_rel = select_shards_for_request(&req, &catalog);
                        let selected_all = shard_candidates.clone().unwrap_or_default();
                        let selected: Vec<std::path::PathBuf> = selected_all
                            .into_iter()
                            .filter(|p| {
                                selected_rel.iter().any(|r| p.ends_with(r))
                            })
                            .collect();
                        if !selected.is_empty() {
                            let mut shard_conns = Vec::new();
                            let mut permits = Vec::new();
                            for shard in selected
                                .into_iter()
                                .take(state.cache.max_open_shards_per_pod())
                            {
                                permits.push(state.cache.try_acquire_shard_permit()?);
                                let conn = crate::effect_adapters::sqlite_adapters::open_readonly_no_mutex(&shard)?;
                                let (cache_kib, shard_mmap) =
                                    state.cache.sqlite_pragmas_for_shard_open();
                                let _ = crate::effect_adapters::sqlite_adapters::apply_readonly_pragmas(
                                    &conn,
                                    cache_kib,
                                    shard_mmap,
                                );
                                shard_conns.push(conn);
                            }
                            let refs: Vec<&Connection> = shard_conns.iter().collect();
                            let response = query_genes_fanout(
                                &refs,
                                &req,
                                &state.limits,
                                b"atlas-server-cursor-secret",
                            )
                            .map_err(|e| CacheError(e.to_string()))?;
                            drop(permits);
                            return Ok(response);
                        }
                    }
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
            let provenance = super::handlers::dataset_provenance(&state, &dataset).await;
            genes_response::build_success_payload(
                &dataset,
                &req,
                class,
                resp,
                explain_mode,
                provenance,
            )
        }
        Ok(Err(err)) => {
            let msg = err.to_string();
            if msg.contains("limit")
                || msg.contains("span")
                || msg.contains("scan")
                || msg.contains("name_prefix")
            {
                let (code, reason_code) = if msg.contains("region span exceeds") {
                    (ApiErrorCode::RangeTooLarge, "RANGE_TOO_LARGE")
                } else if msg.contains("estimated query cost") {
                    (ApiErrorCode::QueryTooExpensive, "QUERY_TOO_EXPENSIVE")
                } else {
                    (ApiErrorCode::QueryRejectedByPolicy, "QUERY_REJECTED")
                };
                let status = if matches!(
                    code,
                    ApiErrorCode::RangeTooLarge | ApiErrorCode::QueryTooExpensive
                ) {
                    StatusCode::PAYLOAD_TOO_LARGE
                } else {
                    StatusCode::UNPROCESSABLE_ENTITY
                };
                let resp = super::handlers::api_error_response(
                    status,
                    super::handlers::error_json(
                        code,
                        "query rejected",
                        json!({"message": msg, "reason_code": reason_code}),
                    ),
                );
                state
                    .metrics
                    .observe_request("/v1/genes", status, started.elapsed())
                    .await;
                return super::handlers::with_request_id(resp, &request_id);
            }
            if msg.contains("Validation:") || msg.contains("seqid does not exist in dataset") {
                let resp = super::handlers::api_error_response(
                    StatusCode::BAD_REQUEST,
                    super::handlers::error_json(
                        ApiErrorCode::InvalidQueryParameter,
                        "invalid query parameter",
                        json!({"message": msg}),
                    ),
                );
                state
                    .metrics
                    .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                    .await;
                return super::handlers::with_request_id(resp, &request_id);
            }
            if req.cursor.is_some() {
                let reason_code = if msg.contains("UnsupportedVersion") {
                    "CURSOR_VERSION_UNSUPPORTED"
                } else if msg.contains("DatasetMismatch") {
                    "CURSOR_DATASET_MISMATCH"
                } else {
                    "CURSOR_INVALID"
                };
                let resp = super::handlers::api_error_response(
                    StatusCode::BAD_REQUEST,
                    super::handlers::error_json(
                        ApiErrorCode::InvalidCursor,
                        "invalid cursor",
                        json!({"message": msg, "reason_code": reason_code}),
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
            if state.api.continue_download_on_request_timeout_for_warmup
                && state.cache.is_pinned_dataset(&dataset)
            {
                let cache = state.cache.clone();
                let ds = dataset.clone();
                tokio::spawn(async move {
                    let _ = cache.prefetch_dataset(ds).await;
                });
            }
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
    finalize_genes_success_response(
        &state,
        &headers,
        &params,
        payload,
        started,
        &etag,
        class,
        &redis_cache_key,
        &exact_gene_id,
        redis_fill_guard,
        &artifact_hash,
        &cache_key_debug,
        coalesce_key,
        &request_id,
    )
    .await
}
