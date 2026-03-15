use crate::app::server::cache::hot::HotEntry;
use crate::adapters::inbound::http::handlers;
use crate::*;
use serde_json::json;
use tracing::info_span;

pub(super) struct GenesResponseFinalizeContext<'a, R> {
    pub(super) state: &'a AppState,
    pub(super) headers: &'a HeaderMap,
    pub(super) params: &'a HashMap<String, String>,
    pub(super) payload: serde_json::Value,
    pub(super) started: Instant,
    pub(super) etag: &'a str,
    pub(super) class: QueryClass,
    pub(super) redis_cache_key: &'a Option<String>,
    pub(super) exact_gene_id: &'a Option<String>,
    pub(super) redis_fill_guard: Option<R>,
    pub(super) artifact_hash: &'a str,
    pub(super) cache_key_debug: &'a str,
    pub(super) coalesce_key: String,
    pub(super) request_id: &'a str,
}

pub(super) async fn finalize_genes_success_response<R>(
    ctx: GenesResponseFinalizeContext<'_, R>,
) -> Response {
    let GenesResponseFinalizeContext {
        state,
        headers,
        params,
        payload,
        started,
        etag,
        class,
        redis_cache_key,
        exact_gene_id,
        redis_fill_guard,
        artifact_hash,
        cache_key_debug,
        coalesce_key,
        request_id,
    } = ctx;
    if let Some(rows) = payload
        .get("data")
        .and_then(|d| d.get("rows"))
        .and_then(serde_json::Value::as_array)
    {
        state
            .metrics
            .observe_query_row_count("/v1/genes", rows.len())
            .await;
    }
    let serialize_started = Instant::now();
    let bytes = match info_span!("serialize_response").in_scope(|| {
        handlers::serialize_payload_with_capacity(
            &payload,
            handlers::wants_pretty(params),
            state.api.response_max_bytes / 4,
        )
    }) {
        Ok(v) => v,
        Err(err) => {
            let resp = handlers::api_error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    started.elapsed(),
                )
                .await;
            return handlers::with_request_id(resp, request_id);
        }
    };
    state
        .metrics
        .observe_stage("serialize", serialize_started.elapsed())
        .await;
    if bytes.len() > state.api.response_max_bytes {
        let resp = handlers::api_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            handlers::error_json(
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
        return handlers::with_request_id(resp, request_id);
    }
    if handlers::if_none_match(headers).as_deref() == Some(etag) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        handlers::put_cache_headers(
            resp.headers_mut(),
            state.api.immutable_gene_ttl,
            etag,
            handlers::CachePolicy::ImmutableDataset,
        );
        resp = handlers::with_query_class(resp, class);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return handlers::with_request_id(resp, request_id);
    }
    if state.api.enable_redis_response_cache {
        if let (Some(redis), Some(cache_key), Some(_)) =
            (&state.redis_backend, redis_cache_key, exact_gene_id)
        {
            if let Err(e) = redis
                .set_gene_cache(cache_key, &bytes, state.api.redis_response_cache_ttl_secs)
                .await
            {
                tracing::warn!("redis cache write fallback: {e}");
            }
        }
    }
    drop(redis_fill_guard);
    let (response_bytes, content_encoding) =
        match handlers::maybe_compress_response(headers, state, bytes) {
            Ok(v) => v,
            Err(err) => {
                let resp = handlers::api_error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        started.elapsed(),
                    )
                    .await;
                return handlers::with_request_id(resp, request_id);
            }
        };
    state
        .metrics
        .observe_response_size("/v1/genes", response_bytes.len())
        .await;
    if handlers::wants_text(headers) {
        let text = String::from_utf8_lossy(&response_bytes).to_string();
        let mut resp = (StatusCode::OK, text).into_response();
        handlers::put_cache_headers(
            resp.headers_mut(),
            state.api.immutable_gene_ttl,
            etag,
            handlers::CachePolicy::ImmutableDataset,
        );
        resp = handlers::with_query_class(resp, class);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
            .await;
        return handlers::with_request_id(resp, request_id);
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
    resp = handlers::with_query_class(resp, class);
    handlers::put_cache_headers(
        resp.headers_mut(),
        state.api.immutable_gene_ttl,
        etag,
        handlers::CachePolicy::ImmutableDataset,
    );
    handlers::cache_debug_headers(
        resp.headers_mut(),
        state.api.enable_debug_datasets,
        artifact_hash,
        cache_key_debug,
    );
    if class == QueryClass::Heavy || class == QueryClass::Cheap {
        let mut cache = state.hot_query_cache.lock().await;
        cache.insert(
            coalesce_key,
            HotEntry {
                body: response_bytes,
                etag: etag.to_string(),
                created_at: Instant::now(),
            },
        );
    }
    state
        .metrics
        .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
        .await;
    tracing::info!(request_id = %request_id, status = 200_u16, "request complete");
    handlers::with_request_id(resp, request_id)
}
