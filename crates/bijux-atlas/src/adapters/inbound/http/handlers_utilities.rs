// SPDX-License-Identifier: Apache-2.0

use crate::*;
use crate::domain::cluster::config::{load_cluster_config_from_path, load_node_config_from_path};
use crate::domain::{
    ClusterStateRegistry, FailureCategory, HeartbeatMessage, NodeDescriptor, NodeIdentity,
    NodeMetadata, NodeRole, NodeState, ReadinessPolicy, ShutdownPolicy,
};
use serde_json::json;
use serde_json::Value;
pub(crate) use crate::adapters::inbound::http::cache_headers::*;
pub(crate) use crate::adapters::inbound::http::dto::*;
pub(crate) use crate::adapters::inbound::http::presenters::*;
pub(crate) use crate::adapters::inbound::http::request_identity::*;
pub(crate) use crate::adapters::inbound::http::response_encoding::*;

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
            "compatible_umbrella": ">=0.3.0,<0.4.0",
            "build_hash": crate::runtime::config::runtime_build_hash(),
        },
        "server": {
            "crate": CRATE_NAME,
            "config_schema_version": crate::runtime::config::CONFIG_SCHEMA_VERSION,
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
    let cluster_path = std::env::var("ATLAS_CLUSTER_CONFIG_PATH").ok();
    let node_path = std::env::var("ATLAS_NODE_CONFIG_PATH").ok();

    let mut response_status = StatusCode::OK;
    let payload = match (cluster_path.as_deref(), node_path.as_deref()) {
        (Some(cluster_path), Some(node_path)) => match (
            load_cluster_config_from_path(std::path::Path::new(cluster_path)),
            load_node_config_from_path(std::path::Path::new(node_path)),
        ) {
        (Ok(cluster_cfg), Ok(node_cfg)) => {
            let cluster = cluster_cfg.to_descriptor();
            let node = node_cfg.to_descriptor();
            let mut registry = ClusterStateRegistry::new(cluster.clone());
            registry.register_node(NodeMetadata {
                descriptor: node,
                state: NodeState::Ready,
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
        },
        _ => {
            response_status = StatusCode::SERVICE_UNAVAILABLE;
            json!({
                "cluster_id": Value::Null,
                "health": "unavailable",
                "error": {
                    "cluster_config": "ATLAS_CLUSTER_CONFIG_PATH must be set",
                    "node_config": "ATLAS_NODE_CONFIG_PATH must be set"
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
        "ingest" => NodeRole::Ingest,
        "query" => NodeRole::Query,
        _ => NodeRole::Hybrid,
    };
    let descriptor = NodeDescriptor {
        identity: NodeIdentity {
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
        readiness: ReadinessPolicy {
            require_membership: true,
            require_dataset_registry: true,
            require_health_probes: true,
        },
        shutdown: ShutdownPolicy {
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
    membership.apply_heartbeat(HeartbeatMessage {
        identity: NodeIdentity {
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

pub(crate) async fn cluster_replica_list_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
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
pub(crate) async fn cluster_recovery_run_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
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
pub(crate) async fn recovery_diagnostics_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
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
                .metrics
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
    let mut resilience = state.resilience_registry.lock().await;
    let event_id = resilience.record_failure(
        plan.category,
        plan.target_id.clone(),
        now_unix_ms,
        plan.detail,
    );
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
                .metrics
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
    let mut resilience = state.resilience_registry.lock().await;
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
    let mut spec = crate::contracts::api::openapi_v1_spec();
    if let Some(info) = spec
        .get_mut("info")
        .and_then(serde_json::Value::as_object_mut)
    {
        info.insert(
            "x-build-id".to_string(),
            serde_json::Value::String(crate::runtime::config::runtime_build_hash().to_string()),
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
