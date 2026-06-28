// SPDX-License-Identifier: Apache-2.0

use crate::packaged::{AUTH_POLICY_YAML, PERMISSIONS_YAML, ROLES_YAML};
use axum::http::StatusCode;
use bijux_atlas_api::ApiErrorCode;
use bijux_atlas_runtime::domain::security::authorization::{
    AuthorizationDecision, AuthorizationEngine, AuthorizationPolicy, PermissionCatalog,
    PermissionEvaluator, RoleCatalog, RoleRegistry,
};
use tracing::{error, info};

pub(super) fn route_auth_exempt(route: &str) -> bool {
    matches!(
        route,
        "/health"
            | "/healthz"
            | "/healthz/overload"
            | "/ready"
            | "/readyz"
            | "/live"
            | "/metrics"
            | "/v1/version"
            | "/v1/openapi.json"
    )
}

pub(super) fn route_is_admin_endpoint(route: &str) -> bool {
    matches!(
        route,
        "/debug/datasets"
            | "/debug/dataset-health"
            | "/debug/registry-health"
            | "/debug/diagnostics"
            | "/debug/runtime-stats"
            | "/debug/system-info"
            | "/debug/build-metadata"
            | "/debug/runtime-config"
            | "/debug/dataset-registry"
            | "/debug/shard-map"
            | "/debug/query-planner-stats"
            | "/debug/cache-stats"
            | "/debug/cluster/nodes"
            | "/debug/cluster-status"
            | "/debug/cluster/register"
            | "/debug/cluster/heartbeat"
            | "/debug/cluster/mode"
            | "/v1/_debug/echo"
    )
}

pub(super) fn route_action_id(route: &str) -> &'static str {
    if route_auth_exempt(route) {
        "catalog.read"
    } else if route_is_admin_endpoint(route) {
        "ops.admin"
    } else {
        "dataset.read"
    }
}

pub(super) fn route_resource_kind(route: &str) -> &'static str {
    if route_auth_exempt(route) || route_is_admin_endpoint(route) {
        "namespace"
    } else {
        "dataset-id"
    }
}

pub(super) fn parse_embedded_auth_policy(raw: &str) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(raw).map_err(|err| format!("embedded auth policy: {err}"))
}

pub(super) fn build_embedded_authorization_engine(
    permissions_raw: &str,
    roles_raw: &str,
    policy_raw: &str,
) -> Result<AuthorizationEngine, String> {
    let permissions: PermissionCatalog = serde_yaml::from_str(permissions_raw)
        .map_err(|err| format!("embedded permission catalog: {err}"))?;
    let roles: RoleCatalog =
        serde_yaml::from_str(roles_raw).map_err(|err| format!("embedded role catalog: {err}"))?;
    let policy: AuthorizationPolicy = serde_yaml::from_str(policy_raw)
        .map_err(|err| format!("embedded authorization policy: {err}"))?;
    let evaluator = PermissionEvaluator::new(permissions);
    let mut registry = RoleRegistry::new();
    for role in roles.roles {
        registry.upsert_role(role);
    }
    for (principal_id, role_id) in [
        ("user", "role.user.readonly"),
        ("service-account", "role.service.readonly"),
        ("operator", "role.operator.admin"),
        ("ci", "role.automation.release"),
    ] {
        registry.assign_role(principal_id, role_id);
    }
    Ok(AuthorizationEngine::new(registry, evaluator, policy))
}

pub(super) fn embedded_policy_allows(
    principal: &str,
    action: &str,
    resource_kind: &str,
    route: &str,
) -> bool {
    static POLICY: std::sync::OnceLock<Result<serde_yaml::Value, String>> =
        std::sync::OnceLock::new();
    let policy = match POLICY.get_or_init(|| parse_embedded_auth_policy(AUTH_POLICY_YAML)) {
        Ok(policy) => policy,
        Err(err) => {
            error!(
                event_id = "embedded_auth_policy_invalid",
                error = %err,
                "embedded auth policy is invalid; denying request"
            );
            return false;
        }
    };
    let default_allow = policy
        .get("default_decision")
        .and_then(serde_yaml::Value::as_str)
        .is_some_and(|value| value == "allow");
    let rules = policy
        .get("rules")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    for rule in rules {
        let principals = rule
            .get("principals")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if !principals
            .iter()
            .filter_map(serde_yaml::Value::as_str)
            .any(|value| value == principal)
        {
            continue;
        }
        let actions = rule
            .get("actions")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if !actions
            .iter()
            .filter_map(serde_yaml::Value::as_str)
            .any(|value| value == action)
        {
            continue;
        }
        let kinds = rule
            .get("resources")
            .and_then(|value| value.get("kinds"))
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if !kinds
            .iter()
            .filter_map(serde_yaml::Value::as_str)
            .any(|value| value == resource_kind)
        {
            continue;
        }
        let routes = rule
            .get("routes")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if !routes
            .iter()
            .filter_map(serde_yaml::Value::as_str)
            .any(|value| route.starts_with(value))
        {
            continue;
        }
        return rule
            .get("effect")
            .and_then(serde_yaml::Value::as_str)
            .is_some_and(|effect| effect == "allow");
    }
    default_allow
}

pub(super) fn embedded_authorization_allows(
    principal: &str,
    action: &str,
    resource_kind: &str,
    route: &str,
) -> bool {
    static ENGINE: std::sync::OnceLock<Result<AuthorizationEngine, String>> =
        std::sync::OnceLock::new();
    let engine = match ENGINE.get_or_init(|| {
        build_embedded_authorization_engine(PERMISSIONS_YAML, ROLES_YAML, AUTH_POLICY_YAML)
    }) {
        Ok(engine) => engine,
        Err(err) => {
            error!(
                event_id = "embedded_authorization_invalid",
                error = %err,
                "embedded authorization contracts are invalid; denying request"
            );
            return false;
        }
    };
    matches!(
        engine.evaluate(principal, action, resource_kind, route),
        AuthorizationDecision::Allow
    )
}

pub(super) fn emit_auth_policy_decision(
    auth_mode: bijux_atlas_runtime::runtime::config::AuthMode,
    principal: &str,
    route: &str,
    allowed: bool,
) {
    info!(
        event_id = "auth_policy_decision",
        event = "auth_policy_decision",
        auth_mode = auth_mode.as_str(),
        principal = principal,
        action = route_action_id(route),
        resource_kind = route_resource_kind(route),
        route = route,
        decision = if allowed { "allow" } else { "deny" },
        "auth policy decision"
    );
}

pub(super) fn auth_error_code(status: StatusCode) -> ApiErrorCode {
    match status {
        StatusCode::UNAUTHORIZED => ApiErrorCode::AuthenticationRequired,
        StatusCode::FORBIDDEN => ApiErrorCode::AccessForbidden,
        _ => ApiErrorCode::QueryRejectedByPolicy,
    }
}
