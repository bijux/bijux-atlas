#![deny(clippy::redundant_clone)]

use crate::*;
use brotli::CompressorWriter;
use flate2::{write::GzEncoder, Compression};
use serde_json::json;
use serde_json::Value;
use std::io::Write;

struct RequestQueueGuard {
    counter: Arc<AtomicU64>,
}

impl Drop for RequestQueueGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) fn api_error_response(status: StatusCode, err: ApiError) -> Response {
    let body = Json(json!({"error": err}));
    (status, body).into_response()
}

pub(crate) fn error_json(code: ApiErrorCode, message: &str, details: Value) -> ApiError {
    ApiError {
        code,
        message: message.to_string(),
        details,
    }
}

pub(crate) fn normalize_query(params: &HashMap<String, String>) -> String {
    let mut kv: Vec<(&String, &String)> = params.iter().collect();
    kv.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(b.1)));
    kv.into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

pub(crate) fn if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get("if-none-match")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string)
}

pub(crate) fn put_cache_headers(headers: &mut HeaderMap, ttl: Duration, etag: &str) {
    if let Ok(value) = HeaderValue::from_str(&format!("public, max-age={}", ttl.as_secs())) {
        headers.insert("cache-control", value);
    }
    if let Ok(value) = HeaderValue::from_str(etag) {
        headers.insert("etag", value);
    }
}

pub(crate) fn wants_pretty(params: &HashMap<String, String>) -> bool {
    params
        .get("pretty")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

pub(crate) fn wants_min_viable_response(params: &HashMap<String, String>) -> bool {
    params
        .get("mvr")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

pub(crate) fn wants_text(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/plain"))
}

pub(crate) fn is_gene_id_exact_query(req: &GeneQueryRequest) -> Option<&str> {
    let gene_id = req.filter.gene_id.as_deref()?;
    if req.filter.name.is_none()
        && req.filter.name_prefix.is_none()
        && req.filter.biotype.is_none()
        && req.filter.region.is_none()
        && req.cursor.is_none()
        && req.limit <= 1
    {
        Some(gene_id)
    } else {
        None
    }
}

pub(crate) fn gene_fields_key(fields: &GeneFields) -> String {
    format!(
        "{}{}{}{}{}{}",
        fields.gene_id as u8,
        fields.name as u8,
        fields.coords as u8,
        fields.biotype as u8,
        fields.transcript_count as u8,
        fields.sequence_length as u8
    )
}

pub(crate) fn accepted_encoding(headers: &HeaderMap) -> Option<&'static str> {
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

pub(crate) fn serialize_payload_with_capacity(
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

pub(crate) fn maybe_compress_response(
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

pub(crate) fn make_request_id(state: &AppState) -> String {
    let id = state.request_id_seed.fetch_add(1, Ordering::Relaxed);
    format!("req-{id:016x}")
}

pub(crate) fn propagated_request_id(headers: &HeaderMap, state: &AppState) -> String {
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

pub(crate) fn with_request_id(mut response: Response, request_id: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", v);
    }
    response
}

pub(crate) async fn dataset_provenance(state: &AppState, dataset: &DatasetId) -> Value {
    let dataset_hash = sha256_hex(dataset.canonical_string().as_bytes());
    let mut out = json!({
        "dataset_hash": dataset_hash,
        "release": dataset.release,
        "species": dataset.species,
        "assembly": dataset.assembly
    });
    if let Ok(manifest) = state.cache.fetch_manifest_summary(dataset).await {
        out["manifest_version"] = json!(manifest.manifest_version);
        out["db_schema_version"] = json!(manifest.db_schema_version);
        out["dataset_signature_sha256"] = json!(manifest.dataset_signature_sha256);
    }
    out
}

fn is_draining(state: &AppState) -> bool {
    !state.accepting_requests.load(Ordering::Relaxed)
}

pub(crate) fn bool_query_flag(params: &HashMap<String, String>, name: &str) -> bool {
    params
        .get(name)
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn parse_region_opt(raw: Option<String>) -> Option<RegionFilter> {
    let value = raw?;
    let (seqid, span) = value.split_once(':')?;
    let (start, end) = span.split_once('-')?;
    Some(RegionFilter {
        seqid: seqid.to_string(),
        start: start.parse::<u64>().ok()?,
        end: end.parse::<u64>().ok()?,
    })
}

pub(crate) async fn landing_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| Catalog::new(vec![]));
    let mut list = String::new();
    for entry in &catalog.datasets {
        let ds = &entry.dataset;
        let canon = ds.canonical_string();
        list.push_str(&format!(
            "<li><code>{canon}</code> - <a href=\"/v1/genes/count?release={}&species={}&assembly={}\">genes/count</a></li>",
            ds.release, ds.species, ds.assembly
        ));
    }
    if list.is_empty() {
        list.push_str("<li>No datasets published yet.</li>");
    }
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Bijux Atlas</title></head><body>\
<h1>Bijux Atlas Dataset Browser</h1>\
<p>Version: <code>{}</code></p>\
<h2>Datasets</h2><ul>{}</ul>\
<h2>Example Queries</h2>\
<ul>\
<li><a href=\"/v1/datasets\">/v1/datasets</a></li>\
<li><a href=\"/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5\">/v1/genes?...&limit=5</a></li>\
<li><a href=\"/v1/diff/genes?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&limit=10\">/v1/diff/genes?...&limit=10</a></li>\
</ul>\
</body></html>",
        env!("CARGO_PKG_VERSION"),
        list
    );
    let mut resp = Response::new(Body::from(html));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        "content-type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    state
        .metrics
        .observe_request("/", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
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

pub(crate) async fn overload_health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let overloaded = crate::middleware::shedding::overloaded(&state).await;
    let status = if overloaded {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };
    let resp = (
        status,
        Json(json!({
            "overloaded": overloaded,
            "draining": !state.accepting_requests.load(Ordering::Relaxed),
            "cached_only_mode": state.cache.cached_only_mode(),
            "emergency_breaker": state.api.emergency_global_breaker
        })),
    )
        .into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/healthz/overload",
            status,
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
    let catalog_ready = if state.api.readiness_requires_catalog && !state.cache.cached_only_mode() {
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
    let payload = json!({"datasets": datasets_payload});
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
                ApiErrorCode::NotReady,
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
                    ApiErrorCode::NotReady,
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

    let mut payload = json!({
        "dataset": dataset,
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
        payload["bill_of_materials"] = json!({
            "checksums": manifest.checksums,
            "manifest_version": manifest.manifest_version,
            "db_schema_version": manifest.db_schema_version
        });
    }
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
    let resp = Json(json!({"datasets": items, "catalog_epoch": state.cache.catalog_epoch().await}))
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
            let body = Json(
                json!({"dataset": dataset, "provenance": provenance, "gene_id": gene_id, "response": resp}),
            )
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
            let body =
                Json(json!({"dataset": dataset, "provenance": provenance, "transcript": row}))
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
