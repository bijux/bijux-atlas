// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::domain::cluster::distributed::{
    BootstrapPolicy, ClusterDescriptor, ClusterMetadataStore, CompatibilityPolicy,
    DiscoveryStrategy, HealthPolicy, MetadataBackend, NodeDescriptor, NodeIdentity, NodeRole,
    ReadinessPolicy, TopologyMode,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterDiscoveryConfig {
    pub strategy: DiscoveryStrategy,
    pub seed_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterHealthQuorumConfig {
    pub ingest: u32,
    pub query: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterHealthConfig {
    pub heartbeat_interval_ms: u64,
    pub node_timeout_ms: u64,
    pub required_role_quorum: ClusterHealthQuorumConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterConfigFile {
    pub schema_version: u64,
    pub cluster_id: String,
    pub topology_mode: TopologyMode,
    pub discovery: ClusterDiscoveryConfig,
    pub bootstrap: BootstrapPolicy,
    pub health: ClusterHealthConfig,
    pub metadata_store: ClusterMetadataStore,
    pub compatibility: CompatibilityPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeShutdownConfig {
    pub drain_timeout_ms: u64,
    pub publish_exit_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeConfigFile {
    pub schema_version: u64,
    pub cluster_id: String,
    pub node_id: String,
    pub generation: u64,
    pub role: NodeRole,
    pub advertise_addr: String,
    pub capabilities: Vec<String>,
    pub readiness: ReadinessPolicy,
    pub shutdown: NodeShutdownConfig,
}

impl ClusterConfigFile {
    #[must_use]
    pub fn validate(&self) -> bool {
        if self.schema_version != 1 {
            return false;
        }
        let descriptor = self.to_descriptor();
        descriptor.is_valid()
    }

    #[must_use]
    pub fn to_descriptor(&self) -> ClusterDescriptor {
        ClusterDescriptor {
            cluster_id: self.cluster_id.clone(),
            topology_mode: self.topology_mode.clone(),
            discovery_strategy: self.discovery.strategy.clone(),
            seed_nodes: self.discovery.seed_nodes.clone(),
            bootstrap: self.bootstrap.clone(),
            health: HealthPolicy {
                heartbeat_interval_ms: self.health.heartbeat_interval_ms,
                node_timeout_ms: self.health.node_timeout_ms,
                required_ingest_quorum: self.health.required_role_quorum.ingest,
                required_query_quorum: self.health.required_role_quorum.query,
            },
            metadata_store: ClusterMetadataStore {
                backend: self.metadata_store.backend.clone(),
                endpoint: self.metadata_store.endpoint.clone(),
            },
            compatibility: self.compatibility.clone(),
        }
    }
}

impl NodeConfigFile {
    #[must_use]
    pub fn validate(&self) -> bool {
        if self.schema_version != 1 {
            return false;
        }
        self.to_descriptor().is_valid()
    }

    #[must_use]
    pub fn to_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            identity: NodeIdentity {
                cluster_id: self.cluster_id.clone(),
                node_id: self.node_id.clone(),
                generation: self.generation,
            },
            role: self.role.clone(),
            advertise_addr: self.advertise_addr.clone(),
            capabilities: self.capabilities.clone(),
            readiness: self.readiness.clone(),
            shutdown: crate::domain::cluster::distributed::ShutdownPolicy {
                drain_timeout_ms: self.shutdown.drain_timeout_ms,
                publish_exit_state: self.shutdown.publish_exit_state,
            },
        }
    }
}

pub fn load_cluster_config_from_path(path: &Path) -> Result<ClusterConfigFile, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("read cluster config {} failed: {err}", path.display()))?;
    let value: ClusterConfigFile = serde_json::from_str(&text)
        .map_err(|err| format!("parse cluster config {} failed: {err}", path.display()))?;
    if !value.validate() {
        return Err(format!(
            "cluster config {} failed validation",
            path.display()
        ));
    }
    Ok(value)
}

pub fn load_node_config_from_path(path: &Path) -> Result<NodeConfigFile, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("read node config {} failed: {err}", path.display()))?;
    let value: NodeConfigFile = serde_json::from_str(&text)
        .map_err(|err| format!("parse node config {} failed: {err}", path.display()))?;
    if !value.validate() {
        return Err(format!("node config {} failed validation", path.display()));
    }
    Ok(value)
}

#[must_use]
pub fn default_metadata_store() -> ClusterMetadataStore {
    ClusterMetadataStore {
        backend: MetadataBackend::Memory,
        endpoint: "in-memory://cluster-state".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{load_cluster_config_from_path, load_node_config_from_path};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root")
            .to_path_buf()
    }

    #[test]
    fn cluster_config_loader_accepts_example_contract() {
        let path = repo_root().join("configs/ops/runtime/cluster-config.example.json");
        let loaded = load_cluster_config_from_path(&path).expect("load cluster config");
        assert_eq!(loaded.schema_version, 1);
        assert!(loaded.validate());
        assert_eq!(loaded.cluster_id, "atlas-prod-eu1");
    }

    #[test]
    fn node_config_loader_accepts_example_contract() {
        let path = repo_root().join("configs/ops/runtime/node-config.example.json");
        let loaded = load_node_config_from_path(&path).expect("load node config");
        assert_eq!(loaded.schema_version, 1);
        assert!(loaded.validate());
        assert_eq!(loaded.node_id, "atlas-node-1");
    }

    #[test]
    fn cluster_bootstrap_contract_enforces_join_attempts() {
        let path = repo_root().join("configs/ops/runtime/cluster-config.example.json");
        let mut loaded = load_cluster_config_from_path(&path).expect("load cluster config");
        loaded.bootstrap.max_join_attempts = 0;
        assert!(!loaded.validate());
    }
}
