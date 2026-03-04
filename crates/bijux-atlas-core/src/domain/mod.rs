// SPDX-License-Identifier: Apache-2.0

pub mod canonical;
pub mod cluster_state;
pub mod config;
pub mod distributed;
pub mod distributed_config;
pub mod membership;
pub mod replication;
pub mod resilience;
pub mod security_auth;
pub mod security_authorization;
pub mod security_runtime;
pub mod sharding;
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
    default_metadata_store, load_cluster_config_from_path, load_node_config_from_path,
    ClusterConfigFile, ClusterDiscoveryConfig, ClusterHealthConfig, ClusterHealthQuorumConfig,
    NodeConfigFile, NodeShutdownConfig,
};
pub use membership::{
    HeartbeatMessage, MembershipMetrics, MembershipPolicy, MembershipRegistry, MembershipState,
    NodeMembershipRecord,
};
pub use replication::{
    ConsistencyGuarantee, ConsistencyLevel, ReplicaDiagnostics, ReplicaHealth, ReplicaMetadata,
    ReplicaRecord, ReplicaRegistry, ReplicaSyncState, ReplicationMetrics, ReplicationPolicy,
};
pub use resilience::{
    FailureCategory, FailureDetectionPolicy, FailureEvent, FailureRecoveryRegistry,
    RecoveryDiagnostics, RecoveryEvent, RecoveryPolicy, ResilienceGuarantees, ResilienceMetrics,
};
pub use security_auth::{
    authentication_context_from_api_key, authentication_context_from_token,
    extract_request_identity, generate_api_key, hash_api_key, mint_signed_token, rotate_api_key,
    validate_signed_token, ApiKeyRecord, ApiKeyStore, AuthValidationError, AuthenticationContext,
    RequestIdentity, TokenClaims, TokenValidationPolicy,
};
pub use security_authorization::{
    AuthorizationDecision, AuthorizationEngine, AuthorizationPolicy, AuthorizationPolicyRule,
    AuthorizationResources, PermissionCatalog, PermissionDefinition, PermissionEvaluator,
    RoleCatalog, RoleDefinition, RoleRegistry,
};
pub use security_runtime::{
    load_security_config_from_path, validate_security_config, EnvSecretsProvider, KeyManager,
    KeyRecord, SecretsProvider, SecurityAuditConfig, SecurityAuthConfig,
    SecurityAuthorizationConfig, SecurityConfig, SecurityEventConfig, SecurityIdentityConfig,
    SecurityKeyConfig, SecurityPolicy, SecurityPolicyRegistry, SecuritySecretsConfig,
    SecurityTransportConfig, StaticSecretsProvider,
};
pub use sharding::{
    stable_hash_u64, DatasetShardLayout, ShardHealth, ShardKeyStrategy, ShardMetadata,
    ShardOwnershipRule, ShardRecord, ShardRegistry, ShardRegistryMetrics, ShardRuntimeStats,
};
