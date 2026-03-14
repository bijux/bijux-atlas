// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::distributed::{ClusterDescriptor, ClusterHealth, NodeDescriptor, NodeState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub descriptor: NodeDescriptor,
    pub state: NodeState,
    pub last_heartbeat_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterStatusSnapshot {
    pub topology_version: u64,
    pub health: ClusterHealth,
    pub node_count: usize,
}

#[derive(Debug, Clone)]
pub struct ClusterStateRegistry {
    pub descriptor: ClusterDescriptor,
    pub topology_version: u64,
    nodes: BTreeMap<String, NodeMetadata>,
}

impl ClusterStateRegistry {
    #[must_use]
    pub fn new(descriptor: ClusterDescriptor) -> Self {
        Self {
            descriptor,
            topology_version: 1,
            nodes: BTreeMap::new(),
        }
    }

    pub fn register_node(&mut self, metadata: NodeMetadata) {
        self.nodes
            .insert(metadata.descriptor.identity.node_id.clone(), metadata);
        self.topology_version += 1;
    }

    #[must_use]
    pub fn nodes(&self) -> Vec<&NodeMetadata> {
        self.nodes.values().collect()
    }

    #[must_use]
    pub fn snapshot(&self) -> ClusterStatusSnapshot {
        let health = if self.nodes.is_empty() {
            ClusterHealth::Unavailable
        } else if self
            .nodes
            .values()
            .any(|node| node.state == NodeState::Failed)
        {
            ClusterHealth::Degraded
        } else {
            ClusterHealth::Healthy
        };
        ClusterStatusSnapshot {
            topology_version: self.topology_version,
            health,
            node_count: self.nodes.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ClusterStateRegistry, NodeMetadata};
    use crate::domain::distributed::{
        BootstrapPolicy, ClusterDescriptor, ClusterMetadataStore, CompatibilityPolicy,
        DiscoveryStrategy, HealthPolicy, MetadataBackend, NodeDescriptor, NodeIdentity, NodeRole,
        NodeState, ReadinessPolicy, ShutdownPolicy, TopologyMode,
    };

    #[test]
    fn node_lifecycle_can_be_recorded_in_cluster_registry() {
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
                backend: MetadataBackend::Memory,
                endpoint: "in-memory://cluster-state".to_string(),
            },
            compatibility: CompatibilityPolicy {
                min_node_version: "1.0.0".to_string(),
                max_skew_major: 0,
            },
        };

        let mut registry = ClusterStateRegistry::new(descriptor);
        let node = NodeMetadata {
            descriptor: NodeDescriptor {
                identity: NodeIdentity {
                    cluster_id: "atlas-prod-eu1".to_string(),
                    node_id: "atlas-node-1".to_string(),
                    generation: 1,
                },
                role: NodeRole::Hybrid,
                advertise_addr: "http://atlas-node-1.internal:8080".to_string(),
                capabilities: vec!["query.execute".to_string()],
                readiness: ReadinessPolicy {
                    require_membership: true,
                    require_dataset_registry: true,
                    require_health_probes: true,
                },
                shutdown: ShutdownPolicy {
                    drain_timeout_ms: 1_000,
                    publish_exit_state: true,
                },
            },
            state: NodeState::Ready,
            last_heartbeat_unix_ms: 1,
        };

        registry.register_node(node);
        let snapshot = registry.snapshot();
        assert_eq!(snapshot.node_count, 1);
        assert_eq!(
            snapshot.health,
            crate::domain::distributed::ClusterHealth::Healthy
        );
    }
}
