// SPDX-License-Identifier: Apache-2.0

use crate::app::server::observability::unix_time_millis;
use crate::app::server::state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bijux_atlas_api::{ApiError, ApiErrorCode};
use bijux_atlas_runtime::domain::security::data_protection::https_enforced;
use std::time::Instant;
use tracing::{info, warn};

mod audit;
mod authentication;
mod authorization;
mod transport;

use self::audit::*;
use self::authentication::*;
use self::authorization::*;
use self::transport::{
    classify_client_type, classify_user_agent_family, normalized_forwarded_for,
    normalized_header_value,
};
pub(crate) use self::transport::{
    cors_middleware, debug_route_hardening_middleware, provenance_headers_middleware,
    resilience_middleware,
};

fn chrono_like_unix_secs() -> u64 {
    (unix_time_millis() / 1000) as u64
}

async fn record_policy_violation(state: &AppState, policy: &str) {
    state.record_policy_violation(policy).await;
}

async fn record_auth_failure(state: &AppState, reason: &str, route: &str) {
    record_policy_violation(state, reason).await;
    let key = format!("auth.{reason}");
    let count = state.increment_policy_violation_bucket(&key).await;
    if count % 50 == 0 {
        warn!(
            event_id = "authentication_failure_alert",
            event = "authentication_failure_alert",
            route = route,
            reason = reason,
            count = count,
            "authentication failure threshold reached"
        );
    }
}

async fn record_authorization_denial(
    state: &AppState,
    route: &str,
    action: &str,
    resource_kind: &str,
) {
    record_policy_violation(state, "authorization.denied").await;
    let count = state
        .increment_policy_violation_bucket("authorization.denied")
        .await;
    warn!(
        event_id = "authorization_denied",
        event = "authorization_denied",
        route = route,
        action = action,
        resource_kind = resource_kind,
        denial_count = count,
        "authorization denied"
    );
}

#[allow(dead_code)] // ATLAS-EXC-0001
pub(crate) async fn record_invariant_violation(state: &AppState, invariant: &str) {
    state.record_invariant_violation(invariant).await;
}

pub(crate) async fn security_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let uri_text = req.uri().to_string();
    let route = req.uri().path().to_string();
    let request_id =
        crate::adapters::inbound::http::handlers::propagated_request_id(req.headers(), &state);
    info!(
        event_id = "authentication_evaluation_started",
        event = "authentication_evaluation_started",
        auth_mode = state.api.auth_mode.as_str(),
        route = route.as_str(),
        "authentication evaluation started"
    );
    let auth_exempt = route_auth_exempt(&route);
    if state.api.require_https {
        info!(
            event_id = "transport_https_policy",
            event = "transport_https_policy",
            route = route.as_str(),
            "evaluating https transport requirement"
        );
        let forwarded_proto = normalized_header_value(req.headers(), "x-forwarded-proto", 16);
        if !https_enforced(forwarded_proto.as_deref(), true) {
            record_policy_violation(&state, "https_required").await;
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
                "https is required",
                serde_json::json!({"class": "transport", "reason": "https_required"}),
                request_id.clone(),
            ));
            return crate::adapters::inbound::http::handlers::with_request_id(
                (StatusCode::UPGRADE_REQUIRED, err).into_response(),
                &request_id,
            );
        }
    }
    if route_is_admin_endpoint(&route) && !state.api.enable_admin_endpoints {
        emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
        let err = Json(ApiError::new(
            ApiErrorCode::DatasetNotFound,
            "admin endpoints are disabled",
            serde_json::json!({}),
            request_id.clone(),
        ));
        return crate::adapters::inbound::http::handlers::with_request_id(
            (StatusCode::NOT_FOUND, err).into_response(),
            &request_id,
        );
    }
    if uri_text.len() > state.api.max_uri_bytes {
        record_policy_violation(&state, "uri_bytes").await;
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "request URI too large",
            serde_json::json!({"max_uri_bytes": state.api.max_uri_bytes, "actual": uri_text.len()}),
            request_id.clone(),
        ));
        return crate::adapters::inbound::http::handlers::with_request_id(
            (StatusCode::BAD_REQUEST, err).into_response(),
            &request_id,
        );
    }
    if let Some(raw_query) = req.uri().query() {
        let query_params = raw_query.split('&').filter(|pair| !pair.is_empty()).count();
        if query_params > state.api.max_query_params {
            record_policy_violation(&state, "query_params").await;
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
                "query parameter count exceeds limit",
                serde_json::json!({
                    "max_query_params": state.api.max_query_params,
                    "actual": query_params
                }),
                request_id.clone(),
            ));
            return crate::adapters::inbound::http::handlers::with_request_id(
                (StatusCode::BAD_REQUEST, err).into_response(),
                &request_id,
            );
        }
    }
    let header_bytes: usize = req
        .headers()
        .iter()
        .map(|(k, v)| k.as_str().len() + v.as_bytes().len())
        .sum();
    state
        .metrics()
        .observe_request_size(&route, uri_text.len().saturating_add(header_bytes))
        .await;
    if header_bytes > state.api.max_header_bytes {
        record_policy_violation(&state, "header_bytes").await;
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "request headers too large",
            serde_json::json!({"max_header_bytes": state.api.max_header_bytes, "actual": header_bytes}),
            request_id.clone(),
        ));
        return crate::adapters::inbound::http::handlers::with_request_id(
            (StatusCode::BAD_REQUEST, err).into_response(),
            &request_id,
        );
    }

    let user_agent = normalized_header_value(req.headers(), "user-agent", 512);
    let client_type = classify_client_type(user_agent.as_deref());
    let ua_family = classify_user_agent_family(user_agent.as_deref());
    state
        .metrics()
        .observe_client_fingerprint(client_type, ua_family)
        .await;

    let api_key = normalized_header_value(req.headers(), "x-api-key", 512);
    let api_key_store = ApiKeyStore::from_allowed_entries(
        &state.api.allowed_api_keys,
        state.api.api_key_expiration_days,
    );
    if !auth_exempt && state.api.require_api_key && api_key.is_none() {
        emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
        record_auth_failure(&state, "api_key_required", &route).await;
        let err = Json(ApiError::new(
            auth_error_code(StatusCode::UNAUTHORIZED),
            "api key required",
            serde_json::json!({}),
            request_id.clone(),
        ));
        return crate::adapters::inbound::http::handlers::with_request_id(
            (StatusCode::UNAUTHORIZED, err).into_response(),
            &request_id,
        );
    }
    if let Some(key) = &api_key {
        if !api_key_store.is_empty()
            && api_key_store
                .validate(key, chrono_like_unix_secs())
                .is_err()
        {
            emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
            record_auth_failure(&state, "api_key_invalid", &route).await;
            let err = Json(ApiError::new(
                auth_error_code(StatusCode::UNAUTHORIZED),
                "invalid api key",
                serde_json::json!({}),
                request_id.clone(),
            ));
            return crate::adapters::inbound::http::handlers::with_request_id(
                (StatusCode::UNAUTHORIZED, err).into_response(),
                &request_id,
            );
        }
    }

    let token = token_header_value(req.headers());
    let token_context = if matches!(
        state.api.auth_mode,
        bijux_atlas_runtime::runtime::config::AuthMode::Token
    ) {
        let Some(raw_token) = token.as_deref() else {
            emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
            record_auth_failure(&state, "token_missing", &route).await;
            let err = Json(ApiError::new(
                auth_error_code(StatusCode::UNAUTHORIZED),
                "bearer token required",
                serde_json::json!({}),
                request_id.clone(),
            ));
            return crate::adapters::inbound::http::handlers::with_request_id(
                (StatusCode::UNAUTHORIZED, err).into_response(),
                &request_id,
            );
        };
        match validate_signed_token(raw_token, &state.api) {
            Ok(context) => Some(context),
            Err(err) => {
                emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
                record_auth_failure(&state, err.as_code(), &route).await;
                let err = Json(ApiError::new(
                    auth_error_code(StatusCode::UNAUTHORIZED),
                    "invalid bearer token",
                    serde_json::json!({"class": "authentication", "reason": err.as_code()}),
                    request_id.clone(),
                ));
                return crate::adapters::inbound::http::handlers::with_request_id(
                    (StatusCode::UNAUTHORIZED, err).into_response(),
                    &request_id,
                );
            }
        }
    } else {
        None
    };

    if let Some(secret) = &state.api.hmac_secret {
        let ts = normalized_header_value(req.headers(), "x-bijux-timestamp", 64);
        let sig = normalized_header_value(req.headers(), "x-bijux-signature", 128);
        if !auth_exempt && state.api.hmac_required && (ts.is_none() || sig.is_none()) {
            emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
            record_auth_failure(&state, "hmac_missing_headers", &route).await;
            let err = Json(ApiError::new(
                auth_error_code(StatusCode::UNAUTHORIZED),
                "missing required HMAC headers",
                serde_json::json!({}),
                request_id.clone(),
            ));
            return crate::adapters::inbound::http::handlers::with_request_id(
                (StatusCode::UNAUTHORIZED, err).into_response(),
                &request_id,
            );
        }
        if let (Some(ts_value), Some(sig_value)) = (ts, sig) {
            let now = unix_time_millis() / 1000;
            let Some(parsed_ts) = ts_value.parse::<u128>().ok() else {
                emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
                record_auth_failure(&state, "hmac_invalid_timestamp", &route).await;
                let err = Json(ApiError::new(
                    auth_error_code(StatusCode::UNAUTHORIZED),
                    "invalid hmac timestamp",
                    serde_json::json!({}),
                    request_id.clone(),
                ));
                return crate::adapters::inbound::http::handlers::with_request_id(
                    (StatusCode::UNAUTHORIZED, err).into_response(),
                    &request_id,
                );
            };
            let skew = now.abs_diff(parsed_ts);
            if skew > state.api.hmac_max_skew_secs as u128 {
                emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
                record_auth_failure(&state, "hmac_skew", &route).await;
                let err = Json(ApiError::new(
                    auth_error_code(StatusCode::UNAUTHORIZED),
                    "hmac timestamp outside allowed skew",
                    serde_json::json!({"max_skew_secs": state.api.hmac_max_skew_secs}),
                    request_id.clone(),
                ));
                return crate::adapters::inbound::http::handlers::with_request_id(
                    (StatusCode::UNAUTHORIZED, err).into_response(),
                    &request_id,
                );
            }
            let method = req.method().as_str();
            let uri = req.uri().path_and_query().map_or("", |pq| pq.as_str());
            if build_hmac_signature(secret, method, uri, &ts_value).as_deref()
                != Some(sig_value.as_str())
            {
                emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
                record_auth_failure(&state, "hmac_signature", &route).await;
                let err = Json(ApiError::new(
                    auth_error_code(StatusCode::UNAUTHORIZED),
                    "invalid hmac signature",
                    serde_json::json!({}),
                    request_id.clone(),
                ));
                return crate::adapters::inbound::http::handlers::with_request_id(
                    (StatusCode::UNAUTHORIZED, err).into_response(),
                    &request_id,
                );
            }
        }
    }

    let auth_context = if route_is_admin_endpoint(&route) {
        AuthenticationContext {
            principal: "operator",
            mechanism: "internal-admin",
            subject: "operator".to_string(),
            issuer: None,
            scopes: Vec::new(),
        }
    } else if auth_exempt
        || state.api.auth_mode == bijux_atlas_runtime::runtime::config::AuthMode::Disabled
    {
        AuthenticationContext {
            principal: "user",
            mechanism: "none",
            subject: "anonymous".to_string(),
            issuer: None,
            scopes: Vec::new(),
        }
    } else if let Some(context) = token_context {
        context
    } else if matches!(
        state.api.auth_mode,
        bijux_atlas_runtime::runtime::config::AuthMode::Oidc
            | bijux_atlas_runtime::runtime::config::AuthMode::Mtls
    ) {
        let Some(principal) = proxy_authenticated_principal(req.headers(), state.api.auth_mode)
        else {
            emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
            record_auth_failure(&state, "proxy_identity_missing", &route).await;
            let err = Json(ApiError::new(
                auth_error_code(StatusCode::UNAUTHORIZED),
                "trusted auth proxy identity header required",
                serde_json::json!({"auth_mode": state.api.auth_mode.as_str()}),
                request_id.clone(),
            ));
            return crate::adapters::inbound::http::handlers::with_request_id(
                (StatusCode::UNAUTHORIZED, err).into_response(),
                &request_id,
            );
        };
        AuthenticationContext {
            principal,
            mechanism: state.api.auth_mode.as_str(),
            subject: principal.to_string(),
            issuer: None,
            scopes: Vec::new(),
        }
    } else {
        AuthenticationContext {
            principal: "service-account",
            mechanism: "api-key",
            subject: "service-account".to_string(),
            issuer: None,
            scopes: Vec::new(),
        }
    };
    let principal = auth_context.principal;
    info!(
        event_id = "authorization_evaluation_started",
        event = "authorization_evaluation_started",
        principal = principal,
        action = route_action_id(&route),
        resource_kind = route_resource_kind(&route),
        route = route.as_str(),
        "authorization evaluation started"
    );
    let policy_allowed = embedded_authorization_allows(
        principal,
        route_action_id(&route),
        route_resource_kind(&route),
        &route,
    );
    let policy_allowed = policy_allowed
        && embedded_policy_allows(
            principal,
            route_action_id(&route),
            route_resource_kind(&route),
            &route,
        );
    info!(
        event_id = "authentication_context",
        event = "authentication_context",
        auth_mode = state.api.auth_mode.as_str(),
        mechanism = auth_context.mechanism,
        subject = auth_context.subject.as_str(),
        issuer = auth_context.issuer.as_deref().unwrap_or_default(),
        issuer_present = auth_context.issuer.is_some(),
        scope_count = auth_context.scopes.len(),
        route = route,
        "authentication context established"
    );
    emit_auth_policy_decision(state.api.auth_mode, principal, &route, policy_allowed);
    if !policy_allowed {
        record_authorization_denial(
            &state,
            &route,
            route_action_id(&route),
            route_resource_kind(&route),
        )
        .await;
        if state.api.audit.enabled {
            emit_audit_event(
                &state.api.audit,
                "authorization_denied",
                Some(principal),
                route_action_id(&route),
                route_resource_kind(&route),
                &route,
                &[
                    ("decision", "deny"),
                    ("reason", "policy_denied"),
                    ("route", route.as_str()),
                ],
            );
        }
        let err = Json(ApiError::new(
            auth_error_code(StatusCode::FORBIDDEN),
            "request denied by access policy",
            serde_json::json!({
                "action": route_action_id(&route),
                "resource_kind": route_resource_kind(&route)
            }),
            request_id.clone(),
        ));
        return crate::adapters::inbound::http::handlers::with_request_id(
            (StatusCode::FORBIDDEN, err).into_response(),
            &request_id,
        );
    }

    let started = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let request_id =
        normalized_header_value(req.headers(), "x-request-id", 128).unwrap_or_default();
    let client_ip = normalized_forwarded_for(req.headers());
    let resp = next.run(req).await;
    if state.api.audit.enabled {
        let event_name = if route_is_admin_endpoint(&path) {
            "admin_action"
        } else {
            "query_executed"
        };
        let status_text = resp.status().as_u16().to_string();
        let latency_ms = started.elapsed().as_millis().to_string();
        let mut audit_fields = vec![
            ("method", method.as_str()),
            ("status", status_text.as_str()),
            ("request_id", request_id.as_str()),
            ("latency_ms", latency_ms.as_str()),
        ];
        if let Some(client_ip) = client_ip.as_deref() {
            audit_fields.push(("client_ip", client_ip));
        }
        emit_audit_event(
            &state.api.audit,
            event_name,
            Some(principal),
            route_action_id(&path),
            route_resource_kind(&path),
            &path,
            &audit_fields,
        );
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packaged::{AUTH_POLICY_YAML, PERMISSIONS_YAML, ROLES_YAML};
    use axum::http::HeaderValue;
    use base64::Engine as _;
    use bijux_atlas_runtime::runtime::config::AuthMode;
    use hmac::{digest::KeyInit, Hmac, Mac};
    use sha2::Sha256;
    use std::time::Duration;

    #[test]
    fn health_endpoints_stay_auth_exempt_in_all_modes() {
        for mode in [
            AuthMode::Disabled,
            AuthMode::ApiKey,
            AuthMode::Token,
            AuthMode::Oidc,
            AuthMode::Mtls,
        ] {
            assert!(
                route_auth_exempt("/healthz"),
                "{mode:?} must allow /healthz"
            );
            assert!(route_auth_exempt("/readyz"), "{mode:?} must allow /readyz");
            assert!(
                route_auth_exempt("/v1/version"),
                "{mode:?} must allow /v1/version"
            );
            assert!(
                !route_auth_exempt("/v1/datasets"),
                "{mode:?} must not mark data routes as auth exempt"
            );
        }
    }

    #[test]
    fn protected_routes_use_auth_failure_codes() {
        assert!(!route_auth_exempt("/v1/datasets"));
        assert_eq!(
            auth_error_code(StatusCode::UNAUTHORIZED),
            ApiErrorCode::AuthenticationRequired
        );
        assert_eq!(
            auth_error_code(StatusCode::FORBIDDEN),
            ApiErrorCode::AccessForbidden
        );
    }

    #[test]
    fn proxy_modes_require_boundary_identity_headers() {
        let mut headers = HeaderMap::new();
        assert_eq!(
            proxy_authenticated_principal(&headers, AuthMode::Oidc),
            None
        );
        assert_eq!(
            proxy_authenticated_principal(&headers, AuthMode::Mtls),
            None
        );
        headers.insert("x-forwarded-user", HeaderValue::from_static("alice"));
        assert_eq!(
            proxy_authenticated_principal(&headers, AuthMode::Oidc),
            Some("user")
        );
        headers.clear();
        headers.insert(
            "x-atlas-mtls-subject",
            HeaderValue::from_static("spiffe://atlas/service"),
        );
        assert_eq!(
            proxy_authenticated_principal(&headers, AuthMode::Mtls),
            Some("service-account")
        );
    }

    #[test]
    fn audit_redaction_removes_known_secret_patterns() {
        assert_eq!(
            redacted_audit_field("authorization", "Bearer topsecret"),
            None
        );
        assert_eq!(
            redacted_audit_field("request_id", "Bearer topsecret"),
            Some("[REDACTED]".to_string())
        );
        assert_eq!(redacted_audit_field("client_ip", "127.0.0.1"), None);
    }

    #[test]
    fn audit_event_contains_required_fields() {
        let event = build_audit_event(
            "query_executed",
            Some("service-account"),
            "dataset.read",
            "dataset-id",
            "/v1/datasets",
            bijux_atlas_runtime::runtime::config::AuditSink::Stdout,
            &[("status", "200")],
        );
        assert_eq!(event["event_id"].as_str(), Some("audit_query_executed"));
        assert_eq!(
            event["timestamp_policy"].as_str(),
            Some("runtime-unix-seconds")
        );
        assert_eq!(event["principal"].as_str(), Some("service-account"));
        assert_eq!(event["action"].as_str(), Some("dataset.read"));
        assert_eq!(event["resource_kind"].as_str(), Some("dataset-id"));
        assert_eq!(event["status"].as_str(), Some("200"));
        assert!(event["timestamp_unix_s"].as_u64().is_some());
    }

    #[test]
    fn audit_event_drops_unknown_or_sensitive_dynamic_fields() {
        let event = build_audit_event(
            "query_executed",
            Some("service-account"),
            "dataset.read",
            "dataset-id",
            "/v1/datasets",
            bijux_atlas_runtime::runtime::config::AuditSink::Stdout,
            &[
                ("status", "200"),
                ("authorization", "Bearer topsecret"),
                ("unknown_field", "should-not-appear"),
            ],
        );
        assert_eq!(event["status"].as_str(), Some("200"));
        assert!(event.get("authorization").is_none());
        assert!(event.get("unknown_field").is_none());
    }

    #[test]
    fn embedded_authorization_enforces_operator_admin_boundary() {
        assert!(embedded_authorization_allows(
            "operator",
            "ops.admin",
            "namespace",
            "/debug/runtime-config"
        ));
        assert!(!embedded_authorization_allows(
            "user",
            "ops.admin",
            "namespace",
            "/debug/runtime-config"
        ));
    }

    #[test]
    fn https_enforcement_requires_https_proto_header() {
        assert!(https_enforced(Some("https"), true));
        assert!(!https_enforced(Some("http"), true));
    }

    #[test]
    fn invalid_embedded_auth_policy_is_rejected_without_panicking() {
        let err = parse_embedded_auth_policy("default_decision: [").expect_err("invalid yaml");
        assert!(err.contains("embedded auth policy"));
    }

    #[test]
    fn invalid_embedded_authorization_contracts_fail_closed_without_panicking() {
        let permissions_err =
            build_embedded_authorization_engine("permissions: [", ROLES_YAML, AUTH_POLICY_YAML)
                .expect_err("bad permissions");
        let roles_err =
            build_embedded_authorization_engine(PERMISSIONS_YAML, "roles: [", AUTH_POLICY_YAML)
                .expect_err("bad roles");
        let policy_err =
            build_embedded_authorization_engine(PERMISSIONS_YAML, ROLES_YAML, "rules: [")
                .expect_err("bad policy");

        assert!(permissions_err.contains("embedded permission catalog"));
        assert!(roles_err.contains("embedded role catalog"));
        assert!(policy_err.contains("embedded authorization policy"));
    }

    fn signed_token(payload: serde_json::Value, secret: &str) -> String {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&payload).unwrap_or_default());
        let signed = format!("{header}.{claims}");
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap_or_else(|_| {
            Hmac::<Sha256>::new_from_slice(b"default").expect("static hmac key")
        });
        mac.update(signed.as_bytes());
        let sig =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        format!("{signed}.{sig}")
    }

    #[test]
    fn generated_api_keys_are_hashed_and_unique() {
        let left = generate_api_key("integration");
        let right = generate_api_key("integration");
        assert!(left.starts_with("atlas_"));
        assert!(right.starts_with("atlas_"));
        assert_ne!(left, right);
        assert_eq!(hash_api_key("alpha").len(), 64);
    }

    #[test]
    fn api_key_store_enforces_expiration_rotation_and_revocation() {
        let now = chrono_like_unix_secs();
        let active = generate_api_key("active");
        let future = generate_api_key("future");
        let revoked = generate_api_key("revoked");
        let store = ApiKeyStore::from_allowed_entries(
            &[
                active.clone(),
                format!(
                    "hash={}|not_before={}",
                    hash_api_key(&future),
                    now.saturating_add(60)
                ),
                format!("hash={}|revoked=true", hash_api_key(&revoked)),
                format!(
                    "hash={}|expires={}",
                    hash_api_key("expired"),
                    now.saturating_sub(1)
                ),
            ],
            90,
        );
        assert!(store.validate(&active, now).is_ok());
        assert_eq!(
            store.validate(&future, now),
            Err(ApiKeyValidationError::NotYetValid)
        );
        assert_eq!(
            store.validate("expired", now),
            Err(ApiKeyValidationError::Expired)
        );
        assert_eq!(
            store.validate(&revoked, now),
            Err(ApiKeyValidationError::Revoked)
        );
    }

    #[test]
    fn token_validation_enforces_expiry_scope_issuer_audience_and_revocation() {
        let now = chrono_like_unix_secs();
        let mut api = bijux_atlas_runtime::runtime::config::ApiConfig {
            token_signing_secret: Some("token-secret".to_string()),
            token_required_issuer: Some("atlas-auth".to_string()),
            token_required_audience: Some("atlas-api".to_string()),
            token_required_scopes: vec!["dataset.read".to_string()],
            ..bijux_atlas_runtime::runtime::config::ApiConfig::default()
        };
        let token = signed_token(
            serde_json::json!({
                "sub":"user-1",
                "iss":"atlas-auth",
                "aud":"atlas-api",
                "exp": now + 60,
                "nbf": now - 1,
                "jti":"token-1",
                "scope":"dataset.read ops.admin"
            }),
            "token-secret",
        );
        let ctx = validate_signed_token(&token, &api).expect("valid token");
        assert_eq!(ctx.principal, "user");
        assert_eq!(ctx.subject, "user-1");
        assert!(ctx.scopes.iter().any(|value| value == "dataset.read"));

        let expired = signed_token(
            serde_json::json!({
                "iss":"atlas-auth","aud":"atlas-api","exp": now - 1
            }),
            "token-secret",
        );
        assert_eq!(
            validate_signed_token(&expired, &api),
            Err(TokenValidationError::Expired)
        );

        api.token_revoked_ids = vec!["token-1".to_string()];
        assert_eq!(
            validate_signed_token(&token, &api),
            Err(TokenValidationError::Revoked)
        );
    }

    #[test]
    fn token_validation_rejects_malformed_tokens() {
        let api = bijux_atlas_runtime::runtime::config::ApiConfig {
            token_signing_secret: Some("token-secret".to_string()),
            ..bijux_atlas_runtime::runtime::config::ApiConfig::default()
        };
        assert_eq!(
            validate_signed_token("not.a.jwt", &api),
            Err(TokenValidationError::Malformed)
        );
    }

    #[test]
    fn token_validation_requires_a_non_empty_subject() {
        let now = chrono_like_unix_secs();
        let api = bijux_atlas_runtime::runtime::config::ApiConfig {
            token_signing_secret: Some("token-secret".to_string()),
            ..bijux_atlas_runtime::runtime::config::ApiConfig::default()
        };
        let missing_subject = signed_token(
            serde_json::json!({
                "iss":"atlas-auth",
                "aud":"atlas-api",
                "exp": now + 60,
                "nbf": now - 1,
                "scope":"dataset.read"
            }),
            "token-secret",
        );
        let empty_subject = signed_token(
            serde_json::json!({
                "sub":"",
                "iss":"atlas-auth",
                "aud":"atlas-api",
                "exp": now + 60,
                "nbf": now - 1,
                "scope":"dataset.read"
            }),
            "token-secret",
        );

        assert_eq!(
            validate_signed_token(&missing_subject, &api),
            Err(TokenValidationError::Malformed)
        );
        assert_eq!(
            validate_signed_token(&empty_subject, &api),
            Err(TokenValidationError::Malformed)
        );
    }

    #[test]
    fn authentication_validation_performance_is_bounded() {
        let now = chrono_like_unix_secs();
        let api_key = generate_api_key("perf");
        let store = ApiKeyStore::from_allowed_entries(std::slice::from_ref(&api_key), 90);
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = store.validate(&api_key, now);
        }
        // Keep this as a coarse regression guard rather than a tight microbenchmark.
        // Debug/profile variance and shared CI runners can exceed 100ms without
        // indicating a meaningful algorithmic regression.
        assert!(start.elapsed() < Duration::from_millis(500));
    }
}
