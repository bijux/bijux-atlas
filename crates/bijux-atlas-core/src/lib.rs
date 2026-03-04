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
    authentication_context_from_api_key, authentication_context_from_token, default_metadata_store,
    extract_request_identity, generate_api_key, hash_api_key, load_cluster_config_from_path,
    load_node_config_from_path, load_security_config_from_path, mint_signed_token,
    resolve_bijux_cache_dir, resolve_bijux_config_path, rotate_api_key, sha256, sha256_hex,
    stable_hash_u64, validate_security_config, validate_signed_token, ApiKeyRecord, ApiKeyStore,
    AuthValidationError, AuthenticationContext, AuthorizationDecision, AuthorizationEngine,
    AuthorizationPolicy, AuthorizationPolicyRule, AuthorizationResources, BootstrapPolicy,
    ClusterConfigFile, ClusterDescriptor, ClusterDiscoveryConfig, ClusterHealth,
    ClusterHealthConfig, ClusterHealthQuorumConfig, ClusterMetadataStore, ClusterStateRegistry,
    ClusterStatusSnapshot, CompatibilityPolicy, ConsistencyGuarantee, ConsistencyLevel,
    DatasetShardLayout, DiscoveryStrategy, EnvSecretsProvider, FailureCategory,
    FailureDetectionPolicy, FailureEvent, FailureRecoveryRegistry, Hash256, HealthPolicy,
    HeartbeatMessage, KeyManager, KeyRecord, MembershipMetrics, MembershipPolicy,
    MembershipRegistry, MembershipState, MetadataBackend, NodeConfigFile, NodeDescriptor,
    NodeIdentity, NodeMembershipRecord, NodeMetadata, NodeRole, NodeShutdownConfig, NodeState,
    PermissionCatalog, PermissionDefinition, PermissionEvaluator, ReadinessPolicy,
    RecoveryDiagnostics, RecoveryEvent, RecoveryPolicy, ReplicaDiagnostics, ReplicaHealth,
    ReplicaMetadata, ReplicaRecord, ReplicaRegistry, ReplicaSyncState, ReplicationMetrics,
    ReplicationPolicy, RequestIdentity, ResilienceGuarantees, ResilienceMetrics, RoleCatalog,
    RoleDefinition, RoleRegistry, SecretsProvider, SecurityAuditConfig, SecurityAuthConfig,
    SecurityAuthorizationConfig, SecurityConfig, SecurityEventConfig, SecurityIdentityConfig,
    SecurityKeyConfig, SecurityPolicy, SecurityPolicyRegistry, SecuritySecretsConfig,
    SecurityTransportConfig, ShardHealth, ShardKeyStrategy, ShardMetadata, ShardOwnershipRule,
    ShardRecord, ShardRegistry, ShardRegistryMetrics, ShardRuntimeStats, ShutdownPolicy,
    StaticSecretsProvider, TokenClaims, TokenValidationPolicy, TopologyMode,
};
pub use crate::errors::{
    ConfigPathScope, Error, ErrorCode, ErrorContext, ExitCode, MachineError, Result, ResultExt,
    ERROR_CODES,
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
