// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityIdentityConfig {
    pub principal_source: String,
    pub trust_header: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuthConfig {
    pub mode: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuthorizationConfig {
    pub default_decision: String,
    pub policy_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecuritySecretsConfig {
    pub provider: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityKeyConfig {
    pub active_key_id: String,
    pub key_ring: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityTransportConfig {
    pub tls_required: bool,
    pub min_tls_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditConfig {
    pub enabled: bool,
    pub sink: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityEventConfig {
    pub classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub schema_version: u64,
    pub identity: SecurityIdentityConfig,
    pub auth: SecurityAuthConfig,
    pub authorization: SecurityAuthorizationConfig,
    pub secrets: SecuritySecretsConfig,
    pub keys: SecurityKeyConfig,
    pub transport: SecurityTransportConfig,
    pub audit: SecurityAuditConfig,
    pub events: SecurityEventConfig,
}

pub fn load_security_config_from_path(path: &Path) -> Result<SecurityConfig, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

pub trait SecretsProvider {
    fn get_secret(&self, reference: &str) -> Option<String>;
}

#[derive(Debug, Default)]
pub struct StaticSecretsProvider {
    pub values: BTreeMap<String, String>,
}

impl SecretsProvider for StaticSecretsProvider {
    fn get_secret(&self, reference: &str) -> Option<String> {
        self.values.get(reference).cloned()
    }
}

#[derive(Debug, Default)]
pub struct EnvSecretsProvider;

impl SecretsProvider for EnvSecretsProvider {
    fn get_secret(&self, reference: &str) -> Option<String> {
        std::env::var(reference).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRecord {
    pub key_id: String,
    pub purpose: String,
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct KeyManager {
    keys: BTreeMap<String, KeyRecord>,
}

impl KeyManager {
    #[must_use]
    pub fn new(records: Vec<KeyRecord>) -> Self {
        let mut keys = BTreeMap::new();
        for record in records {
            keys.insert(record.key_id.clone(), record);
        }
        Self { keys }
    }

    pub fn rotate(&mut self, next_key_id: &str) -> bool {
        if !self.keys.contains_key(next_key_id) {
            return false;
        }
        for record in self.keys.values_mut() {
            record.active = record.key_id == next_key_id;
        }
        true
    }

    #[must_use]
    pub fn active_key(&self) -> Option<&KeyRecord> {
        self.keys.values().find(|record| record.active)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub policy_id: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SecurityPolicyRegistry {
    policies: BTreeMap<String, SecurityPolicy>,
}

impl SecurityPolicyRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, policy: SecurityPolicy) -> bool {
        self.policies
            .insert(policy.policy_id.clone(), policy)
            .is_none()
    }

    #[must_use]
    pub fn list(&self) -> Vec<&SecurityPolicy> {
        self.policies.values().collect()
    }

    #[must_use]
    pub fn enabled_count(&self) -> usize {
        self.policies
            .values()
            .filter(|policy| policy.enabled)
            .count()
    }
}

pub fn validate_security_config(config: &SecurityConfig) -> Vec<String> {
    let mut errors = Vec::new();
    if config.schema_version == 0 {
        errors.push("schema_version must be greater than 0".to_string());
    }
    if config.identity.principal_source.trim().is_empty() {
        errors.push("identity.principal_source must not be empty".to_string());
    }
    if !matches!(
        config.auth.mode.as_str(),
        "disabled" | "api-key" | "oidc" | "mtls"
    ) {
        errors.push("auth.mode must be one of: disabled, api-key, oidc, mtls".to_string());
    }
    if !matches!(
        config.authorization.default_decision.as_str(),
        "allow" | "deny"
    ) {
        errors.push("authorization.default_decision must be allow or deny".to_string());
    }
    if config.secrets.provider.trim().is_empty() {
        errors.push("secrets.provider must not be empty".to_string());
    }
    if config.keys.active_key_id.trim().is_empty() {
        errors.push("keys.active_key_id must not be empty".to_string());
    }
    if config.transport.tls_required && config.transport.min_tls_version.trim().is_empty() {
        errors.push("transport.min_tls_version must not be empty when tls is required".to_string());
    }
    if config.audit.enabled && !matches!(config.audit.sink.as_str(), "stdout" | "file" | "otel") {
        errors.push("audit.sink must be one of: stdout, file, otel".to_string());
    }
    if config.events.classes.is_empty() {
        errors.push("events.classes must not be empty".to_string());
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::{
        validate_security_config, KeyManager, KeyRecord, SecurityAuditConfig, SecurityAuthConfig,
        SecurityAuthorizationConfig, SecurityConfig, SecurityEventConfig, SecurityIdentityConfig,
        SecurityKeyConfig, SecurityPolicy, SecurityPolicyRegistry, SecuritySecretsConfig,
        SecurityTransportConfig, StaticSecretsProvider,
    };
    use crate::domain::security::runtime::SecretsProvider;

    fn sample_config() -> SecurityConfig {
        SecurityConfig {
            schema_version: 1,
            identity: SecurityIdentityConfig {
                principal_source: "ingress".to_string(),
                trust_header: "x-atlas-principal".to_string(),
            },
            auth: SecurityAuthConfig {
                mode: "api-key".to_string(),
                required: true,
            },
            authorization: SecurityAuthorizationConfig {
                default_decision: "deny".to_string(),
                policy_source: "configs/security/policy.yaml".to_string(),
            },
            secrets: SecuritySecretsConfig {
                provider: "env".to_string(),
                references: vec!["ATLAS_API_KEY".to_string()],
            },
            keys: SecurityKeyConfig {
                active_key_id: "atlas-key-1".to_string(),
                key_ring: vec!["atlas-key-1".to_string(), "atlas-key-0".to_string()],
            },
            transport: SecurityTransportConfig {
                tls_required: true,
                min_tls_version: "1.2".to_string(),
            },
            audit: SecurityAuditConfig {
                enabled: true,
                sink: "stdout".to_string(),
            },
            events: SecurityEventConfig {
                classes: vec![
                    "auth.failure".to_string(),
                    "authorization.denied".to_string(),
                ],
            },
        }
    }

    #[test]
    fn security_config_validation_passes_for_valid_config() {
        let errors = validate_security_config(&sample_config());
        assert!(
            errors.is_empty(),
            "unexpected validation errors: {errors:?}"
        );
    }

    #[test]
    fn security_config_validation_fails_for_invalid_values() {
        let mut config = sample_config();
        config.auth.mode = "unknown".to_string();
        config.audit.sink = "bad".to_string();
        config.events.classes.clear();
        let errors = validate_security_config(&config);
        assert!(errors.len() >= 3);
    }

    #[test]
    fn static_secrets_provider_returns_secret_by_reference() {
        let mut provider = StaticSecretsProvider::default();
        provider
            .values
            .insert("ATLAS_API_KEY".to_string(), "abc123".to_string());
        assert_eq!(
            provider.get_secret("ATLAS_API_KEY"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn key_manager_rotates_active_key() {
        let mut manager = KeyManager::new(vec![
            KeyRecord {
                key_id: "atlas-key-0".to_string(),
                purpose: "signing".to_string(),
                active: true,
            },
            KeyRecord {
                key_id: "atlas-key-1".to_string(),
                purpose: "signing".to_string(),
                active: false,
            },
        ]);
        assert!(manager.rotate("atlas-key-1"));
        assert_eq!(
            manager.active_key().map(|key| key.key_id.clone()),
            Some("atlas-key-1".to_string())
        );
    }

    #[test]
    fn security_policy_registry_tracks_policies() {
        let mut registry = SecurityPolicyRegistry::new();
        registry.register(SecurityPolicy {
            policy_id: "sec-auth-required".to_string(),
            description: "authentication required for protected routes".to_string(),
            enabled: true,
        });
        registry.register(SecurityPolicy {
            policy_id: "sec-audit-log".to_string(),
            description: "audit logging on security events".to_string(),
            enabled: false,
        });
        assert_eq!(registry.list().len(), 2);
        assert_eq!(registry.enabled_count(), 1);
    }
}
