// SPDX-License-Identifier: Apache-2.0

pub mod canonical;
pub mod cluster_state;
pub mod config;
pub mod distributed;
pub mod distributed_config;
pub mod time;

pub use canonical::{sha256, sha256_hex, Hash256};
pub use cluster_state::{ClusterStateRegistry, ClusterStatusSnapshot, NodeMetadata};
pub use config::{resolve_bijux_cache_dir, resolve_bijux_config_path};
pub use distributed::{
    BootstrapPolicy, ClusterDescriptor, ClusterHealth, ClusterMetadataStore, CompatibilityPolicy,
    DiscoveryStrategy, HealthPolicy, MetadataBackend, NodeDescriptor, NodeIdentity, NodeRole,
    NodeState, ReadinessPolicy, ShutdownPolicy, TopologyMode,
};
pub use distributed_config::{
    ClusterConfigFile, ClusterDiscoveryConfig, ClusterHealthConfig, ClusterHealthQuorumConfig,
    NodeConfigFile, NodeShutdownConfig, default_metadata_store, load_cluster_config_from_path,
    load_node_config_from_path,
};
