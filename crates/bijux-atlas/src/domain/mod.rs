// SPDX-License-Identifier: Apache-2.0

pub mod canonical;
pub mod cluster;
pub mod config;
pub mod dataset;
pub mod ingest;
pub mod policy;
pub mod query;
pub mod security;
pub mod time;

pub use canonical::{sha256, sha256_hex, Hash256};
pub use cluster::config as distributed_config;
pub use cluster::distributed;
pub use cluster::membership;
pub use cluster::replication;
pub use cluster::resilience;
pub use cluster::routing;
pub use cluster::sharding;
pub use cluster::state as cluster_state;
pub use cluster::state::{ClusterStateRegistry, ClusterStatusSnapshot, NodeMetadata};
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
pub use security::auth as security_auth;
pub use security::authorization as security_authorization;
pub use security::data_protection as security_data_protection;
pub use security::runtime as security_runtime;
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
pub use security_data_protection::{
    calculate_manifest_checksum, detect_tampering, https_enforced, load_certificate_bundle,
    tls_handshake_allowed, validate_certificate_bundle, verify_artifact_checksum,
    verify_artifact_signature, verify_dataset_manifest_integrity, CertificateRotationState,
    CertificateValidationError, DataProtectionPolicy, DatasetManifestIntegrity, EncryptionAtRest,
    LoadedCertificate, TlsConfig, XorEncryption,
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
