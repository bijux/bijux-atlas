// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::route_support::*;
use crate::*;
use serde_json::json;

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
