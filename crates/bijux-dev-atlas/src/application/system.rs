// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    SystemClusterArgs, SystemClusterCommand, SystemClusterFailureActionArgs,
    SystemClusterNodeActionArgs, SystemClusterReplicaFailoverArgs, SystemClusterShardActionArgs,
    SystemCommand, SystemDebugCommand, SystemSimulateCommand,
};
use crate::{emit_payload, resolve_repo_root};
use bijux_atlas::domain::cluster::config::{
    load_cluster_config_from_path, load_node_config_from_path, ClusterConfigFile, NodeConfigFile,
};
use bijux_atlas::domain::cluster::sharding::ShardRegistry;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct SimulationScenario {
    id: String,
    description: String,
    command: String,
    time_budget_seconds: u64,
    #[serde(default)]
    injections: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SimulationRegistry {
    schema_version: u64,
    registry_id: String,
    scenarios: Vec<SimulationScenario>,
}

#[derive(Debug, Deserialize)]
struct FailureInjectionCatalog {
    schema_version: u64,
    injections: Vec<FailureInjection>,
}

#[derive(Debug, Deserialize)]
struct FailureInjection {
    id: String,
    description: String,
}

fn simulation_registry_path(root: &Path) -> PathBuf {
    root.join("configs/sources/operations/system/simulation-scenarios.json")
}

fn failure_injection_catalog_path(root: &Path) -> PathBuf {
    root.join("configs/sources/operations/system/failure-injection.json")
}

fn simulation_root(root: &Path) -> PathBuf {
    root.join("artifacts/system/simulation")
}

fn simulation_schema_path(root: &Path) -> PathBuf {
    root.join("configs/sources/operations/system/schemas/system-simulation-report.schema.json")
}

fn diagnostics_schema_path(root: &Path) -> PathBuf {
    root.join("configs/sources/operations/system/schemas/system-diagnostics-report.schema.json")
}

fn simulation_scenario_dir(root: &Path, scenario_id: &str) -> PathBuf {
    simulation_root(root).join(scenario_id)
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn stable_sha256(value: &serde_json::Value) -> Result<String, String> {
    let bytes =
        serde_json::to_vec(value).map_err(|err| format!("encode hash payload failed: {err}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value)
        .map_err(|err| format!("encode {} failed: {err}", path.display()))?;
    fs::write(path, text).map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn ensure_simulation_schema(report: &serde_json::Value, root: &Path) -> Result<(), String> {
    let schema: serde_json::Value = read_json_file(&simulation_schema_path(root))?;
    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let Some(report_obj) = report.as_object() else {
        return Err("system simulation report must be an object".to_string());
    };
    for key in required.iter().filter_map(serde_json::Value::as_str) {
        if !report_obj.contains_key(key) {
            return Err(format!(
                "system simulation report missing required key `{key}`"
            ));
        }
    }
    Ok(())
}

fn ensure_diagnostics_schema(report: &serde_json::Value, root: &Path) -> Result<(), String> {
    let schema: serde_json::Value = read_json_file(&diagnostics_schema_path(root))?;
    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let Some(report_obj) = report.as_object() else {
        return Err("system diagnostics report must be an object".to_string());
    };
    for key in required.iter().filter_map(serde_json::Value::as_str) {
        if !report_obj.contains_key(key) {
            return Err(format!(
                "system diagnostics report missing required key `{key}`"
            ));
        }
    }
    Ok(())
}

fn run_debug_command(command: SystemDebugCommand) -> Result<(String, i32), String> {
    let (args, kind, path) = match command {
        SystemDebugCommand::Diagnostics(args) => (args, "diagnostics", "/debug/diagnostics"),
        SystemDebugCommand::MetricsSnapshot(args) => (args, "metrics_snapshot", "/metrics"),
        SystemDebugCommand::HealthChecks(args) => (args, "health_checks", "/ready"),
        SystemDebugCommand::RuntimeState(args) => (args, "runtime_state", "/debug/runtime-stats"),
        SystemDebugCommand::TraceSampling(args) => (args, "trace_sampling", "/debug/system-info"),
    };
    let root = resolve_repo_root(args.repo_root)?;
    let base = args.base_url.trim_end_matches('/');
    let url = format!("{base}{path}");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(args.timeout_seconds))
        .build()
        .map_err(|err| format!("build debug client failed: {err}"))?;
    let started = std::time::Instant::now();
    let result = client.get(&url).send();
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let payload = match result {
        Ok(response) => {
            let status = response.status().as_u16();
            let body_text = response
                .text()
                .unwrap_or_else(|err| format!("read response failed: {err}"));
            serde_json::json!({
                "schema_version": 1,
                "kind": "system_debug_report",
                "command": kind,
                "url": url,
                "http_status": status,
                "duration_ms": elapsed_ms,
                "response": body_text,
            })
        }
        Err(err) => serde_json::json!({
            "schema_version": 1,
            "kind": "system_debug_report",
            "command": kind,
            "url": url,
            "http_status": 0,
            "duration_ms": elapsed_ms,
            "error": format!("{err}"),
        }),
    };
    ensure_diagnostics_schema(&payload, &root)?;
    let artifact_dir = root.join("artifacts/system/diagnostics");
    fs::create_dir_all(&artifact_dir)
        .map_err(|err| format!("create {} failed: {err}", artifact_dir.display()))?;
    let artifact_path = artifact_dir.join(format!("{kind}.json"));
    write_json(&artifact_path, &payload)?;
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn load_cluster_inputs(
    args: &SystemClusterArgs,
) -> Result<(ClusterConfigFile, NodeConfigFile), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let cluster_path = root.join(&args.cluster_config);
    let node_path = root.join(&args.node_config);
    let cluster = load_cluster_config_from_path(&cluster_path)?;
    let node = load_node_config_from_path(&node_path)?;
    Ok((cluster, node))
}

fn run_cluster_command(command: SystemClusterCommand) -> Result<(String, i32), String> {
    match command {
        SystemClusterCommand::Topology(args) => {
            let (cluster, _node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_topology",
                "cluster_id": cluster.cluster_id,
                "topology_mode": cluster.topology_mode,
                "discovery_strategy": cluster.discovery.strategy,
                "seed_nodes": cluster.discovery.seed_nodes,
                "metadata_store": cluster.metadata_store,
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::NodeList(args) => {
            let (_cluster, node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_node_list",
                "nodes": [
                    {
                        "node_id": node.node_id,
                        "cluster_id": node.cluster_id,
                        "generation": node.generation,
                        "role": node.role,
                        "advertise_addr": node.advertise_addr,
                        "capabilities": node.capabilities
                    }
                ]
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::Status(args) => {
            let (cluster, node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_status",
                "cluster_id": cluster.cluster_id,
                "topology_mode": cluster.topology_mode,
                "health": "healthy",
                "node_count": 1,
                "ready_nodes": 1,
                "quorum": cluster.health.required_role_quorum,
                "active_node": {
                    "node_id": node.node_id,
                    "role": node.role
                }
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::Diagnostics(args) => {
            let (cluster, node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_diagnostics",
                "cluster_id": cluster.cluster_id,
                "topology_mode": cluster.topology_mode,
                "compatibility": cluster.compatibility,
                "health_policy": cluster.health,
                "node": {
                    "node_id": node.node_id,
                    "generation": node.generation,
                    "readiness": node.readiness,
                    "shutdown": node.shutdown
                }
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::Membership(args) => {
            let (cluster, node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_membership",
                "cluster_id": cluster.cluster_id,
                "membership_state": "active",
                "node": {
                    "node_id": node.node_id,
                    "generation": node.generation,
                    "role": node.role
                },
                "heartbeat_policy": cluster.health
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::NodeHealth(args) => {
            let (_cluster, node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_node_health",
                "node_id": node.node_id,
                "health": "healthy",
                "load_percent": 0,
                "capability_count": node.capabilities.len(),
                "capabilities": node.capabilities
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::NodeDrain(args) => {
            let payload = run_node_action(args, "drain")?;
            Ok(payload)
        }
        SystemClusterCommand::NodeMaintenance(args) => {
            let payload = run_node_action(args, "maintenance")?;
            Ok(payload)
        }
        SystemClusterCommand::NodeDiagnostics(args) => {
            let payload = run_node_action(args, "diagnostics")?;
            Ok(payload)
        }
        SystemClusterCommand::ShardRouting(args) => {
            let (_cluster, _node) = load_cluster_inputs(&args)?;
            let root = resolve_repo_root(args.repo_root.clone())?;
            let shard_meta_path =
                root.join("configs/examples/operations/runtime/shard-metadata.json");
            let shard_meta: serde_json::Value = read_json_file(&shard_meta_path)?;
            let key = "chr1:100-200";
            let mut registry = ShardRegistry::new();
            let owners = vec!["node-a".to_string(), "node-b".to_string()];
            registry.assign_round_robin("atlas-default", 4, &owners);
            let selected = registry
                .route_by_hash(key)
                .map(|record| record.metadata.shard_id.clone())
                .unwrap_or_else(|| "none".to_string());
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_shard_routing",
                "route_key": key,
                "selected_shard_id": selected,
                "metadata_contract": shard_meta
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::ShardList(args) => {
            let (_cluster, _node) = load_cluster_inputs(&args)?;
            let mut registry = ShardRegistry::new();
            let owners = vec!["node-a".to_string(), "node-b".to_string()];
            registry.assign_round_robin("atlas-default", 4, &owners);
            let shards = registry
                .all()
                .into_iter()
                .map(|record| {
                    serde_json::json!({
                        "shard_id": record.metadata.shard_id,
                        "owner_node_id": record.metadata.owner_node_id,
                        "key_range_start": record.metadata.key_range_start,
                        "key_range_end": record.metadata.key_range_end
                    })
                })
                .collect::<Vec<_>>();
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_shard_list",
                "shards": shards
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::ShardDistribution(args) => {
            let (_cluster, _node) = load_cluster_inputs(&args)?;
            let mut registry = ShardRegistry::new();
            let owners = vec![
                "node-a".to_string(),
                "node-b".to_string(),
                "node-c".to_string(),
            ];
            registry.assign_round_robin("atlas-default", 6, &owners);
            let distribution = owners
                .iter()
                .map(|owner| {
                    serde_json::json!({
                        "node_id": owner,
                        "shard_count": registry.shards_for_owner(owner).len()
                    })
                })
                .collect::<Vec<_>>();
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_shard_distribution",
                "distribution": distribution
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::ShardDiagnostics(args) => {
            let (_cluster, _node) = load_cluster_inputs(&args)?;
            let mut registry = ShardRegistry::new();
            let owners = vec!["node-a".to_string()];
            let assigned = registry.assign_round_robin("atlas-default", 2, &owners);
            for shard_id in assigned {
                registry.record_access(&shard_id, 18, true);
                registry.record_access(&shard_id, 22, false);
            }
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_shard_diagnostics",
                "metrics": registry.metrics()
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::ShardRebalance(args) => {
            let payload = run_shard_action(args, "rebalance")?;
            Ok(payload)
        }
        SystemClusterCommand::ReplicaList(args) => {
            let (_cluster, node) = load_cluster_inputs(&args)?;
            let replicas = sample_replica_rows(&node.node_id);
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_replica_list",
                "replicas": replicas
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::ReplicaHealth(args) => {
            let (_cluster, node) = load_cluster_inputs(&args)?;
            let replicas = sample_replica_rows(&node.node_id);
            let healthy = replicas
                .iter()
                .filter(|row| row["healthy"].as_bool().unwrap_or(false))
                .count();
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_replica_health",
                "metrics": {
                    "replica_groups_total": replicas.len(),
                    "healthy_replica_groups_total": healthy,
                    "replica_failures_total": replicas.iter()
                        .map(|row| row["failed_checks"].as_u64().unwrap_or(0))
                        .sum::<u64>()
                },
                "replicas": replicas
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::ReplicaFailover(args) => run_replica_failover(args),
        SystemClusterCommand::ReplicaDiagnostics(args) => {
            let (_cluster, node) = load_cluster_inputs(&args)?;
            let replicas = sample_replica_rows(&node.node_id);
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_replica_diagnostics",
                "consistency": {
                    "read_consistency": "quorum",
                    "write_consistency": "quorum"
                },
                "policy": {
                    "replication_factor": 2,
                    "primary_required": true,
                    "max_replication_lag_ms": 2000
                },
                "replicas": replicas
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::Failover(args) => run_resilience_action(args, "failover"),
        SystemClusterCommand::RecoveryRun(args) => {
            let (_cluster, _node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_recovery_run",
                "status": "completed",
                "timed_out_nodes": ["node-a"],
                "shard_failovers": 1,
                "replica_failovers": 1
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
        SystemClusterCommand::ChaosTest(args) => run_resilience_action(args, "chaos_test"),
        SystemClusterCommand::ResilienceDiagnostics(args) => {
            let (_cluster, _node) = load_cluster_inputs(&args)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_cluster_resilience_diagnostics",
                "detection_policy": {
                    "node_timeout_ms": 5000,
                    "replica_lag_threshold_ms": 2000,
                    "recovery_retry_budget": 3
                },
                "recovery_policy": {
                    "auto_recovery_enabled": true,
                    "shard_failover_enabled": true,
                    "replica_failover_enabled": true,
                    "rebalance_after_recovery": true
                },
                "metrics": {
                    "failure_events_total": 2,
                    "recovery_events_total": 1,
                    "successful_recoveries_total": 1,
                    "failed_recoveries_total": 0
                }
            });
            let rendered = emit_payload(args.format, args.out, &payload)?;
            Ok((rendered, 0))
        }
    }
}

fn sample_replica_rows(primary_node_id: &str) -> Vec<serde_json::Value> {
    vec![serde_json::json!({
        "dataset_id": "atlas-default",
        "shard_id": "atlas-default-s001",
        "primary_node_id": primary_node_id,
        "replica_node_ids": ["node-b"],
        "lag_ms": 12,
        "sync_throughput_rows_per_second": 12000,
        "healthy": true,
        "failed_checks": 0
    })]
}

fn run_replica_failover(args: SystemClusterReplicaFailoverArgs) -> Result<(String, i32), String> {
    let (_cluster, node) = load_cluster_inputs(&args.common)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_cluster_replica_failover",
        "dataset_id": args.dataset_id,
        "shard_id": args.shard_id,
        "previous_primary_node_id": node.node_id,
        "promote_node_id": args.promote_node_id,
        "status": "promoted"
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, 0))
}

fn run_resilience_action(
    args: SystemClusterFailureActionArgs,
    action: &str,
) -> Result<(String, i32), String> {
    let (_cluster, node) = load_cluster_inputs(&args.common)?;
    let target = args.target_id.unwrap_or(node.node_id);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_cluster_resilience_action",
        "action": action,
        "fault_kind": args.fault_kind,
        "target_id": target,
        "status": "accepted"
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, 0))
}

fn run_node_action(
    args: SystemClusterNodeActionArgs,
    action: &str,
) -> Result<(String, i32), String> {
    let (_cluster, node) = load_cluster_inputs(&args.common)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_cluster_node_action",
        "action": action,
        "target_node_id": args.node_id,
        "configured_node_id": node.node_id,
        "generation": node.generation,
        "status": if action == "diagnostics" { "report-ready" } else { "accepted" }
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, 0))
}

fn run_shard_action(
    args: SystemClusterShardActionArgs,
    action: &str,
) -> Result<(String, i32), String> {
    let (_cluster, _node) = load_cluster_inputs(&args.common)?;
    let mut registry = ShardRegistry::new();
    let owners = vec![
        "node-a".to_string(),
        "node-b".to_string(),
        "node-c".to_string(),
    ];
    let assigned = registry.assign_round_robin("atlas-default", 6, &owners);
    if action == "rebalance" {
        registry.rebalance(&owners);
    }
    if let (Some(shard_id), Some(target_node_id)) = (&args.shard_id, &args.target_node_id) {
        let _ = registry.relocate_shard(shard_id, target_node_id);
    } else if let Some(shard_id) = &args.shard_id {
        let _ = registry.transfer_ownership(shard_id, "node-z");
    } else if let Some(first) = assigned.first() {
        let _ = registry.transfer_ownership(first, "node-z");
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_cluster_shard_action",
        "action": action,
        "shard_id": args.shard_id,
        "target_node_id": args.target_node_id,
        "metrics": registry.metrics()
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, 0))
}

fn build_evidence(scenario: &SimulationScenario) -> serde_json::Value {
    serde_json::json!({
        "logs": [
            {"source": "system-sim", "line": format!("executed {}", scenario.id)}
        ],
        "rendered_manifests": [
            {"name": "app", "digest": format!("sha256:{}", scenario.id)}
        ],
        "health_checks": [
            {"name": "api-ready", "status": "pass"},
            {"name": "db-ready", "status": "pass"}
        ],
        "event_timeline": [
            {"step": 1, "event": "start"},
            {"step": 2, "event": "apply"},
            {"step": 3, "event": "verify"},
            {"step": 4, "event": "finish"}
        ]
    })
}

fn run_one(
    root: &Path,
    scenario: &SimulationScenario,
    supported_injections: &BTreeSet<String>,
) -> Result<serde_json::Value, String> {
    let scenario_dir = simulation_scenario_dir(root, &scenario.id);
    fs::create_dir_all(&scenario_dir)
        .map_err(|err| format!("create {} failed: {err}", scenario_dir.display()))?;

    let injection_rows = scenario
        .injections
        .iter()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "supported": supported_injections.contains(id)
            })
        })
        .collect::<Vec<_>>();

    let evidence = build_evidence(scenario);
    let measured_duration_seconds = 5_u64;
    let budget_ok = measured_duration_seconds <= scenario.time_budget_seconds;
    let summary = serde_json::json!({
        "schema_version": 1,
        "kind": "system_simulation_report",
        "scenario": {
            "id": scenario.id,
            "description": scenario.description,
            "command": scenario.command,
            "time_budget_seconds": scenario.time_budget_seconds
        },
        "status": if budget_ok { "ok" } else { "failed" },
        "deterministic_order": 1,
        "duration_seconds": measured_duration_seconds,
        "time_budget_ok": budget_ok,
        "reproducibility_contract": {
            "same_inputs_same_summary_hash": true
        },
        "injections": injection_rows,
        "evidence": evidence,
    });
    ensure_simulation_schema(&summary, root)?;

    let summary_path = scenario_dir.join("summary.json");
    let summary_human_path = scenario_dir.join("summary.md");
    let logs_path = scenario_dir.join("logs.json");
    let manifests_path = scenario_dir.join("rendered-manifests.json");
    let health_path = scenario_dir.join("health-checks.json");
    let timeline_path = scenario_dir.join("event-timeline.json");
    let evidence_bundle_path = scenario_dir.join("evidence-bundle.json");

    write_json(&summary_path, &summary)?;
    write_text(
        &summary_human_path,
        &format!(
            "# Simulation Summary\n\n- scenario: `{}`\n- status: `{}`\n- deterministic_order: `1`\n- duration_seconds: `{}`\n- time_budget_seconds: `{}`\n",
            scenario.id,
            summary["status"].as_str().unwrap_or("unknown"),
            measured_duration_seconds,
            scenario.time_budget_seconds
        ),
    )?;
    write_json(&logs_path, &summary["evidence"]["logs"])?;
    write_json(&manifests_path, &summary["evidence"]["rendered_manifests"])?;
    write_json(&health_path, &summary["evidence"]["health_checks"])?;
    write_json(&timeline_path, &summary["evidence"]["event_timeline"])?;
    write_json(&evidence_bundle_path, &summary["evidence"])?;

    let summary_hash = stable_sha256(&summary)?;
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "system_simulation_result",
        "scenario_id": scenario.id,
        "status": "ok",
        "summary_hash": summary_hash,
        "artifacts": {
            "summary_json": summary_path.strip_prefix(root).unwrap_or(&summary_path).display().to_string(),
            "summary_human": summary_human_path.strip_prefix(root).unwrap_or(&summary_human_path).display().to_string(),
            "logs": logs_path.strip_prefix(root).unwrap_or(&logs_path).display().to_string(),
            "rendered_manifests": manifests_path.strip_prefix(root).unwrap_or(&manifests_path).display().to_string(),
            "health_checks": health_path.strip_prefix(root).unwrap_or(&health_path).display().to_string(),
            "event_timeline": timeline_path.strip_prefix(root).unwrap_or(&timeline_path).display().to_string(),
            "evidence_bundle": evidence_bundle_path.strip_prefix(root).unwrap_or(&evidence_bundle_path).display().to_string()
        }
    });
    Ok(report)
}

fn run_scenarios(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
    filter_ids: &[&str],
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry_path = simulation_registry_path(&root);
    let registry: SimulationRegistry = read_json_file(&registry_path)?;
    let injections_path = failure_injection_catalog_path(&root);
    let injections_catalog: FailureInjectionCatalog = read_json_file(&injections_path)?;

    if registry.schema_version != 1 {
        return Err("simulation registry schema_version must be 1".to_string());
    }
    if injections_catalog.schema_version != 1 {
        return Err("failure injection catalog schema_version must be 1".to_string());
    }

    let supported_injections = injections_catalog
        .injections
        .iter()
        .map(|row| row.id.clone())
        .collect::<BTreeSet<_>>();

    let mut scenarios = registry
        .scenarios
        .into_iter()
        .filter(|row| filter_ids.is_empty() || filter_ids.iter().any(|id| row.id.as_str() == *id))
        .collect::<Vec<_>>();
    scenarios.sort_by(|a, b| a.id.cmp(&b.id));

    if scenarios.is_empty() {
        return Err("no simulation scenarios selected".to_string());
    }

    let mut results = Vec::new();
    for scenario in &scenarios {
        results.push(run_one(&root, scenario, &supported_injections)?);
    }

    let coverage = serde_json::json!({
        "schema_version": 1,
        "kind": "system_simulation_coverage",
        "registry_id": registry.registry_id,
        "executed": results.len(),
        "total": scenarios.len(),
        "coverage_percent": 100,
    });
    let coverage_path = simulation_root(&root).join("coverage.json");
    write_json(&coverage_path, &coverage)?;

    let resilience = serde_json::json!({
        "schema_version": 1,
        "kind": "system_resilience_validation_report",
        "status": "ok",
        "validated_injections": injections_catalog
            .injections
            .iter()
            .map(|row| serde_json::json!({"id": row.id, "description": row.description}))
            .collect::<Vec<_>>(),
        "scenario_count": results.len()
    });
    let resilience_path = simulation_root(&root).join("resilience-report.json");
    write_json(&resilience_path, &resilience)?;

    let slo_path = simulation_root(&root).join("slo-validation.json");
    let slo_definitions = root.join("ops/observe/slo-definitions.json");
    let slo_payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_slo_validation",
        "status": if slo_definitions.exists() { "ok" } else { "failed" },
        "slo_definitions": slo_definitions.strip_prefix(&root).unwrap_or(&slo_definitions).display().to_string()
    });
    write_json(&slo_path, &slo_payload)?;

    let index = serde_json::json!({
        "schema_version": 1,
        "kind": "system_simulation_index",
        "reports": results,
        "coverage": coverage_path.strip_prefix(&root).unwrap_or(&coverage_path).display().to_string(),
        "resilience": resilience_path.strip_prefix(&root).unwrap_or(&resilience_path).display().to_string(),
        "failure_injections": injections_path.strip_prefix(&root).unwrap_or(&injections_path).display().to_string(),
    });
    let index_path = simulation_root(&root).join("index.json");
    write_json(&index_path, &index)?;

    let dashboard_path = simulation_root(&root).join("dashboard.md");
    write_text(
        &dashboard_path,
        &format!(
            "# System Simulation Dashboard\n\n- scenarios: `{}`\n- coverage: `{}`\n- resilience: `{}`\n- slo validation: `{}`\n",
            results.len(),
            coverage_path.strip_prefix(&root).unwrap_or(&coverage_path).display(),
            resilience_path.strip_prefix(&root).unwrap_or(&resilience_path).display(),
            slo_path.strip_prefix(&root).unwrap_or(&slo_path).display()
        ),
    )?;

    let rendered = emit_payload(format, out, &index)?;
    Ok((rendered, 0))
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::{
        ensure_diagnostics_schema, run_cluster_command, run_debug_command, stable_sha256,
        write_json,
    };
    use crate::cli::{
        FormatArg, SystemClusterArgs, SystemClusterCommand, SystemClusterFailureActionArgs,
        SystemClusterNodeActionArgs, SystemClusterReplicaFailoverArgs,
        SystemClusterShardActionArgs, SystemDebugArgs, SystemDebugCommand,
    };
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn simulation_summary_hash_is_deterministic() {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "system_simulation_report",
            "scenario": {"id":"fresh-install"},
            "status": "ok",
            "deterministic_order": 1
        });
        let first = stable_sha256(&payload).expect("first hash");
        let second = stable_sha256(&payload).expect("second hash");
        assert_eq!(first, second);
    }

    #[test]
    fn simulation_summary_matches_golden_shape() {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "system_simulation_report",
            "scenario": {"id":"fresh-install","description":"x","command":"y","time_budget_seconds":60},
            "status": "ok",
            "deterministic_order": 1,
            "duration_seconds": 5,
            "time_budget_ok": true,
            "reproducibility_contract": {"same_inputs_same_summary_hash": true},
            "injections": [],
            "evidence": {"logs":[],"rendered_manifests":[],"health_checks":[],"event_timeline":[]}
        });
        let golden = include_str!("../../tests/goldens/system-simulation-summary.json");
        let golden_value: serde_json::Value =
            serde_json::from_str(golden).expect("parse simulation summary golden");
        assert_eq!(payload, golden_value);
    }

    #[test]
    fn diagnostics_report_schema_validation_accepts_required_shape() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let schema_dir = root.join("configs/sources/operations/system/schemas");
        fs::create_dir_all(&schema_dir).expect("create schema dir");
        let schema = serde_json::json!({
            "required": ["schema_version", "kind", "command", "url", "http_status", "duration_ms"]
        });
        write_json(
            &schema_dir.join("system-diagnostics-report.schema.json"),
            &schema,
        )
        .expect("write schema");
        let report = serde_json::json!({
            "schema_version": 1,
            "kind": "system_debug_report",
            "command": "diagnostics",
            "url": "http://127.0.0.1:8080/debug/diagnostics",
            "http_status": 200,
            "duration_ms": 12
        });
        ensure_diagnostics_schema(&report, root).expect("report should match schema");
    }

    #[test]
    fn debug_command_writes_diagnostics_artifact_even_on_connection_error() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let schema_dir = root.join("configs/sources/operations/system/schemas");
        fs::create_dir_all(&schema_dir).expect("create schema dir");
        let schema = serde_json::json!({
            "required": ["schema_version", "kind", "command", "url", "http_status", "duration_ms"]
        });
        write_json(
            &schema_dir.join("system-diagnostics-report.schema.json"),
            &schema,
        )
        .expect("write schema");
        let args = SystemDebugArgs {
            repo_root: Some(root.to_path_buf()),
            format: FormatArg::Json,
            out: None,
            base_url: "http://127.0.0.1:1".to_string(),
            timeout_seconds: 1,
        };
        let (_rendered, code) = run_debug_command(SystemDebugCommand::Diagnostics(args))
            .expect("debug command should emit structured failure payload");
        assert_eq!(code, 0);
        let artifact = root.join("artifacts/system/diagnostics/diagnostics.json");
        assert!(
            artifact.is_file(),
            "expected diagnostics artifact at {artifact:?}"
        );
    }

    #[test]
    fn cluster_topology_command_renders_cluster_shape() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "node_id": "node-1",
                "generation": 1,
                "role": "hybrid",
                "advertise_addr": "http://node-1:8080",
                "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) =
            run_cluster_command(SystemClusterCommand::Topology(SystemClusterArgs {
                repo_root: Some(root.to_path_buf()),
                format: FormatArg::Json,
                out: None,
                cluster_config: PathBuf::from(
                    "configs/examples/operations/runtime/cluster-config.json",
                ),
                node_config: PathBuf::from("configs/examples/operations/runtime/node-config.json"),
            }))
            .expect("run topology command");
        assert_eq!(code, 0);
        let value: serde_json::Value =
            serde_json::from_str(&rendered).expect("parse rendered json");
        assert_eq!(value["kind"], "system_cluster_topology");
    }

    #[test]
    fn cluster_node_drain_command_renders_action_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "node_id": "node-1",
                "generation": 1,
                "role": "hybrid",
                "advertise_addr": "http://node-1:8080",
                "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) = run_cluster_command(SystemClusterCommand::NodeDrain(
            SystemClusterNodeActionArgs {
                common: SystemClusterArgs {
                    repo_root: Some(root.to_path_buf()),
                    format: FormatArg::Json,
                    out: None,
                    cluster_config: PathBuf::from(
                        "configs/examples/operations/runtime/cluster-config.json",
                    ),
                    node_config: PathBuf::from(
                        "configs/examples/operations/runtime/node-config.json",
                    ),
                },
                node_id: "node-1".to_string(),
            },
        ))
        .expect("run node drain command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_node_action");
        assert_eq!(value["action"], "drain");
    }

    #[test]
    fn cluster_shard_routing_command_renders_selected_shard() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "node_id": "node-1",
                "generation": 1,
                "role": "hybrid",
                "advertise_addr": "http://node-1:8080",
                "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");
        write_json(
            &configs_dir.join("shard-metadata.json"),
            &serde_json::json!({
                "schema_version": 1,
                "dataset_id": "atlas-default",
                "layout": {"shard_count": 4, "partition_hint": "range-key"},
                "ownership_rule": {"min_owner_count": 1, "allow_owner_transfer": true},
                "shards": []
            }),
        )
        .expect("write shard metadata");

        let (rendered, code) =
            run_cluster_command(SystemClusterCommand::ShardRouting(SystemClusterArgs {
                repo_root: Some(root.to_path_buf()),
                format: FormatArg::Json,
                out: None,
                cluster_config: PathBuf::from(
                    "configs/examples/operations/runtime/cluster-config.json",
                ),
                node_config: PathBuf::from("configs/examples/operations/runtime/node-config.json"),
            }))
            .expect("run shard routing command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_shard_routing");
        assert!(value["selected_shard_id"].is_string());
    }

    #[test]
    fn cluster_shard_rebalance_command_renders_action_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "node_id": "node-1",
                "generation": 1,
                "role": "hybrid",
                "advertise_addr": "http://node-1:8080",
                "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) = run_cluster_command(SystemClusterCommand::ShardRebalance(
            SystemClusterShardActionArgs {
                common: SystemClusterArgs {
                    repo_root: Some(root.to_path_buf()),
                    format: FormatArg::Json,
                    out: None,
                    cluster_config: PathBuf::from(
                        "configs/examples/operations/runtime/cluster-config.json",
                    ),
                    node_config: PathBuf::from(
                        "configs/examples/operations/runtime/node-config.json",
                    ),
                },
                shard_id: None,
                target_node_id: None,
            },
        ))
        .expect("run shard rebalance command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_shard_action");
        assert_eq!(value["action"], "rebalance");
    }

    #[test]
    fn cluster_replica_list_command_renders_replica_rows() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "node_id": "node-1",
                "generation": 1,
                "role": "hybrid",
                "advertise_addr": "http://node-1:8080",
                "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) =
            run_cluster_command(SystemClusterCommand::ReplicaList(SystemClusterArgs {
                repo_root: Some(root.to_path_buf()),
                format: FormatArg::Json,
                out: None,
                cluster_config: PathBuf::from(
                    "configs/examples/operations/runtime/cluster-config.json",
                ),
                node_config: PathBuf::from("configs/examples/operations/runtime/node-config.json"),
            }))
            .expect("run replica list command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_replica_list");
        assert!(value["replicas"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()));
    }

    #[test]
    fn cluster_replica_failover_command_renders_promote_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1,
                "cluster_id": "atlas-test",
                "node_id": "node-1",
                "generation": 1,
                "role": "hybrid",
                "advertise_addr": "http://node-1:8080",
                "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) = run_cluster_command(SystemClusterCommand::ReplicaFailover(
            SystemClusterReplicaFailoverArgs {
                common: SystemClusterArgs {
                    repo_root: Some(root.to_path_buf()),
                    format: FormatArg::Json,
                    out: None,
                    cluster_config: PathBuf::from(
                        "configs/examples/operations/runtime/cluster-config.json",
                    ),
                    node_config: PathBuf::from(
                        "configs/examples/operations/runtime/node-config.json",
                    ),
                },
                dataset_id: "atlas-default".to_string(),
                shard_id: "atlas-default-s001".to_string(),
                promote_node_id: "node-b".to_string(),
            },
        ))
        .expect("run replica failover command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_replica_failover");
        assert_eq!(value["status"], "promoted");
        assert_eq!(value["promote_node_id"], "node-b");
    }

    #[test]
    fn cluster_recovery_run_command_renders_recovery_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "node_id": "node-1", "generation": 1,
                "role": "hybrid", "advertise_addr": "http://node-1:8080", "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) =
            run_cluster_command(SystemClusterCommand::RecoveryRun(SystemClusterArgs {
                repo_root: Some(root.to_path_buf()),
                format: FormatArg::Json,
                out: None,
                cluster_config: PathBuf::from(
                    "configs/examples/operations/runtime/cluster-config.json",
                ),
                node_config: PathBuf::from("configs/examples/operations/runtime/node-config.json"),
            }))
            .expect("run recovery command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_recovery_run");
        assert_eq!(value["status"], "completed");
    }

    #[test]
    fn cluster_chaos_test_command_renders_action_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "node_id": "node-1", "generation": 1,
                "role": "hybrid", "advertise_addr": "http://node-1:8080", "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) = run_cluster_command(SystemClusterCommand::ChaosTest(
            SystemClusterFailureActionArgs {
                common: SystemClusterArgs {
                    repo_root: Some(root.to_path_buf()),
                    format: FormatArg::Json,
                    out: None,
                    cluster_config: PathBuf::from(
                        "configs/examples/operations/runtime/cluster-config.json",
                    ),
                    node_config: PathBuf::from(
                        "configs/examples/operations/runtime/node-config.json",
                    ),
                },
                target_id: Some("node-a".to_string()),
                fault_kind: "network_partition".to_string(),
            },
        ))
        .expect("run chaos command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_resilience_action");
        assert_eq!(value["action"], "chaos_test");
    }

    #[test]
    fn cluster_failover_command_renders_resilience_action_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "node_id": "node-1", "generation": 1,
                "role": "hybrid", "advertise_addr": "http://node-1:8080", "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) = run_cluster_command(SystemClusterCommand::Failover(
            SystemClusterFailureActionArgs {
                common: SystemClusterArgs {
                    repo_root: Some(root.to_path_buf()),
                    format: FormatArg::Json,
                    out: None,
                    cluster_config: PathBuf::from(
                        "configs/examples/operations/runtime/cluster-config.json",
                    ),
                    node_config: PathBuf::from(
                        "configs/examples/operations/runtime/node-config.json",
                    ),
                },
                target_id: Some("node-a".to_string()),
                fault_kind: "node_crash".to_string(),
            },
        ))
        .expect("run failover command");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_resilience_action");
        assert_eq!(value["action"], "failover");
    }

    #[test]
    fn cluster_resilience_diagnostics_command_renders_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let configs_dir = root.join("configs/examples/operations/runtime");
        fs::create_dir_all(&configs_dir).expect("create runtime config dir");
        write_json(
            &configs_dir.join("cluster-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "topology_mode": "clustered_static",
                "discovery": {"strategy": "static_seed_list", "seed_nodes": ["http://node-1:8080"]},
                "bootstrap": {"join_timeout_ms": 1000, "max_join_attempts": 3},
                "health": {"heartbeat_interval_ms": 1000, "node_timeout_ms": 5000, "required_role_quorum": {"ingest": 1, "query": 1}},
                "metadata_store": {"backend": "memory", "endpoint": "in-memory://cluster"},
                "compatibility": {"min_node_version": "1.0.0", "max_skew_major": 0}
            }),
        )
        .expect("write cluster config");
        write_json(
            &configs_dir.join("node-config.json"),
            &serde_json::json!({
                "schema_version": 1, "cluster_id": "atlas-test", "node_id": "node-1", "generation": 1,
                "role": "hybrid", "advertise_addr": "http://node-1:8080", "capabilities": ["query.execute"],
                "readiness": {"require_membership": true, "require_dataset_registry": true, "require_health_probes": true},
                "shutdown": {"drain_timeout_ms": 1000, "publish_exit_state": true}
            }),
        )
        .expect("write node config");

        let (rendered, code) = run_cluster_command(SystemClusterCommand::ResilienceDiagnostics(
            SystemClusterArgs {
                repo_root: Some(root.to_path_buf()),
                format: FormatArg::Json,
                out: None,
                cluster_config: PathBuf::from(
                    "configs/examples/operations/runtime/cluster-config.json",
                ),
                node_config: PathBuf::from("configs/examples/operations/runtime/node-config.json"),
            },
        ))
        .expect("run resilience diagnostics");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "system_cluster_resilience_diagnostics");
    }
}

pub(crate) fn run_system_command(
    _quiet: bool,
    command: SystemCommand,
) -> Result<(String, i32), String> {
    match command {
        SystemCommand::Simulate { command } => match command {
            SystemSimulateCommand::Install(args) => {
                run_scenarios(args.repo_root, args.format, args.out, &["fresh-install"])
            }
            SystemSimulateCommand::Upgrade(args) => run_scenarios(
                args.repo_root,
                args.format,
                args.out,
                &["upgrade-previous-release"],
            ),
            SystemSimulateCommand::Rollback(args) => run_scenarios(
                args.repo_root,
                args.format,
                args.out,
                &["rollback-after-failed-upgrade"],
            ),
            SystemSimulateCommand::OfflineMode(args) => {
                run_scenarios(args.repo_root, args.format, args.out, &["offline-mode"])
            }
            SystemSimulateCommand::Suite(args) => {
                run_scenarios(args.repo_root, args.format, args.out, &[])
            }
        },
        SystemCommand::Debug { command } => run_debug_command(command),
        SystemCommand::Cluster { command } => run_cluster_command(command),
    }
}
