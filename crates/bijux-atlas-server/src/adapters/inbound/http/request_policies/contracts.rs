// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::packaged::{AUTH_POLICY_YAML, PERMISSIONS_YAML, ROLES_YAML};
use axum::http::HeaderMap;
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
    let policy_err = build_embedded_authorization_engine(PERMISSIONS_YAML, ROLES_YAML, "rules: [")
        .expect_err("bad policy");

    assert!(permissions_err.contains("embedded permission catalog"));
    assert!(roles_err.contains("embedded role catalog"));
    assert!(policy_err.contains("embedded authorization policy"));
}

fn signed_token(payload: serde_json::Value, secret: &str) -> String {
    let header =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&payload).unwrap_or_default());
    let signed = format!("{header}.{claims}");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .unwrap_or_else(|_| Hmac::<Sha256>::new_from_slice(b"default").expect("static hmac key"));
    mac.update(signed.as_bytes());
    let sig = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
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
