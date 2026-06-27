// SPDX-License-Identifier: Apache-2.0

use super::simulation_paths::{simulation_current_chart_path, simulation_previous_chart_path};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReleaseChartSource {
    Current,
    Previous,
}

pub fn release_chart_source_path(
    repo_root: &Path,
    source: ReleaseChartSource,
) -> Result<PathBuf, String> {
    let path = match source {
        ReleaseChartSource::Current => simulation_current_chart_path(repo_root),
        ReleaseChartSource::Previous => simulation_previous_chart_path(repo_root),
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

pub fn manifest_diff_summary(before: &str, after: &str) -> serde_json::Value {
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

pub fn configmap_env_keys_from_manifest(manifest: &str) -> Vec<String> {
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

pub fn manifest_contract_summary(manifest: &str) -> serde_json::Value {
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

pub fn lifecycle_compatibility_checks(
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
        "removed_required_env_keys": removed_required_env_keys
    })
}

#[cfg(test)]
mod tests {
    use super::{
        configmap_env_keys_from_manifest, lifecycle_compatibility_checks, manifest_diff_summary,
        release_chart_source_path, ReleaseChartSource,
    };

    #[test]
    fn configmap_contract_filters_to_env_style_keys() {
        let manifest = r#"
kind: ConfigMap
apiVersion: v1
metadata:
  name: atlas-env
data:
  LOG_LEVEL: info
  FEATURE_FLAG_X: enabled
  lower_case: ignored
"#;

        let keys = configmap_env_keys_from_manifest(manifest);

        assert_eq!(keys, vec!["FEATURE_FLAG_X", "LOG_LEVEL"]);
    }

    #[test]
    fn lifecycle_compatibility_reports_removed_required_env_keys() {
        let before = r#"
kind: ConfigMap
apiVersion: v1
metadata:
  name: atlas-env
data:
  LOG_LEVEL: info
  FEATURE_FLAG_X: enabled
"#;
        let after = r#"
kind: ConfigMap
apiVersion: v1
metadata:
  name: atlas-env
data:
  LOG_LEVEL: info
"#;

        let checks = lifecycle_compatibility_checks(before, after);

        assert_eq!(
            checks["removed_required_env_keys"],
            serde_json::json!(["FEATURE_FLAG_X"])
        );
    }

    #[test]
    fn manifest_diff_summary_counts_changed_lines() {
        let diff = manifest_diff_summary("alpha\nbeta\n", "alpha\ngamma\ndelta\n");

        assert_eq!(diff["before_lines"], 2);
        assert_eq!(diff["after_lines"], 3);
        assert_eq!(diff["changed_lines"], 2);
    }

    #[test]
    fn release_chart_source_path_reports_missing_previous_archive() {
        let root = tempfile::tempdir().expect("tempdir");

        let error = release_chart_source_path(root.path(), ReleaseChartSource::Previous)
            .expect_err("missing archive should fail");

        assert!(error.contains("artifacts/ops/chart-sources/previous/bijux-atlas.tgz"));
    }
}
