// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod generated;

pub mod domain;
pub mod effects;
pub mod errors;
pub mod ports;
pub mod types;

pub use crate::domain::canonical;
pub use crate::domain::time;
// export contract: pub use crate::domain::{resolve_bijux_cache_dir, resolve_bijux_config_path, sha256, sha256_hex, Hash256}
pub use crate::domain::{
    BootstrapPolicy, ClusterConfigFile, ClusterDescriptor, ClusterDiscoveryConfig, ClusterHealth,
    ClusterHealthConfig, ClusterHealthQuorumConfig, ClusterMetadataStore, ClusterStateRegistry,
    ClusterStatusSnapshot, CompatibilityPolicy, DiscoveryStrategy, HeartbeatMessage, HealthPolicy,
    MembershipMetrics, MembershipPolicy, MembershipRegistry, MembershipState, MetadataBackend,
    FailureCategory, FailureDetectionPolicy, FailureEvent, FailureRecoveryRegistry,
    RecoveryDiagnostics, RecoveryEvent, RecoveryPolicy, ResilienceGuarantees, ResilienceMetrics,
    ConsistencyGuarantee, ConsistencyLevel, ReplicaDiagnostics, ReplicaHealth, ReplicaMetadata,
    ReplicaRecord, ReplicaRegistry, ReplicaSyncState, ReplicationMetrics, ReplicationPolicy,
    DatasetShardLayout, ShardHealth, ShardKeyStrategy, ShardMetadata, ShardOwnershipRule,
    ShardRecord, ShardRegistry, ShardRegistryMetrics, ShardRuntimeStats, stable_hash_u64,
    NodeConfigFile, NodeDescriptor, NodeIdentity, NodeMembershipRecord, NodeMetadata, NodeRole,
    NodeShutdownConfig, NodeState, ReadinessPolicy, ShutdownPolicy, TopologyMode,
    default_metadata_store, load_cluster_config_from_path, load_node_config_from_path,
    resolve_bijux_cache_dir, resolve_bijux_config_path, sha256, sha256_hex, Hash256,
};
pub use crate::errors::{
    ConfigPathScope, ERROR_CODES, Error, ErrorCode, ErrorContext, ExitCode, MachineError, Result,
    ResultExt,
};
pub use crate::ports::{ClockPort, FsPort, NetPort, ProcessPort, ProcessResult};
pub use crate::types::{DatasetId, RunId, ShardId};

pub const CRATE_NAME: &str = "bijux-atlas-core";
pub const ENV_BIJUX_LOG_LEVEL: &str = "BIJUX_LOG_LEVEL";
pub const ENV_BIJUX_CACHE_DIR: &str = "BIJUX_CACHE_DIR";
pub const NO_RANDOMNESS_POLICY: &str = "Randomness is forbidden in bijux-atlas-core";

#[must_use]
pub const fn no_randomness_policy() -> &'static str {
    NO_RANDOMNESS_POLICY
}
