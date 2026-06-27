// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::route_support::*;
use crate::*;
use bijux_atlas_runtime::domain::cluster::resilience::FailureCategory;
use serde_json::json;

mod cluster_membership;

pub(crate) use cluster_membership::{
    cluster_heartbeat_handler, cluster_mode_handler, cluster_nodes_handler,
    cluster_register_handler, cluster_status_handler,
};

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
        crate::version::runtime_version(),
        list
    );
    let mut resp = Response::new(Body::from(html));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        "content-type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    state
        .metrics()
        .observe_request("/", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn healthz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let resp = (StatusCode::OK, "ok").into_response();
    state
        .metrics()
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
        .metrics()
        .observe_request_with_trace(
            "/health",
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
    let overloaded = crate::adapters::inbound::http::middleware::shedding::overloaded(&state).await;
    let live = state.accepting_requests.load(Ordering::Relaxed);
    let catalog_present = state.cache.current_catalog().await.is_some();
    let ready = state.ready.load(Ordering::Relaxed)
        && readyz_catalog_ready(
            state.api.readiness_requires_catalog,
            state.cache.cached_only_mode(),
            catalog_present,
        );
    let status = if overloaded {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };
    let resp = (
        status,
        Json(json!({
            "overloaded": overloaded,
            "ready": ready,
            "live": live,
            "draining": !live,
            "cached_only_mode": state.cache.cached_only_mode(),
            "emergency_breaker": state.api.emergency_global_breaker
        })),
    )
        .into_response();
    state
        .metrics()
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
            "version": crate::version::runtime_version(),
            "compatible_umbrella": ">=0.3.0,<0.4.0",
            "build_hash": bijux_atlas_runtime::runtime::config::runtime_build_hash(),
        },
        "server": {
            "crate": CRATE_NAME,
            "config_schema_version": bijux_atlas_runtime::runtime::config::CONFIG_SCHEMA_VERSION,
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
        .metrics()
        .observe_request("/v1/version", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn cluster_replica_list_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let replica_registry = state.replica_registry();
    let registry = replica_registry.lock().await;
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
        .metrics()
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
    let replica_registry = state.replica_registry();
    let registry = replica_registry.lock().await;
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
        .metrics()
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
    let replica_registry = state.replica_registry();
    let mut registry = replica_registry.lock().await;
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
        .metrics()
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
    let replica_registry = state.replica_registry();
    let registry = replica_registry.lock().await;
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
        .metrics()
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
pub(crate) async fn cluster_recovery_run_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let now_unix_ms = chrono_like_unix_millis() as u64;

    let membership_registry = state.membership_registry();
    let mut membership = membership_registry.lock().await;
    let timed_out_nodes = membership.detect_timeouts(now_unix_ms);
    let live_nodes = membership
        .nodes()
        .into_iter()
        .filter(|node| membership.node_is_live(&node.descriptor.identity.node_id, now_unix_ms))
        .map(|node| node.descriptor.identity.node_id.clone())
        .collect::<Vec<_>>();
    drop(membership);

    let shard_registry_handle = state.shard_registry();
    let replica_registry_handle = state.replica_registry();
    let resilience_registry = state.resilience_registry();
    let mut shard_registry = shard_registry_handle.lock().await;
    let mut replica_registry = replica_registry_handle.lock().await;
    let mut resilience = resilience_registry.lock().await;

    let mut shard_failovers = 0_u64;
    let mut replica_failovers = 0_u64;
    for node_id in &timed_out_nodes {
        resilience.record_failure(
            FailureCategory::NodeUnreachable,
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
            .map(|record| {
                (
                    record.metadata.dataset_id.clone(),
                    record.metadata.shard_id.clone(),
                )
            })
            .collect::<Vec<_>>();
        for (dataset_id, shard_id) in replica_keys {
            let failover_target =
                replica_registry
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
        .metrics()
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
pub(crate) async fn recovery_diagnostics_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let resilience_registry = state.resilience_registry();
    let diagnostics = resilience_registry.lock().await.diagnostics();
    let payload = json!({
        "schema_version": 1,
        "kind": "cluster_recovery_diagnostics_report",
        "diagnostics": diagnostics
    });
    let response = Json(payload).into_response();
    state
        .metrics()
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
    let plan = match resolve_failure_injection_plan(&req) {
        Ok(plan) => plan,
        Err(field) => {
            let response = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::InvalidQueryParameter,
                    "debug failure injection requires an explicit supported target",
                    json!({
                        "field": field,
                        "kind": req.kind,
                        "supported_kinds": ["node_crash", "shard_corruption", "network_partition"],
                    }),
                ),
            );
            state
                .metrics()
                .observe_request_with_trace(
                    "/debug/failure-injection",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                    Some(&request_id),
                )
                .await;
            return with_request_id(response, &request_id);
        }
    };
    let now_unix_ms = chrono_like_unix_millis() as u64;
    let resilience_registry = state.resilience_registry();
    let mut resilience = resilience_registry.lock().await;
    let category = match plan.category {
        FailureInjectionCategory::NodeCrash => FailureCategory::NodeUnreachable,
        FailureInjectionCategory::ShardCorruption => FailureCategory::ShardCorruption,
        FailureInjectionCategory::NetworkPartition => FailureCategory::NetworkPartition,
    };
    let event_id =
        resilience.record_failure(category, plan.target_id.clone(), now_unix_ms, plan.detail);
    tracing::warn!(
        event_id = "failure_injection",
        route = "/debug/failure-injection",
        simulation_id = %event_id,
        target = %plan.target_id,
        fault_kind = %req.kind,
        "failure injection recorded"
    );
    let payload = json!({
        "schema_version": 1,
        "kind": "failure_injection_result",
        "event_id": event_id,
        "target_id": plan.target_id,
        "fault_kind": req.kind
    });
    let response = Json(payload).into_response();
    state
        .metrics()
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
    let target_node_id = match resolve_chaos_target_node(&req) {
        Ok(node_id) => node_id,
        Err(field) => {
            let response = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::InvalidQueryParameter,
                    "debug chaos run requires an explicit node_id",
                    json!({
                        "field": field,
                        "kind": req.kind,
                    }),
                ),
            );
            state
                .metrics()
                .observe_request_with_trace(
                    "/debug/chaos/run",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                    Some(&request_id),
                )
                .await;
            return with_request_id(response, &request_id);
        }
    };
    let now_unix_ms = chrono_like_unix_millis() as u64;
    let resilience_registry = state.resilience_registry();
    let mut resilience = resilience_registry.lock().await;
    let id1 = resilience.record_failure(
        FailureCategory::NodeUnreachable,
        target_node_id.clone(),
        now_unix_ms,
        "chaos scenario injected node crash",
    );
    let id2 = resilience.record_failure(
        FailureCategory::NetworkPartition,
        target_node_id.clone(),
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
        .metrics()
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
    if let Some(info) = spec
        .get_mut("info")
        .and_then(serde_json::Value::as_object_mut)
    {
        info.insert(
            "x-build-id".to_string(),
            serde_json::Value::String(
                bijux_atlas_runtime::runtime::config::runtime_build_hash().to_string(),
            ),
        );
    }
    let mut response = Json(spec).into_response();
    if let Ok(value) = HeaderValue::from_str("public, max-age=30") {
        response.headers_mut().insert("cache-control", value);
    }
    state
        .metrics()
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
            .metrics()
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
            .metrics()
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
        .metrics()
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
        .metrics()
        .observe_request_with_trace("/live", status, started.elapsed(), Some(&request_id))
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) fn readyz_catalog_ready(
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

#[cfg(test)]
mod tests {
    use super::readyz_catalog_ready;

    #[test]
    fn readyz_requires_catalog_when_enabled_and_not_cached_only() {
        assert!(!readyz_catalog_ready(true, false, false));
        assert!(readyz_catalog_ready(true, false, true));
    }

    #[test]
    fn readyz_ignores_catalog_when_cached_only_or_not_required() {
        assert!(readyz_catalog_ready(true, true, false));
        assert!(readyz_catalog_ready(false, false, false));
        assert!(readyz_catalog_ready(false, true, false));
    }

    #[test]
    fn readyz_offline_profile_stays_ready_without_catalog() {
        assert!(readyz_catalog_ready(true, true, false));
    }

    #[test]
    fn readyz_baseline_and_perf_profiles_require_catalog_when_enabled() {
        assert!(!readyz_catalog_ready(true, false, false));
        assert!(readyz_catalog_ready(true, false, true));
    }
}
