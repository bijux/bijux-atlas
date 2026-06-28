// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::route_support::*;
use crate::*;
use bijux_atlas_runtime::domain::cluster::coordination::distributed::{
    NodeDescriptor, NodeIdentity, NodeRole, NodeState, ReadinessPolicy, ShutdownPolicy,
};
use bijux_atlas_runtime::domain::cluster::topology::config::{
    load_cluster_config_from_path, load_node_config_from_path,
};
use bijux_atlas_runtime::domain::cluster::topology::membership::HeartbeatMessage;
use bijux_atlas_runtime::domain::cluster::topology::state::{ClusterStateRegistry, NodeMetadata};
use serde_json::json;
use serde_json::Value;

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
                let membership_registry = state.membership_registry();
                let membership = membership_registry.lock().await;
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
        .metrics()
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
    let membership_registry = state.membership_registry();
    let mut membership = membership_registry.lock().await;
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
        .metrics()
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
    let membership_registry = state.membership_registry();
    let mut membership = membership_registry.lock().await;
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
        .metrics()
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
    let membership_registry = state.membership_registry();
    let mut membership = membership_registry.lock().await;
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
        .metrics()
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
    let membership_registry = state.membership_registry();
    let mut membership = membership_registry.lock().await;
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
        .metrics()
        .observe_request_with_trace(
            "/debug/cluster/mode",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(response, &request_id)
}
