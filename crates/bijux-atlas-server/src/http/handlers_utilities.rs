// SPDX-License-Identifier: Apache-2.0

use crate::*;
use brotli::CompressorWriter;
use flate2::{write::GzEncoder, Compression};
use serde_json::json;
use serde_json::Value;
use std::io::Write;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ClusterRegisterRequest {
    pub cluster_id: String,
    pub node_id: String,
    pub generation: u64,
    pub role: String,
    pub advertise_addr: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ClusterHeartbeatRequest {
    pub cluster_id: String,
    pub node_id: String,
    pub generation: u64,
    pub load_percent: u8,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ClusterModeRequest {
    pub node_id: String,
    pub mode: String,
    #[serde(default)]
    pub generation: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ClusterReplicaFailoverRequest {
    pub dataset_id: String,
    pub shard_id: String,
    pub promote_node_id: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct FailureInjectionRequest {
    pub kind: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub shard_id: Option<String>,
}

struct RequestQueueGuard {
    counter: Arc<AtomicU64>,
}

impl Drop for RequestQueueGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) use crate::http::response_contract::api_error_response;
pub(crate) use crate::http::response_contract::api_error as error_json;

pub(crate) fn json_envelope(
    dataset: Option<Value>,
    page: Option<Value>,
    data: Value,
    links: Option<Value>,
    warnings: Option<Vec<Value>>,
) -> Value {
    let mut root = json!({
        "api_version": "v1",
        "contract_version": "v1",
        "dataset": dataset.unwrap_or(Value::Null),
        "page": page.unwrap_or(Value::Null),
        "data": data,
        "links": links.unwrap_or(Value::Null)
    });
    if let Some(warnings) = warnings {
        if !warnings.is_empty() {
            root["meta"] = json!({ "warnings": warnings });
        }
    }
    root
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

pub(crate) enum CachePolicy {
    ImmutableDataset,
    CatalogDiscovery,
}

pub(crate) fn put_cache_headers(
    headers: &mut HeaderMap,
    ttl: Duration,
    etag: &str,
    policy: CachePolicy,
) {
    let stale = (ttl.as_secs() / 2).max(1);
    let cache_control = match policy {
        CachePolicy::ImmutableDataset => {
            format!(
                "public, max-age={}, stale-while-revalidate={}, immutable",
                ttl.as_secs(),
                stale
            )
        }
        CachePolicy::CatalogDiscovery => format!(
            "public, max-age={}, stale-while-revalidate={}",
            ttl.as_secs(),
            stale
        ),
    };
    if let Ok(value) = HeaderValue::from_str(&cache_control) {
        headers.insert("cache-control", value);
    }
    if let Ok(value) = HeaderValue::from_str(etag) {
        headers.insert("etag", value);
    }
    headers.insert("vary", HeaderValue::from_static("accept-encoding"));
}

pub(crate) fn dataset_artifact_hash(
    manifest: Option<&ArtifactManifest>,
    dataset: &DatasetId,
) -> String {
    if let Some(summary) = manifest {
        if !summary.dataset_signature_sha256.trim().is_empty() {
            return summary.dataset_signature_sha256.clone();
        }
    }
    sha256_hex(dataset.canonical_string().as_bytes())
}

pub(crate) fn dataset_etag(
    artifact_hash: &str,
    path: &str,
    params: &HashMap<String, String>,
) -> String {
    let normalized = normalize_query(params);
    format!(
        "\"{}\"",
        sha256_hex(format!("{artifact_hash}|{path}|{normalized}").as_bytes())
    )
}

pub(crate) fn cache_debug_headers(
    headers: &mut HeaderMap,
    enabled: bool,
    artifact_hash: &str,
    normalized_request: &str,
) {
    if !enabled {
        return;
    }
    if let Ok(v) = HeaderValue::from_str(artifact_hash) {
        headers.insert("x-atlas-artifact-hash", v);
    }
    if let Ok(v) = HeaderValue::from_str(normalized_request) {
        headers.insert("x-atlas-cache-key", v);
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
    crate::http::request_tracing::extract_request_trace(headers, state).request_id
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
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-trace-id", v);
    }
    let should_set_json = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_none_or(|v| v.eq_ignore_ascii_case("application/json"));
    if should_set_json
        && !matches!(
            response.status(),
            StatusCode::NO_CONTENT | StatusCode::NOT_MODIFIED
        )
    {
        response.headers_mut().insert(
            "content-type",
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
    }
    response
}

pub(crate) fn with_query_class(mut response: Response, class: QueryClass) -> Response {
    let value = match class {
        QueryClass::Cheap => "cheap",
        QueryClass::Medium => "medium",
        QueryClass::Heavy => "heavy",
        _ => "heavy",
    };
    response
        .headers_mut()
        .insert("x-atlas-query-class", HeaderValue::from_static(value));
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

pub(crate) async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let resp = (StatusCode::OK, "ok").into_response();
    state
        .metrics
        .observe_request_with_trace("/health", StatusCode::OK, started.elapsed(), Some(&request_id))
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
        "api_version": "v1",
        "contract_version": "v1",
        "plugin": {
            "name": "bijux-atlas",
            "version": env!("CARGO_PKG_VERSION"),
            "compatible_umbrella": ">=0.1.0,<0.2.0",
            "build_hash": option_env!("BIJUX_BUILD_HASH").unwrap_or("dev"),
        },
        "server": {
            "crate": CRATE_NAME,
            "config_schema_version": crate::config::CONFIG_SCHEMA_VERSION,
            "api_version": "v1",
            "api_contract_version": "v1",
            "runtime_policy_hash": &*state.runtime_policy_hash,
            "artifact_schema_versions": {
                "manifest_schema_version": "1",
                "sqlite_schema_version": "4"
            }
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

pub(crate) async fn cluster_status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let cluster_path =
        std::env::var("ATLAS_CLUSTER_CONFIG_PATH").unwrap_or_else(|_| {
            "configs/ops/runtime/cluster-config.example.json".to_string()
        });
    let node_path =
        std::env::var("ATLAS_NODE_CONFIG_PATH").unwrap_or_else(|_| {
            "configs/ops/runtime/node-config.example.json".to_string()
        });

    let mut response_status = StatusCode::OK;
    let payload = match (
        bijux_atlas_core::load_cluster_config_from_path(std::path::Path::new(&cluster_path)),
        bijux_atlas_core::load_node_config_from_path(std::path::Path::new(&node_path)),
    ) {
        (Ok(cluster_cfg), Ok(node_cfg)) => {
            let cluster = cluster_cfg.to_descriptor();
            let node = node_cfg.to_descriptor();
            let mut registry = bijux_atlas_core::ClusterStateRegistry::new(cluster.clone());
            registry.register_node(bijux_atlas_core::NodeMetadata {
                descriptor: node,
                state: bijux_atlas_core::NodeState::Ready,
                last_heartbeat_unix_ms: chrono_like_unix_millis() as u64,
            });
            let snapshot = registry.snapshot();
            let membership = state.membership.lock().await;
            let membership_metrics = membership.metrics();
            json!({
                "cluster_id": cluster.cluster_id,
                "topology_mode": cluster.topology_mode,
                "discovery_strategy": cluster.discovery_strategy,
                "seed_nodes": cluster.seed_nodes,
                "metadata_store": cluster.metadata_store,
                "health": snapshot.health,
                "topology_version": snapshot.topology_version,
                "node_count": snapshot.node_count,
                "membership": {
                    "total_nodes": membership_metrics.total_nodes,
                    "active_nodes": membership_metrics.active_nodes,
                    "timed_out_nodes": membership_metrics.timed_out_nodes,
                    "average_load_percent": membership_metrics.average_load_percent
                }
            })
        }
        (cluster_result, node_result) => {
            response_status = StatusCode::SERVICE_UNAVAILABLE;
            json!({
                "cluster_id": Value::Null,
                "health": "unavailable",
                "error": {
                    "cluster_config": cluster_result.err().unwrap_or_default(),
                    "node_config": node_result.err().unwrap_or_default()
                }
            })
        }
    };

    let mut response = Json(payload).into_response();
    *response.status_mut() = response_status;
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster-status",
            response_status,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_nodes_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let now_unix_ms = chrono_like_unix_millis() as u64;
    let mut membership = state.membership.lock().await;
    let _timed_out = membership.detect_timeouts(now_unix_ms);
    let nodes = membership
        .nodes()
        .into_iter()
        .map(|node| {
            json!({
                "node_id": node.descriptor.identity.node_id,
                "cluster_id": node.descriptor.identity.cluster_id,
                "generation": node.descriptor.identity.generation,
                "state": node.state,
                "role": node.descriptor.role,
                "load_percent": node.load_percent,
                "last_heartbeat_unix_ms": node.last_heartbeat_unix_ms,
                "is_live": membership.node_is_live(&node.descriptor.identity.node_id, now_unix_ms),
                "capabilities": node.descriptor.capabilities
            })
        })
        .collect::<Vec<_>>();
    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_node_status_report",
        "nodes": nodes,
        "metrics": membership.metrics()
    });
    tracing::info!(
        event_id = "cluster_membership_nodes_view",
        route = "/debug/cluster/nodes",
        node_count = payload["nodes"].as_array().map_or(0, |rows| rows.len()),
        "cluster membership node status snapshot"
    );
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/nodes",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_register_handler(
    State(state): State<AppState>,
    Json(req): Json<ClusterRegisterRequest>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let role = match req.role.as_str() {
        "ingest" => bijux_atlas_core::NodeRole::Ingest,
        "query" => bijux_atlas_core::NodeRole::Query,
        _ => bijux_atlas_core::NodeRole::Hybrid,
    };
    let descriptor = bijux_atlas_core::NodeDescriptor {
        identity: bijux_atlas_core::NodeIdentity {
            cluster_id: req.cluster_id,
            node_id: req.node_id.clone(),
            generation: req.generation.max(1),
        },
        role,
        advertise_addr: req.advertise_addr,
        capabilities: if req.capabilities.is_empty() {
            vec!["query.execute".to_string()]
        } else {
            req.capabilities
        },
        readiness: bijux_atlas_core::ReadinessPolicy {
            require_membership: true,
            require_dataset_registry: true,
            require_health_probes: true,
        },
        shutdown: bijux_atlas_core::ShutdownPolicy {
            drain_timeout_ms: 10_000,
            publish_exit_state: true,
        },
    };

    let now_unix_ms = chrono_like_unix_millis() as u64;
    let mut membership = state.membership.lock().await;
    membership.join_node(descriptor, now_unix_ms);
    membership.activate_node(&req.node_id);
    tracing::info!(
        event_id = "cluster_membership_register",
        route = "/debug/cluster/register",
        node_id = %req.node_id,
        generation = req.generation,
        "cluster membership node registered"
    );

    let response = Json(json!({
        "schema_version": 1,
        "kind": "cluster_node_register_result",
        "node_id": req.node_id,
        "status": "registered"
    }))
    .into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/register",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_heartbeat_handler(
    State(state): State<AppState>,
    Json(req): Json<ClusterHeartbeatRequest>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let mut membership = state.membership.lock().await;
    membership.apply_heartbeat(bijux_atlas_core::HeartbeatMessage {
        identity: bijux_atlas_core::NodeIdentity {
            cluster_id: req.cluster_id,
            node_id: req.node_id.clone(),
            generation: req.generation.max(1),
        },
        sent_at_unix_ms: chrono_like_unix_millis() as u64,
        load_percent: req.load_percent.min(100),
    });
    tracing::info!(
        event_id = "cluster_membership_heartbeat",
        route = "/debug/cluster/heartbeat",
        node_id = %req.node_id,
        generation = req.generation,
        load_percent = req.load_percent,
        "cluster membership heartbeat accepted"
    );
    let response = Json(json!({
        "schema_version": 1,
        "kind": "cluster_node_heartbeat_result",
        "status": "accepted"
    }))
    .into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/heartbeat",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_mode_handler(
    State(state): State<AppState>,
    Json(req): Json<ClusterModeRequest>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let now_unix_ms = chrono_like_unix_millis() as u64;
    let mut membership = state.membership.lock().await;
    match req.mode.as_str() {
        "quarantine" => membership.set_quarantine(&req.node_id),
        "maintenance" => membership.set_maintenance(&req.node_id),
        "drain" => membership.set_draining(&req.node_id),
        "restart" => membership.handle_restart(
            &req.node_id,
            req.generation.unwrap_or(1).max(1),
            now_unix_ms,
        ),
        "recover" => membership.recover_node(&req.node_id, now_unix_ms),
        "remove" => membership.remove_node(&req.node_id),
        _ => {}
    }
    tracing::info!(
        event_id = "cluster_membership_mode_change",
        route = "/debug/cluster/mode",
        node_id = %req.node_id,
        mode = %req.mode,
        "cluster membership mode change applied"
    );
    let response = Json(json!({
        "schema_version": 1,
        "kind": "cluster_node_mode_result",
        "node_id": req.node_id,
        "mode": req.mode
    }))
    .into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/mode",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_replica_list_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let registry = state.replica_registry.lock().await;
    let replicas = registry
        .list()
        .into_iter()
        .map(|replica| {
            json!({
                "dataset_id": replica.metadata.dataset_id,
                "shard_id": replica.metadata.shard_id,
                "primary_node_id": replica.metadata.primary_node_id,
                "replica_node_ids": replica.metadata.replica_node_ids,
                "lag_ms": replica.sync.lag_ms,
                "sync_throughput_rows_per_second": replica.sync.sync_throughput_rows_per_second,
                "healthy": replica.health.healthy,
            })
        })
        .collect::<Vec<_>>();
    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_replica_list_report",
        "replicas": replicas,
        "consistency": registry.consistency(),
        "policy": registry.policy()
    });
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/replicas",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_replica_health_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let registry = state.replica_registry.lock().await;
    let metrics = registry.metrics();
    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_replica_health_report",
        "metrics": metrics,
        "replicas": registry.list().into_iter().map(|replica| {
            json!({
                "dataset_id": replica.metadata.dataset_id,
                "shard_id": replica.metadata.shard_id,
                "healthy": replica.health.healthy,
                "failed_checks": replica.health.failed_checks,
                "last_failure_reason": replica.health.last_failure_reason,
            })
        }).collect::<Vec<_>>()
    });
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/replicas/health",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_replica_failover_handler(
    State(state): State<AppState>,
    Json(req): Json<ClusterReplicaFailoverRequest>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let mut registry = state.replica_registry.lock().await;
    let succeeded = registry.failover(&req.dataset_id, &req.shard_id, &req.promote_node_id);
    let status = if succeeded {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_replica_failover_result",
        "dataset_id": req.dataset_id,
        "shard_id": req.shard_id,
        "promote_node_id": req.promote_node_id,
        "status": if succeeded { "promoted" } else { "rejected" }
    });
    let response = (status, Json(payload)).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/replicas/failover",
            status,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_replica_diagnostics_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let registry = state.replica_registry.lock().await;
    let diagnostics = registry
        .list()
        .into_iter()
        .filter_map(|replica| {
            registry
                .diagnostics(&replica.metadata.dataset_id, &replica.metadata.shard_id)
                .map(|row| json!(row))
        })
        .collect::<Vec<_>>();
    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_replica_diagnostics_report",
        "diagnostics": diagnostics,
    });
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/cluster/replicas/diagnostics",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

#[tracing::instrument(skip(state))]
pub(crate) async fn cluster_recovery_run_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let now_unix_ms = chrono_like_unix_millis() as u64;

    let mut membership = state.membership.lock().await;
    let timed_out_nodes = membership.detect_timeouts(now_unix_ms);
    let live_nodes = membership
        .nodes()
        .into_iter()
        .filter(|node| membership.node_is_live(&node.descriptor.identity.node_id, now_unix_ms))
        .map(|node| node.descriptor.identity.node_id.clone())
        .collect::<Vec<_>>();
    drop(membership);

    let mut shard_registry = state.shard_registry.lock().await;
    let mut replica_registry = state.replica_registry.lock().await;
    let mut resilience = state.resilience_registry.lock().await;

    let mut shard_failovers = 0_u64;
    let mut replica_failovers = 0_u64;
    for node_id in &timed_out_nodes {
        resilience.record_failure(
            bijux_atlas_core::FailureCategory::NodeUnreachable,
            node_id,
            now_unix_ms,
            "node heartbeat timeout detected",
        );
    }

    if !live_nodes.is_empty() {
        for node_id in &timed_out_nodes {
            let shard_ids = shard_registry
                .shards_for_owner(node_id)
                .into_iter()
                .map(|shard| shard.metadata.shard_id.clone())
                .collect::<Vec<_>>();
            for shard_id in shard_ids {
                if let Some(new_owner) = live_nodes.iter().find(|candidate| *candidate != node_id) {
                    if shard_registry.transfer_ownership(&shard_id, new_owner) {
                        shard_failovers = shard_failovers.saturating_add(1);
                    }
                }
            }
        }

        let replica_keys = replica_registry
            .list()
            .into_iter()
            .map(|record| (record.metadata.dataset_id.clone(), record.metadata.shard_id.clone()))
            .collect::<Vec<_>>();
        for (dataset_id, shard_id) in replica_keys {
            let failover_target = replica_registry
                .get(&dataset_id, &shard_id)
                .and_then(|replica| {
                    if timed_out_nodes
                        .iter()
                        .any(|node| node == &replica.metadata.primary_node_id)
                    {
                        replica.metadata.replica_node_ids.first().cloned()
                    } else {
                        None
                    }
                });
            if let Some(target) = failover_target {
                if replica_registry.failover(&dataset_id, &shard_id, &target) {
                    replica_failovers = replica_failovers.saturating_add(1);
                }
            }
        }
    }

    let completed_at = chrono_like_unix_millis() as u64;
    resilience.record_recovery(
        "cluster",
        "automatic_recovery_workflow",
        now_unix_ms,
        completed_at,
        true,
    );
    tracing::info!(
        event_id = "cluster_recovery_run",
        timed_out_nodes = timed_out_nodes.len(),
        shard_failovers,
        replica_failovers,
        route = "/debug/recovery/run",
        "automatic cluster recovery run completed"
    );

    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_recovery_run_result",
        "timed_out_nodes": timed_out_nodes,
        "shard_failovers": shard_failovers,
        "replica_failovers": replica_failovers,
    });
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/recovery/run",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

#[tracing::instrument(skip(state))]
pub(crate) async fn recovery_diagnostics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let diagnostics = state.resilience_registry.lock().await.diagnostics();
    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_recovery_diagnostics_report",
        "diagnostics": diagnostics
    });
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/recovery/diagnostics",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

#[tracing::instrument(skip(state))]
pub(crate) async fn failure_injection_handler(
    State(state): State<AppState>,
    Json(req): Json<FailureInjectionRequest>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let now_unix_ms = chrono_like_unix_millis() as u64;
    let mut resilience = state.resilience_registry.lock().await;
    let (category, target_id, detail) = match req.kind.as_str() {
        "node_crash" => (
            bijux_atlas_core::FailureCategory::NodeUnreachable,
            req.node_id.unwrap_or_else(|| "node-a".to_string()),
            "simulated node crash",
        ),
        "shard_corruption" => (
            bijux_atlas_core::FailureCategory::ShardCorruption,
            req.shard_id.unwrap_or_else(|| "atlas-default-s001".to_string()),
            "simulated shard corruption",
        ),
        "network_partition" => (
            bijux_atlas_core::FailureCategory::NetworkPartition,
            req.node_id.unwrap_or_else(|| "node-b".to_string()),
            "simulated network partition",
        ),
        _ => (
            bijux_atlas_core::FailureCategory::Unknown,
            "cluster".to_string(),
            "simulated unknown fault",
        ),
    };
    let event_id = resilience.record_failure(category, target_id.clone(), now_unix_ms, detail);
    tracing::warn!(
        event_id = "failure_injection",
        route = "/debug/failure-injection",
        simulation_id = %event_id,
        target = %target_id,
        fault_kind = %req.kind,
        "failure injection recorded"
    );
    let payload = json!({
        "schema_version": 1,
        "kind": "failure_injection_result",
        "event_id": event_id,
        "target_id": target_id,
        "fault_kind": req.kind
    });
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/failure-injection",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

#[tracing::instrument(skip(state))]
pub(crate) async fn chaos_run_handler(
    State(state): State<AppState>,
    Json(req): Json<FailureInjectionRequest>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let now_unix_ms = chrono_like_unix_millis() as u64;
    let mut resilience = state.resilience_registry.lock().await;
    let id1 = resilience.record_failure(
        bijux_atlas_core::FailureCategory::NodeUnreachable,
        req.node_id.clone().unwrap_or_else(|| "node-a".to_string()),
        now_unix_ms,
        "chaos scenario injected node crash",
    );
    let id2 = resilience.record_failure(
        bijux_atlas_core::FailureCategory::NetworkPartition,
        req.node_id.unwrap_or_else(|| "node-b".to_string()),
        now_unix_ms.saturating_add(1),
        "chaos scenario injected network partition",
    );
    resilience.record_recovery(
        "cluster",
        "chaos_recovery_evaluation",
        now_unix_ms,
        now_unix_ms.saturating_add(10),
        true,
    );
    tracing::warn!(
        event_id = "chaos_run",
        route = "/debug/chaos/run",
        injection_a = %id1,
        injection_b = %id2,
        "chaos run executed"
    );
    let payload = json!({
        "schema_version": 1,
        "kind": "chaos_run_result",
        "injection_events": [id1, id2],
        "status": "recorded"
    });
    let response = Json(payload).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/debug/chaos/run",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn openapi_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let mut spec = bijux_atlas_api::openapi_v1_spec();
    if let Some(info) = spec.get_mut("info").and_then(serde_json::Value::as_object_mut) {
        info.insert(
            "x-build-id".to_string(),
            serde_json::Value::String(option_env!("BIJUX_BUILD_HASH").unwrap_or("dev").to_string()),
        );
    }
    let mut response = Json(spec).into_response();
    if let Ok(value) = HeaderValue::from_str("public, max-age=30") {
        response.headers_mut().insert("cache-control", value);
    }
    state
        .metrics
        .observe_request("/v1/openapi.json", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn readyz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let catalog_present = state.cache.current_catalog().await.is_some();
    let catalog_ready = readyz_catalog_ready(
        state.api.readiness_requires_catalog,
        state.cache.cached_only_mode(),
        catalog_present,
    );
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

pub(crate) async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let catalog_present = state.cache.current_catalog().await.is_some();
    let catalog_ready = readyz_catalog_ready(
        state.api.readiness_requires_catalog,
        state.cache.cached_only_mode(),
        catalog_present,
    );
    let status = if state.ready.load(Ordering::Relaxed) && catalog_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = if status == StatusCode::OK {
        "ready"
    } else {
        "not-ready"
    };
    let resp = (status, body).into_response();
    state
        .metrics
        .observe_request_with_trace("/ready", status, started.elapsed(), Some(&request_id))
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn live_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let is_live = state.accepting_requests.load(Ordering::Relaxed);
    let status = if is_live {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let resp = (
        status,
        Json(json!({
            "live": is_live,
            "draining": !is_live
        })),
    )
        .into_response();
    state
        .metrics
        .observe_request_with_trace("/live", status, started.elapsed(), Some(&request_id))
        .await;
    with_request_id(resp, &request_id)
}

fn readyz_catalog_ready(
    readiness_requires_catalog: bool,
    cached_only_mode: bool,
    catalog_present: bool,
) -> bool {
    if readiness_requires_catalog && !cached_only_mode {
        catalog_present
    } else {
        true
    }
}
