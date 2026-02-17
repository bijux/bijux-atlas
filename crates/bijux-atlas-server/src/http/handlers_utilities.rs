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

pub(crate) fn normalized_forwarded_for(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("x-forwarded-for")?.to_str().ok()?;
    let first = raw.split(',').next()?.trim();
    if first.is_empty() || first.len() > 64 {
        return None;
    }
    if first
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b':' || b == b'-')
    {
        Some(first.to_string())
    } else {
        None
    }
}

pub(crate) fn normalized_api_key(headers: &HeaderMap) -> Option<String> {
    let key = headers.get("x-api-key")?.to_str().ok()?.trim();
    if key.is_empty() || key.len() > 256 {
        return None;
    }
    Some(key.to_string())
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
