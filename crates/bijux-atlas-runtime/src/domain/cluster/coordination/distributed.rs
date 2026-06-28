// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyMode {
    SingleNode,
    ClusteredStatic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeRole {
    Ingest,
    Query,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryStrategy {
    StaticSeedList,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub cluster_id: String,
    pub node_id: String,
    pub generation: u64,
}

impl NodeIdentity {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.cluster_id.trim().is_empty() && !self.node_id.trim().is_empty() && self.generation > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterHealth {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    Booting,
    Joining,
    Ready,
    Draining,
    Left,
    Failed,
}

impl NodeState {
    #[must_use]
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Booting, Self::Joining)
                | (Self::Joining, Self::Ready)
                | (Self::Ready, Self::Draining)
                | (Self::Draining, Self::Left)
                | (_, Self::Failed)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessPolicy {
    pub require_membership: bool,
    pub require_dataset_registry: bool,
    pub require_health_probes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShutdownPolicy {
    pub drain_timeout_ms: u64,
    pub publish_exit_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub identity: NodeIdentity,
    pub role: NodeRole,
    pub advertise_addr: String,
    pub capabilities: Vec<String>,
    pub readiness: ReadinessPolicy,
    pub shutdown: ShutdownPolicy,
}

impl NodeDescriptor {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.identity.is_valid()
            && !self.advertise_addr.trim().is_empty()
            && !self.capabilities.is_empty()
            && self.shutdown.drain_timeout_ms >= 100
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapPolicy {
    pub join_timeout_ms: u64,
    pub max_join_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthPolicy {
    pub heartbeat_interval_ms: u64,
    pub node_timeout_ms: u64,
    pub required_ingest_quorum: u32,
    pub required_query_quorum: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataBackend {
    Sqlite,
    Postgres,
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMetadataStore {
    pub backend: MetadataBackend,
    pub endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityPolicy {
    pub min_node_version: String,
    pub max_skew_major: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterDescriptor {
    pub cluster_id: String,
    pub topology_mode: TopologyMode,
    pub discovery_strategy: DiscoveryStrategy,
    pub seed_nodes: Vec<String>,
    pub bootstrap: BootstrapPolicy,
    pub health: HealthPolicy,
    pub metadata_store: ClusterMetadataStore,
    pub compatibility: CompatibilityPolicy,
}

impl ClusterDescriptor {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.cluster_id.trim().is_empty()
            && !self.seed_nodes.is_empty()
            && self.bootstrap.join_timeout_ms >= 100
            && self.bootstrap.max_join_attempts > 0
            && self.health.heartbeat_interval_ms >= 100
            && self.health.node_timeout_ms > self.health.heartbeat_interval_ms
            && !self.metadata_store.endpoint.trim().is_empty()
            && !self.compatibility.min_node_version.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_state_transition_contract_is_enforced() {
        assert!(NodeState::Booting.can_transition_to(&NodeState::Joining));
        assert!(NodeState::Joining.can_transition_to(&NodeState::Ready));
        assert!(NodeState::Ready.can_transition_to(&NodeState::Draining));
        assert!(NodeState::Draining.can_transition_to(&NodeState::Left));
        assert!(NodeState::Ready.can_transition_to(&NodeState::Failed));

        assert!(!NodeState::Booting.can_transition_to(&NodeState::Ready));
        assert!(!NodeState::Ready.can_transition_to(&NodeState::Joining));
    }

    #[test]
    fn cluster_descriptor_validation_requires_minimum_contract_fields() {
        let descriptor = ClusterDescriptor {
            cluster_id: "atlas-prod-eu1".to_string(),
            topology_mode: TopologyMode::ClusteredStatic,
            discovery_strategy: DiscoveryStrategy::StaticSeedList,
            seed_nodes: vec!["http://atlas-node-1.internal:8080".to_string()],
            bootstrap: BootstrapPolicy {
                join_timeout_ms: 5_000,
                max_join_attempts: 3,
            },
            health: HealthPolicy {
                heartbeat_interval_ms: 1_000,
                node_timeout_ms: 5_000,
                required_ingest_quorum: 1,
                required_query_quorum: 1,
            },
            metadata_store: ClusterMetadataStore {
                backend: MetadataBackend::Sqlite,
                endpoint: "file:.atlas/cluster-metadata.db".to_string(),
            },
            compatibility: CompatibilityPolicy {
                min_node_version: "1.0.0".to_string(),
                max_skew_major: 0,
            },
        };

        assert!(descriptor.is_valid());
    }
}
