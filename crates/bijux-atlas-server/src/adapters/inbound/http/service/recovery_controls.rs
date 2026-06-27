// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::route_support::*;
use crate::*;
use bijux_atlas_runtime::domain::cluster::resilience::FailureCategory;
use serde_json::json;

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
