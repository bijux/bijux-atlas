// SPDX-License-Identifier: Apache-2.0
//! Drift detection contract model.

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftType {
    Configuration,
    Artifact,
    Registry,
    RuntimeConfig,
    OpsProfile,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftClass {
    Missing,
    Mismatch,
    Unexpected,
}

pub fn explain_drift_type(input: &str) -> Option<serde_json::Value> {
    match input {
        "configuration" | "config" => Some(serde_json::json!({
            "drift_type": "configuration",
            "description": "Detects drift in baseline configuration contracts such as schema versions and required config files.",
            "detectors": ["configs/inventory.json schema guard"]
        })),
        "artifact" => Some(serde_json::json!({
            "drift_type": "artifact",
            "description": "Detects drift between registry manifest references and artifact files on disk.",
            "detectors": ["release/evidence/manifest.json file presence checks"]
        })),
        "registry" => Some(serde_json::json!({
            "drift_type": "registry",
            "description": "Detects drift between invariant runtime registry and checked-in registry index.",
            "detectors": ["ops/invariants/registry.json completeness checks"]
        })),
        "runtime_config" | "runtime-config" => Some(serde_json::json!({
            "drift_type": "runtime_config",
            "description": "Detects drift between runtime pinned datasets and generated dataset index.",
            "detectors": ["ops/k8s/values/offline.yaml pinned datasets"]
        })),
        "ops_profile" | "ops-profile" | "profile" => Some(serde_json::json!({
            "drift_type": "ops_profile",
            "description": "Detects drift between install matrix references and stack profile registry.",
            "detectors": ["ops/k8s/install-matrix.json profile linkage"]
        })),
        _ => None,
    }
}
