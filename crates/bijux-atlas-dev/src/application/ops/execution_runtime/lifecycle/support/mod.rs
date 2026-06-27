// SPDX-License-Identifier: Apache-2.0
//! Foundational helpers for install-status flows.

use super::{
    simulation_current_chart_path, simulation_previous_chart_path, simulation_report_path,
};
use crate::ops_commands::sha256_hex;
use crate::{OpsProcess, RunId};

pub(super) fn readiness_baseline_path(
    repo_root: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let path = repo_root.join("artifacts/ops/history/readiness-baselines.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

pub(super) fn resolve_chart_source(
    repo_root: &std::path::Path,
    chart_source: crate::cli::OpsHelmChartSource,
) -> Result<std::path::PathBuf, String> {
    let path = match chart_source {
        crate::cli::OpsHelmChartSource::Current => simulation_current_chart_path(repo_root),
        crate::cli::OpsHelmChartSource::Previous => simulation_previous_chart_path(repo_root),
    };
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "missing chart source {}; current uses the working tree chart and previous uses artifacts/ops/chart-sources/previous/bijux-atlas.tgz",
            path.display()
        ))
    }
}

pub(super) fn manifest_diff_summary(before: &str, after: &str) -> serde_json::Value {
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let shared = before_lines.len().min(after_lines.len());
    let changed_lines = (0..shared)
        .filter(|index| before_lines[*index] != after_lines[*index])
        .count()
        + before_lines.len().saturating_sub(shared)
        + after_lines.len().saturating_sub(shared);
    serde_json::json!({
        "before_sha256": sha256_hex(before),
        "after_sha256": sha256_hex(after),
        "before_lines": before_lines.len(),
        "after_lines": after_lines.len(),
        "changed_lines": changed_lines
    })
}

pub(super) fn configmap_env_keys_from_manifest(manifest: &str) -> Vec<String> {
    let mut keys = std::collections::BTreeSet::<String>::new();
    for document in serde_yaml::Deserializer::from_str(manifest) {
        let value: serde_yaml::Value = match serde::Deserialize::deserialize(document) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("kind").and_then(serde_yaml::Value::as_str) != Some("ConfigMap") {
            continue;
        }
        let Some(data) = value.get("data").and_then(serde_yaml::Value::as_mapping) else {
            continue;
        };
        for key in data.keys().filter_map(serde_yaml::Value::as_str) {
            if key
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                keys.insert(key.to_string());
            }
        }
    }
    keys.into_iter().collect()
}

pub(super) fn manifest_contract_summary(manifest: &str) -> serde_json::Value {
    let mut services = Vec::<serde_json::Value>::new();
    let mut pvcs = Vec::<serde_json::Value>::new();
    let mut ingresses = Vec::<serde_json::Value>::new();
    let mut hpas = Vec::<serde_json::Value>::new();
    let mut network_policies = Vec::<serde_json::Value>::new();

    for document in serde_yaml::Deserializer::from_str(manifest) {
        let value: serde_yaml::Value = match serde::Deserialize::deserialize(document) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let kind = value
            .get("kind")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        let metadata = value.get("metadata");
        let name = metadata
            .and_then(|meta| meta.get("name"))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        match kind {
            "Service" => {
                let selector = value
                    .get("spec")
                    .and_then(|spec| spec.get("selector"))
                    .and_then(serde_yaml::Value::as_mapping)
                    .map(|mapping| {
                        let mut pairs = mapping
                            .iter()
                            .filter_map(|(key, value)| {
                                Some((key.as_str()?.to_string(), value.as_str()?.to_string()))
                            })
                            .collect::<Vec<_>>();
                        pairs.sort();
                        pairs
                    })
                    .unwrap_or_default();
                let mut ports = value
                    .get("spec")
                    .and_then(|spec| spec.get("ports"))
                    .and_then(serde_yaml::Value::as_sequence)
                    .map(|rows| {
                        rows.iter()
                            .filter_map(|row| row.get("port").and_then(serde_yaml::Value::as_i64))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                ports.sort();
                services.push(serde_json::json!({
                    "name": name,
                    "selector": selector,
                    "ports": ports
                }));
            }
            "PersistentVolumeClaim" => {
                pvcs.push(serde_json::json!({
                    "name": name,
                    "storage_class_name": value
                        .get("spec")
                        .and_then(|spec| spec.get("storageClassName"))
                        .and_then(serde_yaml::Value::as_str)
                        .unwrap_or_default()
                }));
            }
            "Ingress" => {
                let mut hosts = value
                    .get("spec")
                    .and_then(|spec| spec.get("rules"))
                    .and_then(serde_yaml::Value::as_sequence)
                    .map(|rows| {
                        rows.iter()
                            .filter_map(|row| row.get("host").and_then(serde_yaml::Value::as_str))
                            .map(str::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                hosts.sort();
                ingresses.push(serde_json::json!({
                    "name": name,
                    "hosts": hosts
                }));
            }
            "HorizontalPodAutoscaler" => {
                let spec = value.get("spec");
                let metrics = spec
                    .and_then(|row| row.get("metrics"))
                    .and_then(serde_yaml::Value::as_sequence)
                    .cloned()
                    .unwrap_or_default();
                let metric_target = |resource_name: &str| {
                    metrics.iter().find_map(|metric| {
                        let resource = metric.get("resource")?;
                        if resource.get("name").and_then(serde_yaml::Value::as_str)
                            == Some(resource_name)
                        {
                            resource
                                .get("target")
                                .and_then(|target| target.get("averageUtilization"))
                                .and_then(serde_yaml::Value::as_i64)
                        } else {
                            None
                        }
                    })
                };
                hpas.push(serde_json::json!({
                    "name": name,
                    "min_replicas": spec.and_then(|row| row.get("minReplicas")).and_then(serde_yaml::Value::as_i64),
                    "max_replicas": spec.and_then(|row| row.get("maxReplicas")).and_then(serde_yaml::Value::as_i64),
                    "cpu_target": metric_target("cpu"),
                    "memory_target": metric_target("memory")
                }));
            }
            "NetworkPolicy" => {
                network_policies.push(serde_json::json!({ "name": name }));
            }
            _ => {}
        }
    }

    services.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    pvcs.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    ingresses.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    hpas.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    network_policies.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));

    serde_json::json!({
        "services": services,
        "persistent_volume_claims": pvcs,
        "ingresses": ingresses,
        "hpas": hpas,
        "network_policies": network_policies,
        "configmap_env_keys": configmap_env_keys_from_manifest(manifest)
    })
}

pub(super) fn lifecycle_compatibility_checks(
    before_manifest: &str,
    after_manifest: &str,
) -> serde_json::Value {
    let before = manifest_contract_summary(before_manifest);
    let after = manifest_contract_summary(after_manifest);
    let service_names_stable = before["services"]
        .as_array()
        .zip(after["services"].as_array())
        .map(|(left, right)| {
            left.iter()
                .map(|row| row["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>()
                == right
                    .iter()
                    .map(|row| row["name"].as_str().unwrap_or_default())
                    .collect::<Vec<_>>()
        })
        .unwrap_or(false);
    let service_selector_stable = before["services"] == after["services"]
        || before["services"]
            .as_array()
            .zip(after["services"].as_array())
            .map(|(left, right)| {
                left.iter()
                    .map(|row| (&row["name"], &row["selector"]))
                    .collect::<Vec<_>>()
                    == right
                        .iter()
                        .map(|row| (&row["name"], &row["selector"]))
                        .collect::<Vec<_>>()
            })
            .unwrap_or(false);
    let service_ports_stable = before["services"]
        .as_array()
        .zip(after["services"].as_array())
        .map(|(left, right)| {
            left.iter()
                .map(|row| (&row["name"], &row["ports"]))
                .collect::<Vec<_>>()
                == right
                    .iter()
                    .map(|row| (&row["name"], &row["ports"]))
                    .collect::<Vec<_>>()
        })
        .unwrap_or(false);
    let pvc_stable = before["persistent_volume_claims"] == after["persistent_volume_claims"];
    let ingress_host_shape_stable = before["ingresses"] == after["ingresses"];
    let network_policy_default_stable = before["network_policies"] == after["network_policies"];
    let hpa_defaults_stable = before["hpas"] == after["hpas"];
    let before_env = before["configmap_env_keys"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let after_env = after["configmap_env_keys"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let removed_required_env_keys = before_env
        .iter()
        .filter_map(serde_json::Value::as_str)
        .filter(|key| {
            !after_env
                .iter()
                .any(|candidate| candidate.as_str() == Some(*key))
        })
        .map(str::to_string)
        .collect::<Vec<_>>();
    serde_json::json!({
        "immutable_fields_safe": service_names_stable && service_selector_stable && pvc_stable,
        "service_name_stable": service_names_stable,
        "service_selector_stable": service_selector_stable,
        "service_ports_stable": service_ports_stable,
        "pvc_definitions_stable": pvc_stable,
        "ingress_host_shape_stable": ingress_host_shape_stable,
        "networkpolicy_default_stable": network_policy_default_stable,
        "hpa_defaults_stable": hpa_defaults_stable,
        "required_env_keys_stable": removed_required_env_keys.is_empty(),
        "removed_required_env_keys": removed_required_env_keys,
        "before": before,
        "after": after
    })
}

pub(super) fn deployment_revision(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> Option<i64> {
    let argv = vec![
        "get".to_string(),
        "deployment".to_string(),
        "bijux-atlas".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let (stdout, _) = process.run_subprocess("kubectl", &argv, repo_root).ok()?;
    let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
    json.get("metadata")
        .and_then(|row| row.get("annotations"))
        .and_then(|row| row.get("deployment.kubernetes.io/revision"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<i64>().ok())
}

pub(super) fn rollout_history(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> serde_json::Value {
    let argv = vec![
        "rollout".to_string(),
        "history".to_string(),
        "deployment/bijux-atlas".to_string(),
        "-n".to_string(),
        namespace.to_string(),
    ];
    match process.run_subprocess("kubectl", &argv, repo_root) {
        Ok((stdout, event)) => serde_json::json!({
            "status": "ok",
            "stdout": stdout,
            "event": event
        }),
        Err(err) => serde_json::json!({
            "status": "failed",
            "error": err.to_stable_message()
        }),
    }
}

pub(super) fn pods_restart_count(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> u64 {
    let argv = vec![
        "get".to_string(),
        "pods".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let Ok((stdout, _)) = process.run_subprocess("kubectl", &argv, repo_root) else {
        return 0;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) else {
        return 0;
    };
    json.get("items")
        .and_then(serde_json::Value::as_array)
        .map(|rows| {
            rows.iter()
                .flat_map(|row| {
                    row.get("status")
                        .and_then(|status| status.get("containerStatuses"))
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .filter_map(|container| {
                    container
                        .get("restartCount")
                        .and_then(serde_json::Value::as_u64)
                })
                .sum()
        })
        .unwrap_or(0)
}

pub(super) fn update_lifecycle_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
    namespace: &str,
    update: LifecycleSummaryUpdate<'_>,
) -> Result<std::path::PathBuf, String> {
    let LifecycleSummaryUpdate {
        upgrade_report_path,
        upgrade_status,
        rollback_report_path,
        rollback_status,
    } = update;
    let summary_path = simulation_report_path(repo_root, run_id, "ops-lifecycle-summary.json")?;
    let mut payload = if summary_path.exists() {
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&summary_path)
                .map_err(|err| format!("failed to read {}: {err}", summary_path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", summary_path.display()))?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "cluster": "kind",
            "profiles": []
        })
    };
    if !payload["profiles"].is_array() {
        payload["profiles"] = serde_json::json!([]);
    }
    let rows = payload["profiles"]
        .as_array_mut()
        .ok_or_else(|| "ops-lifecycle-summary.json profiles must be an array".to_string())?;
    if let Some(existing) = rows
        .iter_mut()
        .find(|row| row.get("profile").and_then(|v| v.as_str()) == Some(profile))
    {
        existing["namespace"] = serde_json::json!(namespace);
        if let Some(path) = upgrade_report_path {
            existing["upgrade_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = upgrade_status {
            existing["upgrade_status"] = serde_json::json!(status);
        }
        if let Some(path) = rollback_report_path {
            existing["rollback_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = rollback_status {
            existing["rollback_status"] = serde_json::json!(status);
        }
    } else {
        let mut row = serde_json::json!({
            "profile": profile,
            "namespace": namespace
        });
        if let Some(path) = upgrade_report_path {
            row["upgrade_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = upgrade_status {
            row["upgrade_status"] = serde_json::json!(status);
        }
        if let Some(path) = rollback_report_path {
            row["rollback_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = rollback_status {
            row["rollback_status"] = serde_json::json!(status);
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| {
        a.get("profile")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("profile").and_then(serde_json::Value::as_str))
    });
    std::fs::write(
        &summary_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", summary_path.display()))?;
    Ok(summary_path)
}

pub(super) struct LifecycleSummaryUpdate<'a> {
    pub(super) upgrade_report_path: Option<&'a std::path::Path>,
    pub(super) upgrade_status: Option<&'a str>,
    pub(super) rollback_report_path: Option<&'a std::path::Path>,
    pub(super) rollback_status: Option<&'a str>,
}

pub(super) fn load_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
) -> Result<Option<u128>, String> {
    let path = readiness_baseline_path(repo_root)?;
    if !path.exists() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(value
        .get("profiles")
        .and_then(|rows| rows.as_object())
        .and_then(|rows| rows.get(profile))
        .and_then(|row| row.get("baseline_elapsed_ms"))
        .and_then(serde_json::Value::as_u64)
        .map(u128::from))
}

pub(super) fn update_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
    elapsed_ms: u128,
) -> Result<std::path::PathBuf, String> {
    let path = readiness_baseline_path(repo_root)?;
    let mut payload = if path.exists() {
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "profiles": {}
        })
    };
    if !payload["profiles"].is_object() {
        payload["profiles"] = serde_json::json!({});
    }
    payload["profiles"][profile] = serde_json::json!({
        "baseline_elapsed_ms": elapsed_ms
    });
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use crate::ops_execution_runtime::lifecycle::simulation_records::drill_check_paths;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn drill_source_paths_exist_for_current_workspace_layout() {
        let root = repo_root();
        for drill in ["catalog-unreachable", "invalid-config-rejected"] {
            for (_, path) in drill_check_paths(&root, drill) {
                assert!(
                    path.exists(),
                    "missing drill source path: {}",
                    path.display()
                );
            }
        }
    }
}
