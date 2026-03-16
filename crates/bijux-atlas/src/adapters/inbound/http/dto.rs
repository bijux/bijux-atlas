// SPDX-License-Identifier: Apache-2.0

use crate::domain::cluster::resilience::FailureCategory;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FailureInjectionPlan {
    pub(crate) category: FailureCategory,
    pub(crate) target_id: String,
    pub(crate) detail: &'static str,
}

fn required_debug_target(value: Option<&str>, field: &'static str) -> Result<String, &'static str> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(field);
    };
    Ok(value.to_string())
}

pub(crate) fn resolve_failure_injection_plan(
    req: &FailureInjectionRequest,
) -> Result<FailureInjectionPlan, &'static str> {
    match req.kind.as_str() {
        "node_crash" => Ok(FailureInjectionPlan {
            category: FailureCategory::NodeUnreachable,
            target_id: required_debug_target(req.node_id.as_deref(), "node_id")?,
            detail: "simulated node crash",
        }),
        "shard_corruption" => Ok(FailureInjectionPlan {
            category: FailureCategory::ShardCorruption,
            target_id: required_debug_target(req.shard_id.as_deref(), "shard_id")?,
            detail: "simulated shard corruption",
        }),
        "network_partition" => Ok(FailureInjectionPlan {
            category: FailureCategory::NetworkPartition,
            target_id: required_debug_target(req.node_id.as_deref(), "node_id")?,
            detail: "simulated network partition",
        }),
        _ => Err("kind"),
    }
}

pub(crate) fn resolve_chaos_target_node(
    req: &FailureInjectionRequest,
) -> Result<String, &'static str> {
    required_debug_target(req.node_id.as_deref(), "node_id")
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_chaos_target_node, resolve_failure_injection_plan, FailureInjectionRequest,
    };
    use crate::domain::cluster::resilience::FailureCategory;

    #[test]
    fn failure_injection_requires_explicit_targets() {
        let node_crash = resolve_failure_injection_plan(&FailureInjectionRequest {
            kind: "node_crash".to_string(),
            node_id: None,
            shard_id: None,
        });
        let shard_corruption = resolve_failure_injection_plan(&FailureInjectionRequest {
            kind: "shard_corruption".to_string(),
            node_id: None,
            shard_id: Some(String::new()),
        });
        let unknown = resolve_failure_injection_plan(&FailureInjectionRequest {
            kind: "unknown".to_string(),
            node_id: Some("node-1".to_string()),
            shard_id: None,
        });

        assert_eq!(node_crash, Err("node_id"));
        assert_eq!(shard_corruption, Err("shard_id"));
        assert_eq!(unknown, Err("kind"));
    }

    #[test]
    fn failure_injection_uses_explicit_supported_targets() {
        let node_crash = resolve_failure_injection_plan(&FailureInjectionRequest {
            kind: "node_crash".to_string(),
            node_id: Some("node-prod-1".to_string()),
            shard_id: None,
        })
        .expect("node crash plan");
        let shard_corruption = resolve_failure_injection_plan(&FailureInjectionRequest {
            kind: "shard_corruption".to_string(),
            node_id: None,
            shard_id: Some("shard-prod-9".to_string()),
        })
        .expect("shard corruption plan");

        assert_eq!(node_crash.target_id, "node-prod-1");
        assert_eq!(node_crash.category, FailureCategory::NodeUnreachable);
        assert_eq!(shard_corruption.target_id, "shard-prod-9");
        assert_eq!(shard_corruption.category, FailureCategory::ShardCorruption);
    }

    #[test]
    fn chaos_run_requires_explicit_node_id() {
        let missing = resolve_chaos_target_node(&FailureInjectionRequest {
            kind: "chaos".to_string(),
            node_id: None,
            shard_id: None,
        });
        let present = resolve_chaos_target_node(&FailureInjectionRequest {
            kind: "chaos".to_string(),
            node_id: Some("node-prod-1".to_string()),
            shard_id: None,
        });

        assert_eq!(missing, Err("node_id"));
        assert_eq!(present.as_deref(), Ok("node-prod-1"));
    }
}
