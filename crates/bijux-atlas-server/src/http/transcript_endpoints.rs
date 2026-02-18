pub(crate) async fn gene_transcripts_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(gene_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
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
    let _queue_guard = RequestQueueGuard {
        counter: Arc::clone(&state.queued_requests),
    };
    let release = params.get("release").cloned().unwrap_or_default();
    let species = params.get("species").cloned().unwrap_or_default();
    let assembly = params.get("assembly").cloned().unwrap_or_default();
    let dataset = match DatasetId::new(&release, &species, &assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::MissingDatasetDimension,
                    "missing dataset dimensions",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes/{gene_id}/transcripts",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50)
        .min(state.limits.max_transcript_limit);
    let filter = TranscriptFilter {
        parent_gene_id: Some(gene_id.clone()),
        biotype: params.get("biotype").cloned(),
        transcript_type: params.get("type").cloned(),
        region: parse_region_opt(params.get("region").cloned()),
    };
    let req = TranscriptQueryRequest {
        filter,
        limit,
        cursor: params.get("cursor").cloned(),
    };
    let class = QueryClass::Heavy;
    if crate::middleware::shedding::should_shed_noncheap(&state, class).await {
        let backoff = crate::middleware::shedding::heavy_backoff_ms(&state);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
        let mut resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server is shedding non-cheap query load",
                json!({"class":"heavy","retry_after_ms": backoff}),
            ),
        );
        if let Ok(v) = HeaderValue::from_str(&(backoff / 1000).max(1).to_string()) {
            resp.headers_mut().insert("retry-after", v);
        }
        state
            .metrics
            .observe_request(
                "/v1/genes/{gene_id}/transcripts",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let _class_permit = match state.class_heavy.clone().try_acquire_owned() {
        Ok(v) => v,
        Err(_) => {
            let resp = api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                error_json(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "transcript endpoint heavy bulkhead saturated",
                    json!({}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes/{gene_id}/transcripts",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let conn = match state.cache.open_dataset_connection(&dataset).await {
        Ok(c) => c,
        Err(e) => {
            let msg = e.to_string();
            let (status, code) = if msg.contains("quarantined") {
                (StatusCode::CONFLICT, ApiErrorCode::ArtifactQuarantined)
            } else if msg.contains("corrupt") {
                (StatusCode::CONFLICT, ApiErrorCode::ArtifactCorrupted)
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, ApiErrorCode::UpstreamStoreUnavailable)
            };
            let resp = api_error_response(
                status,
                error_json(
                    code,
                    "dataset unavailable",
                    json!({"message": msg}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes/{gene_id}/transcripts",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    match bijux_atlas_query::query_transcripts(&conn.conn, &req) {
        Ok(resp) => {
            let provenance = dataset_provenance(&state, &dataset).await;
            let body = Json(json_envelope(
                Some(json!(dataset)),
                Some(json!({ "next_cursor": resp.next_cursor.clone() })),
                json!({
                    "provenance": provenance,
                    "gene_id": gene_id,
                    "rows": resp.rows
                }),
                resp.next_cursor.map(|c| json!({ "next_cursor": c })),
                None,
            ))
            .into_response();
            state
                .metrics
                .observe_request(
                    "/v1/genes/{gene_id}/transcripts",
                    StatusCode::OK,
                    started.elapsed(),
                )
                .await;
            with_request_id(body, &request_id)
        }
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::InvalidQueryParameter,
                    "transcript query failed",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes/{gene_id}/transcripts",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            with_request_id(resp, &request_id)
        }
    }
}

pub(crate) async fn transcript_summary_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(tx_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
    let dataset = match DatasetId::new(
        params.get("release").map_or("", String::as_str),
        params.get("species").map_or("", String::as_str),
        params.get("assembly").map_or("", String::as_str),
    ) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::MissingDatasetDimension,
                    "missing dataset dimensions",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/transcripts/{tx_id}",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let class = QueryClass::Medium;
    if crate::middleware::shedding::should_shed_noncheap(&state, class).await {
        let resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server is shedding non-cheap query load",
                json!({"class":"medium"}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/transcripts/{tx_id}",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let _class_permit = match state.class_medium.clone().try_acquire_owned() {
        Ok(v) => v,
        Err(_) => {
            let resp = api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                error_json(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "transcript summary medium bulkhead saturated",
                    json!({}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/transcripts/{tx_id}",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let conn = match state.cache.open_dataset_connection(&dataset).await {
        Ok(c) => c,
        Err(e) => {
            let msg = e.to_string();
            let (status, code) = if msg.contains("quarantined") {
                (StatusCode::CONFLICT, ApiErrorCode::ArtifactQuarantined)
            } else if msg.contains("corrupt") {
                (StatusCode::CONFLICT, ApiErrorCode::ArtifactCorrupted)
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, ApiErrorCode::UpstreamStoreUnavailable)
            };
            let resp = api_error_response(
                status,
                error_json(
                    code,
                    "dataset unavailable",
                    json!({"message": msg}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/transcripts/{tx_id}",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    match bijux_atlas_query::query_transcript_by_id(&conn.conn, &tx_id) {
        Ok(Some(row)) => {
            let provenance = dataset_provenance(&state, &dataset).await;
            let body = Json(json_envelope(
                Some(json!(dataset)),
                None,
                json!({"provenance": provenance, "transcript": row}),
                None,
                None,
            ))
            .into_response();
            state
                .metrics
                .observe_request("/v1/transcripts/{tx_id}", StatusCode::OK, started.elapsed())
                .await;
            with_request_id(body, &request_id)
        }
        Ok(None) => {
            let resp = api_error_response(
                StatusCode::NOT_FOUND,
                error_json(
                    ApiErrorCode::InvalidQueryParameter,
                    "transcript not found",
                    json!({"transcript_id": tx_id}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/transcripts/{tx_id}",
                    StatusCode::NOT_FOUND,
                    started.elapsed(),
                )
                .await;
            with_request_id(resp, &request_id)
        }
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::InvalidQueryParameter,
                    "transcript query failed",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/transcripts/{tx_id}",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            with_request_id(resp, &request_id)
        }
    }
}
