// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionDefinition {
    pub id: String,
    pub action: String,
    pub resource_kind: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub id: String,
    pub description: String,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub inherits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationPolicyRule {
    pub id: String,
    pub effect: String,
    pub principals: Vec<String>,
    pub actions: Vec<String>,
    pub resources: AuthorizationResources,
    pub routes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationResources {
    pub kinds: Vec<String>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationPolicy {
    pub schema_version: u64,
    pub default_decision: String,
    pub rules: Vec<AuthorizationPolicyRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionCatalog {
    pub schema_version: u64,
    pub permissions: Vec<PermissionDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleCatalog {
    pub schema_version: u64,
    pub roles: Vec<RoleDefinition>,
}

#[derive(Debug, Clone, Default)]
pub struct RoleRegistry {
    roles: BTreeMap<String, RoleDefinition>,
    principal_roles: BTreeMap<String, BTreeSet<String>>,
}

impl RoleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_role(&mut self, role: RoleDefinition) {
        self.roles.insert(role.id.clone(), role);
    }

    pub fn assign_role(&mut self, principal: &str, role_id: &str) {
        self.principal_roles
            .entry(principal.to_string())
            .or_default()
            .insert(role_id.to_string());
    }

    #[must_use]
    pub fn roles_for_principal(&self, principal: &str) -> BTreeSet<String> {
        self.principal_roles
            .get(principal)
            .cloned()
            .unwrap_or_default()
    }

    fn collect_effective_roles(&self, assigned: &BTreeSet<String>) -> BTreeSet<String> {
        let mut effective = BTreeSet::new();
        let mut queue = assigned.iter().cloned().collect::<Vec<_>>();
        while let Some(role_id) = queue.pop() {
            if !effective.insert(role_id.clone()) {
                continue;
            }
            if let Some(role) = self.roles.get(&role_id) {
                for inherited in &role.inherits {
                    queue.push(inherited.clone());
                }
            }
        }
        effective
    }

    fn resolve_permissions(&self, role_ids: &BTreeSet<String>) -> BTreeSet<String> {
        let mut permissions = BTreeSet::new();
        let effective = self.collect_effective_roles(role_ids);
        for role_id in effective {
            if let Some(role) = self.roles.get(&role_id) {
                for permission in &role.permissions {
                    permissions.insert(permission.clone());
                }
            }
        }
        permissions
    }
}

#[derive(Debug, Clone)]
pub struct PermissionEvaluator {
    by_id: BTreeMap<String, PermissionDefinition>,
}

impl PermissionEvaluator {
    #[must_use]
    pub fn new(catalog: PermissionCatalog) -> Self {
        let by_id = catalog
            .permissions
            .into_iter()
            .map(|permission| (permission.id.clone(), permission))
            .collect();
        Self { by_id }
    }

    #[must_use]
    pub fn has_matching_permission(
        &self,
        permission_ids: &BTreeSet<String>,
        action: &str,
        resource_kind: &str,
    ) -> bool {
        permission_ids.iter().any(|id| {
            self.by_id.get(id).is_some_and(|permission| {
                permission.action == action && permission.resource_kind == resource_kind
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
}

#[derive(Debug, Clone)]
pub struct AuthorizationEngine {
    role_registry: RoleRegistry,
    permission_evaluator: PermissionEvaluator,
    policy: AuthorizationPolicy,
}

impl AuthorizationEngine {
    #[must_use]
    pub fn new(
        role_registry: RoleRegistry,
        permission_evaluator: PermissionEvaluator,
        policy: AuthorizationPolicy,
    ) -> Self {
        Self {
            role_registry,
            permission_evaluator,
            policy,
        }
    }

    #[must_use]
    pub fn evaluate(
        &self,
        principal: &str,
        action: &str,
        resource_kind: &str,
        route: &str,
    ) -> AuthorizationDecision {
        let assigned = self.role_registry.roles_for_principal(principal);
        let permissions = self.role_registry.resolve_permissions(&assigned);
        if !self
            .permission_evaluator
            .has_matching_permission(&permissions, action, resource_kind)
        {
            return AuthorizationDecision::Deny;
        }

        for rule in &self.policy.rules {
            let principal_match = rule.principals.iter().any(|value| value == principal);
            let action_match = rule.actions.iter().any(|value| value == action);
            let resource_match = rule
                .resources
                .kinds
                .iter()
                .any(|value| value == resource_kind);
            let route_match = rule.routes.iter().any(|prefix| route.starts_with(prefix));
            if principal_match && action_match && resource_match && route_match {
                return if rule.effect == "allow" {
                    AuthorizationDecision::Allow
                } else {
                    AuthorizationDecision::Deny
                };
            }
        }

        if self.policy.default_decision == "allow" {
            AuthorizationDecision::Allow
        } else {
            AuthorizationDecision::Deny
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuthorizationDecision, AuthorizationEngine, AuthorizationPolicy, AuthorizationPolicyRule,
        AuthorizationResources, PermissionCatalog, PermissionDefinition, PermissionEvaluator,
        RoleCatalog, RoleDefinition, RoleRegistry,
    };

    fn permission_catalog() -> PermissionCatalog {
        PermissionCatalog {
            schema_version: 1,
            permissions: vec![
                PermissionDefinition {
                    id: "perm.catalog.read".to_string(),
                    action: "catalog.read".to_string(),
                    resource_kind: "namespace".to_string(),
                    description: "Read catalog".to_string(),
                },
                PermissionDefinition {
                    id: "perm.dataset.read".to_string(),
                    action: "dataset.read".to_string(),
                    resource_kind: "dataset-id".to_string(),
                    description: "Read dataset".to_string(),
                },
                PermissionDefinition {
                    id: "perm.ops.admin".to_string(),
                    action: "ops.admin".to_string(),
                    resource_kind: "namespace".to_string(),
                    description: "Admin operations".to_string(),
                },
            ],
        }
    }

    fn role_catalog() -> RoleCatalog {
        RoleCatalog {
            schema_version: 1,
            roles: vec![
                RoleDefinition {
                    id: "role.user.readonly".to_string(),
                    description: "User readonly role".to_string(),
                    permissions: vec![
                        "perm.catalog.read".to_string(),
                        "perm.dataset.read".to_string(),
                    ],
                    inherits: Vec::new(),
                },
                RoleDefinition {
                    id: "role.operator.admin".to_string(),
                    description: "Operator admin role".to_string(),
                    permissions: vec!["perm.ops.admin".to_string()],
                    inherits: vec!["role.user.readonly".to_string()],
                },
            ],
        }
    }

    fn policy() -> AuthorizationPolicy {
        AuthorizationPolicy {
            schema_version: 1,
            default_decision: "deny".to_string(),
            rules: vec![
                AuthorizationPolicyRule {
                    id: "AUTHZ-CATALOG-READ".to_string(),
                    effect: "allow".to_string(),
                    principals: vec!["user".to_string(), "operator".to_string()],
                    actions: vec!["catalog.read".to_string()],
                    resources: AuthorizationResources {
                        kinds: vec!["namespace".to_string()],
                        values: vec!["*".to_string()],
                    },
                    routes: vec!["/v1/datasets".to_string(), "/healthz".to_string()],
                },
                AuthorizationPolicyRule {
                    id: "AUTHZ-OPS-ADMIN".to_string(),
                    effect: "allow".to_string(),
                    principals: vec!["operator".to_string()],
                    actions: vec!["ops.admin".to_string()],
                    resources: AuthorizationResources {
                        kinds: vec!["namespace".to_string()],
                        values: vec!["*".to_string()],
                    },
                    routes: vec!["/debug".to_string()],
                },
            ],
        }
    }

    #[test]
    fn authorization_engine_denies_principal_without_permission() {
        let evaluator = PermissionEvaluator::new(permission_catalog());
        let mut registry = RoleRegistry::new();
        for role in role_catalog().roles {
            registry.upsert_role(role);
        }
        registry.assign_role("user", "role.user.readonly");
        let engine = AuthorizationEngine::new(registry, evaluator, policy());

        assert_eq!(
            engine.evaluate("user", "ops.admin", "namespace", "/debug/runtime-config"),
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn authorization_engine_supports_role_inheritance_and_multi_role_resolution() {
        let evaluator = PermissionEvaluator::new(permission_catalog());
        let mut registry = RoleRegistry::new();
        for role in role_catalog().roles {
            registry.upsert_role(role);
        }
        registry.assign_role("operator", "role.operator.admin");
        registry.assign_role("operator", "role.user.readonly");
        let engine = AuthorizationEngine::new(registry, evaluator, policy());

        assert_eq!(
            engine.evaluate("operator", "catalog.read", "namespace", "/v1/datasets"),
            AuthorizationDecision::Allow
        );
        assert_eq!(
            engine.evaluate(
                "operator",
                "ops.admin",
                "namespace",
                "/debug/runtime-config"
            ),
            AuthorizationDecision::Allow
        );
    }

    #[test]
    fn authorization_engine_respects_default_deny() {
        let evaluator = PermissionEvaluator::new(permission_catalog());
        let mut registry = RoleRegistry::new();
        for role in role_catalog().roles {
            registry.upsert_role(role);
        }
        registry.assign_role("user", "role.user.readonly");
        let engine = AuthorizationEngine::new(registry, evaluator, policy());

        assert_eq!(
            engine.evaluate("user", "dataset.read", "dataset-id", "/unknown/route"),
            AuthorizationDecision::Deny
        );
    }
}
