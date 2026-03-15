// SPDX-License-Identifier: Apache-2.0

use crate::app::server::state::AppState;
use crate::contracts::api::{ApiError, ApiErrorCode};
use crate::domain::dataset::DatasetId;
use crate::domain::security::authorization::{
    AuthorizationDecision, AuthorizationEngine, AuthorizationPolicy, PermissionCatalog,
    PermissionEvaluator, RoleCatalog, RoleRegistry,
};
use crate::domain::security::data_protection::https_enforced;
use base64::Engine as _;
use crate::sha256_hex;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tracing::{error, info, warn};

pub fn chrono_like_unix_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

fn chrono_like_unix_secs() -> u64 {
    (chrono_like_unix_millis() / 1000) as u64
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthenticationContext {
    principal: &'static str,
    mechanism: &'static str,
    subject: String,
    issuer: Option<String>,
    scopes: Vec<String>,
}

#[derive(Debug, Clone)]
struct ApiKeyRecord {
    key_hash: String,
    not_before_unix_s: Option<u64>,
    expires_unix_s: Option<u64>,
    revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ApiKeyValidationError {
    Unknown,
    NotYetValid,
    Expired,
    Revoked,
}

#[derive(Debug, Clone)]
struct ApiKeyStore {
    records: Vec<ApiKeyRecord>,
}

impl ApiKeyStore {
    fn from_allowed_entries(entries: &[String], expiration_days: u64) -> Self {
        let now = chrono_like_unix_secs();
        let expires_unix_s = now.saturating_add(expiration_days.saturating_mul(86_400));
        let mut records = Vec::new();
        for entry in entries {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parsed = parse_api_key_record_line(trimmed).unwrap_or_else(|| ApiKeyRecord {
                key_hash: hash_api_key(trimmed),
                not_before_unix_s: None,
                expires_unix_s: Some(expires_unix_s),
                revoked: false,
            });
            records.push(parsed);
        }
        Self { records }
    }

    fn validate(&self, raw_key: &str, now_unix_s: u64) -> Result<(), ApiKeyValidationError> {
        let candidate_hash = hash_api_key(raw_key);
        let Some(record) = self.records.iter().find(|item| item.key_hash == candidate_hash) else {
            return Err(ApiKeyValidationError::Unknown);
        };
        if record.revoked {
            return Err(ApiKeyValidationError::Revoked);
        }
        if let Some(not_before) = record.not_before_unix_s {
            if now_unix_s < not_before {
                return Err(ApiKeyValidationError::NotYetValid);
            }
        }
        if let Some(expires) = record.expires_unix_s {
            if now_unix_s > expires {
                return Err(ApiKeyValidationError::Expired);
            }
        }
        Ok(())
    }
}

fn hash_api_key(raw_key: &str) -> String {
    sha256_hex(raw_key.as_bytes())
}

fn parse_api_key_record_line(input: &str) -> Option<ApiKeyRecord> {
    if !input.starts_with("hash=") {
        return None;
    }
    let mut hash = None;
    let mut not_before_unix_s = None;
    let mut expires_unix_s = None;
    let mut revoked = false;
    for part in input.split('|') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next()?;
        let value = kv.next().unwrap_or_default();
        match key {
            "hash" => hash = Some(value.to_string()),
            "not_before" => not_before_unix_s = value.parse::<u64>().ok(),
            "expires" => expires_unix_s = value.parse::<u64>().ok(),
            "revoked" => revoked = value.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }
    let key_hash = hash?;
    if key_hash.len() != 64 || !key_hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    Some(ApiKeyRecord {
        key_hash,
        not_before_unix_s,
        expires_unix_s,
        revoked,
    })
}

#[allow(dead_code)]
fn generate_api_key(subject: &str) -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let now = chrono_like_unix_millis();
    let sequence = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let seed = format!("{subject}:{now}:{sequence}:{}", std::process::id());
    let material = sha256_hex(seed.as_bytes());
    format!("atlas_{material}")
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TokenClaims {
    sub: Option<String>,
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<u64>,
    nbf: Option<u64>,
    jti: Option<String>,
    scope: Option<String>,
    scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenValidationError {
    Malformed,
    Signature,
    Expired,
    NotYetValid,
    Issuer,
    Audience,
    Scope,
    Revoked,
}

impl TokenValidationError {
    const fn as_code(&self) -> &'static str {
        match self {
            Self::Malformed => "token_malformed",
            Self::Signature => "token_signature_invalid",
            Self::Expired => "token_expired",
            Self::NotYetValid => "token_not_yet_valid",
            Self::Issuer => "token_issuer_invalid",
            Self::Audience => "token_audience_invalid",
            Self::Scope => "token_scope_missing",
            Self::Revoked => "token_revoked",
        }
    }
}

fn token_header_value(headers: &HeaderMap) -> Option<String> {
    let raw = normalized_header_value(headers, "authorization", 4096)?;
    let mut parts = raw.splitn(2, ' ');
    let scheme = parts.next().unwrap_or_default();
    let token = parts.next().unwrap_or_default();
    if !scheme.eq_ignore_ascii_case("bearer") || token.trim().is_empty() {
        return None;
    }
    Some(token.trim().to_string())
}

fn validate_signed_token(
    token: &str,
    api: &crate::runtime::config::ApiConfig,
) -> Result<AuthenticationContext, TokenValidationError> {
    let Some(secret) = api.token_signing_secret.as_deref() else {
        return Err(TokenValidationError::Malformed);
    };
    let mut parts = token.split('.');
    let (Some(header_b64), Some(payload_b64), Some(sig_b64), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(TokenValidationError::Malformed);
    };
    let signed_content = format!("{header_b64}.{payload_b64}");
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| TokenValidationError::Malformed)?;
    mac.update(signed_content.as_bytes());
    let expected = mac.finalize().into_bytes();
    let parsed_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| TokenValidationError::Malformed)?;
    if parsed_sig != expected.as_slice() {
        return Err(TokenValidationError::Signature);
    }
    let claims_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| TokenValidationError::Malformed)?;
    let claims: TokenClaims =
        serde_json::from_slice(&claims_json).map_err(|_| TokenValidationError::Malformed)?;
    let now = chrono_like_unix_secs();
    if let Some(nbf) = claims.nbf {
        if now < nbf {
            return Err(TokenValidationError::NotYetValid);
        }
    }
    if let Some(exp) = claims.exp {
        if now >= exp {
            return Err(TokenValidationError::Expired);
        }
    }
    if let Some(required) = api.token_required_issuer.as_deref() {
        if claims.iss.as_deref() != Some(required) {
            return Err(TokenValidationError::Issuer);
        }
    }
    if let Some(required) = api.token_required_audience.as_deref() {
        if claims.aud.as_deref() != Some(required) {
            return Err(TokenValidationError::Audience);
        }
    }
    if let Some(jti) = claims.jti.as_deref() {
        if api.token_revoked_ids.iter().any(|value| value == jti) {
            return Err(TokenValidationError::Revoked);
        }
    }
    let mut scopes = claims.scopes.unwrap_or_default();
    if let Some(scope_text) = claims.scope {
        for scope in scope_text.split(' ') {
            let normalized = scope.trim();
            if !normalized.is_empty() && !scopes.iter().any(|value| value == normalized) {
                scopes.push(normalized.to_string());
            }
        }
    }
    for required in &api.token_required_scopes {
        if !scopes.iter().any(|scope| scope == required) {
            return Err(TokenValidationError::Scope);
        }
    }
    let Some(subject) = claims.sub.filter(|value| !value.trim().is_empty()) else {
        return Err(TokenValidationError::Malformed);
    };
    Ok(AuthenticationContext {
        principal: "user",
        mechanism: "token",
        subject,
        issuer: claims.iss,
        scopes,
    })
}

pub fn route_sli_class(route: &str) -> &'static str {
    if matches!(
        route,
        "/health"
            | "/healthz"
            | "/ready"
            | "/readyz"
            | "/live"
            | "/metrics"
            | "/v1/version"
    ) {
        return "cheap";
    }
    if route.contains("/diff") || route.contains("/region") || route.contains("/sequence") {
        return "heavy";
    }
    "standard"
}

fn route_auth_exempt(route: &str) -> bool {
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

fn route_is_admin_endpoint(route: &str) -> bool {
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

fn route_action_id(route: &str) -> &'static str {
    if route_auth_exempt(route) {
        "catalog.read"
    } else if route_is_admin_endpoint(route) {
        "ops.admin"
    } else {
        "dataset.read"
    }
}

fn route_resource_kind(route: &str) -> &'static str {
    if route_auth_exempt(route) || route_is_admin_endpoint(route) {
        "namespace"
    } else {
        "dataset-id"
    }
}

fn parse_embedded_auth_policy(raw: &str) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(raw).map_err(|err| format!("embedded auth policy: {err}"))
}

fn build_embedded_authorization_engine(
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
    Ok(AuthorizationEngine::new(
        registry, evaluator, policy,
    ))
}

fn embedded_policy_allows(
    principal: &str,
    action: &str,
    resource_kind: &str,
    route: &str,
) -> bool {
    const EMBEDDED_AUTH_POLICY: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs/security/policy.yaml"));
    static POLICY: std::sync::OnceLock<Result<serde_yaml::Value, String>> =
        std::sync::OnceLock::new();
    let policy = match POLICY.get_or_init(|| parse_embedded_auth_policy(EMBEDDED_AUTH_POLICY)) {
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

fn embedded_authorization_allows(
    principal: &str,
    action: &str,
    resource_kind: &str,
    route: &str,
) -> bool {
    const EMBEDDED_PERMISSIONS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../configs/security/permissions.yaml"
    ));
    const EMBEDDED_ROLES: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs/security/roles.yaml"));
    const EMBEDDED_AUTHZ_POLICY: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs/security/policy.yaml"));
    static ENGINE: std::sync::OnceLock<Result<AuthorizationEngine, String>> =
        std::sync::OnceLock::new();
    let engine = match ENGINE.get_or_init(|| {
        build_embedded_authorization_engine(
            EMBEDDED_PERMISSIONS,
            EMBEDDED_ROLES,
            EMBEDDED_AUTHZ_POLICY,
        )
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

fn emit_auth_policy_decision(
    auth_mode: crate::runtime::config::AuthMode,
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

fn auth_error_code(status: StatusCode) -> ApiErrorCode {
    match status {
        StatusCode::UNAUTHORIZED => ApiErrorCode::AuthenticationRequired,
        StatusCode::FORBIDDEN => ApiErrorCode::AccessForbidden,
        _ => ApiErrorCode::QueryRejectedByPolicy,
    }
}

fn redacted_audit_field(key: &str, value: &str) -> Option<String> {
    let normalized_key = key.to_ascii_lowercase();
    if [
        "authorization",
        "token",
        "api_key",
        "api-key",
        "signature",
        "secret",
        "email",
        "client_ip",
    ]
    .iter()
    .any(|needle| normalized_key.contains(needle))
    {
        return None;
    }
    let normalized_value = value.to_ascii_lowercase();
    if normalized_value.contains("bearer ")
        || normalized_value.contains("x-api-key")
        || normalized_value.contains('@')
    {
        return Some("[REDACTED]".to_string());
    }
    Some(value.to_string())
}

fn audit_dynamic_field_allowed(key: &str) -> bool {
    matches!(
        key,
        "status"
            | "decision"
            | "reason"
            | "route"
            | "source"
            | "outcome"
            | "auth_mode"
            | "admin_endpoints_enabled"
            | "audit_enabled"
            | "catalog_configured"
    )
}

fn build_audit_event(
    event_name: &str,
    principal: Option<&str>,
    action: &str,
    resource_kind: &str,
    resource_id: &str,
    sink: crate::runtime::config::AuditSink,
    fields: &[(&str, &str)],
) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert(
        "event_id".to_string(),
        serde_json::Value::String(format!("audit_{event_name}")),
    );
    object.insert(
        "event_name".to_string(),
        serde_json::Value::String(event_name.to_string()),
    );
    object.insert(
        "timestamp_policy".to_string(),
        serde_json::Value::String("runtime-unix-seconds".to_string()),
    );
    object.insert(
        "timestamp_unix_s".to_string(),
        serde_json::Value::Number(serde_json::Number::from(chrono_like_unix_secs())),
    );
    object.insert(
        "sink".to_string(),
        serde_json::Value::String(sink.as_str().to_string()),
    );
    if let Some(value) = principal {
        if let Some(redacted) = redacted_audit_field("principal", value) {
            object.insert("principal".to_string(), serde_json::Value::String(redacted));
        }
    }
    object.insert(
        "action".to_string(),
        serde_json::Value::String(action.to_string()),
    );
    object.insert(
        "resource_kind".to_string(),
        serde_json::Value::String(resource_kind.to_string()),
    );
    if let Some(redacted) = redacted_audit_field("resource_id", resource_id) {
        object.insert("resource_id".to_string(), serde_json::Value::String(redacted));
    }
    for (key, value) in fields {
        if !audit_dynamic_field_allowed(key) {
            continue;
        }
        if let Some(redacted) = redacted_audit_field(key, value) {
            object.insert(
                (*key).to_string(),
                serde_json::Value::String(redacted),
            );
        }
    }
    serde_json::Value::Object(object)
}

fn emit_audit_event(
    audit: &crate::runtime::config::AuditConfig,
    event_name: &str,
    principal: Option<&str>,
    action: &str,
    resource_kind: &str,
    resource_id: &str,
    fields: &[(&str, &str)],
) {
    let payload = build_audit_event(
        event_name,
        principal,
        action,
        resource_kind,
        resource_id,
        audit.sink,
        fields,
    );
    if matches!(audit.sink, crate::runtime::config::AuditSink::File) {
        let _ = crate::adapters::outbound::fs::write_audit_file_record(
            &audit.file_path,
            audit.max_bytes,
            &payload,
        );
    }
    info!(
        target: "atlas_audit",
        event_id = format!("audit_{event_name}"),
        audit_payload = %payload,
        "audit event"
    );
}

fn parse_dataset_from_uri(uri: &Uri) -> Option<DatasetId> {
    let path = uri.path();
    let mut release: Option<String> = None;
    let mut species: Option<String> = None;
    let mut assembly: Option<String> = None;

    if let Some(q) = uri.query() {
        for part in q.split('&') {
            let mut kv = part.splitn(2, '=');
            let k = kv.next().unwrap_or_default();
            let v = kv.next().unwrap_or_default();
            match k {
                "release" => release = Some(v.to_string()),
                "species" => species = Some(v.to_string()),
                "assembly" => assembly = Some(v.to_string()),
                _ => {}
            }
        }
    }

    if release.is_none() || species.is_none() || assembly.is_none() {
        let seg: Vec<&str> = path.split('/').collect();
        if seg.len() >= 8 && seg.get(1) == Some(&"v1") && seg.get(2) == Some(&"releases") {
            release = seg.get(3).map(|x| (*x).to_string());
            if seg.get(4) == Some(&"species") {
                species = seg.get(5).map(|x| (*x).to_string());
            }
            if seg.get(6) == Some(&"assemblies") {
                assembly = seg.get(7).map(|x| (*x).to_string());
            }
        }
    }

    DatasetId::new(
        release.as_deref().unwrap_or_default(),
        species.as_deref().unwrap_or_default(),
        assembly.as_deref().unwrap_or_default(),
    )
    .ok()
}

pub(crate) async fn cors_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let origin = normalized_header_value(req.headers(), "origin", 256);
    let method = req.method().clone();
    if method == axum::http::Method::OPTIONS {
        let mut resp = StatusCode::NO_CONTENT.into_response();
        if let Some(origin_value) = origin {
            if state
                .api
                .cors_allowed_origins
                .iter()
                .any(|allowed| allowed == &origin_value)
            {
                if let Ok(value) = HeaderValue::from_str(&origin_value) {
                    resp.headers_mut().insert("access-control-allow-origin", value);
                }
                resp.headers_mut().insert(
                    "access-control-allow-methods",
                    HeaderValue::from_static("GET,OPTIONS"),
                );
                resp.headers_mut().insert(
                    "access-control-allow-headers",
                    HeaderValue::from_static(
                        "x-api-key,x-bijux-signature,x-bijux-timestamp,content-type",
                    ),
                );
            }
        }
        return resp;
    }

    let mut resp = next.run(req).await;
    if let Some(origin_value) = origin {
        if state
            .api
            .cors_allowed_origins
            .iter()
            .any(|allowed| allowed == &origin_value)
        {
            if let Ok(value) = HeaderValue::from_str(&origin_value) {
                resp.headers_mut().insert("access-control-allow-origin", value);
            }
            resp.headers_mut()
                .insert("vary", HeaderValue::from_static("Origin"));
        }
    }
    resp
}

pub(crate) async fn provenance_headers_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let dataset = parse_dataset_from_uri(req.uri());
    let mut resp = next.run(req).await;

    let (dataset_hash, release, artifact_hash): (Option<String>, Option<String>, Option<String>) =
        if let Some(ds) = dataset {
        let artifact_hash = state
            .cache
            .fetch_manifest_summary(&ds)
            .await
            .ok()
            .map(|m| m.dataset_signature_sha256);
        (
            Some(sha256_hex(ds.canonical_string().as_bytes())),
            Some(ds.release.to_string()),
            artifact_hash,
        )
        } else {
            (None, None, None)
        };

    if let Some(dataset_hash) = dataset_hash {
        if let Ok(v) = HeaderValue::from_str(&dataset_hash) {
            resp.headers_mut().insert("x-atlas-dataset-hash", v);
        }
    }
    if let Some(artifact_hash) = artifact_hash {
        if let Ok(v) = HeaderValue::from_str(&artifact_hash) {
            resp.headers_mut().insert("x-atlas-artifact-hash", v);
        }
    }
    if let Some(release) = release {
        if let Ok(v) = HeaderValue::from_str(&release) {
            resp.headers_mut().insert("x-atlas-release", v);
        }
    }
    resp
}

pub(crate) async fn resilience_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let request_id = crate::adapters::inbound::http::handlers::propagated_request_id(req.headers(), &state);
    if state.api.emergency_global_breaker
        && path != "/healthz"
        && path != "/healthz/overload"
        && path != "/readyz"
        && path != "/metrics"
    {
        let err = Json(ApiError::new(
            ApiErrorCode::NotReady,
            "emergency global circuit breaker is enabled",
            serde_json::json!({}),
            request_id.clone(),
        ));
        return crate::adapters::inbound::http::handlers::with_request_id(
            (StatusCode::SERVICE_UNAVAILABLE, err).into_response(),
            &request_id,
        );
    }
    if state.api.disable_heavy_endpoints && is_heavy_endpoint_path(&path) {
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "heavy endpoints are temporarily disabled by safety valve policy",
            serde_json::json!({"policy":"disable_heavy_endpoints"}),
            request_id.clone(),
        ));
        return crate::adapters::inbound::http::handlers::with_request_id(
            (StatusCode::SERVICE_UNAVAILABLE, err).into_response(),
            &request_id,
        );
    }
    let mut resp = next.run(req).await;
    if crate::adapters::inbound::http::middleware::shedding::overloaded(&state).await {
        resp.headers_mut()
            .insert("x-atlas-system-stress", HeaderValue::from_static("true"));
    }
    resp
}

fn is_heavy_endpoint_path(path: &str) -> bool {
    path == "/v1/genes"
        || path == "/v1/sequence/region"
        || path == "/v1/diff/genes"
        || path == "/v1/diff/region"
        || (path.starts_with("/v1/genes/") && path.ends_with("/sequence"))
}

pub(super) fn normalized_header_value(
    headers: &HeaderMap,
    key: &str,
    max_len: usize,
) -> Option<String> {
    let raw = headers.get(key)?.to_str().ok()?.trim();
    if raw.is_empty() || raw.len() > max_len {
        return None;
    }
    Some(raw.to_string())
}

fn normalized_forwarded_for(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("x-forwarded-for")?.to_str().ok()?;
    let first = raw.split(',').next()?.trim();
    if first.is_empty() || first.len() > 64 {
        return None;
    }
    if first
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b':' || b == b'-')
    {
        Some(first.to_string())
    } else {
        None
    }
}

fn proxy_authenticated_principal(
    headers: &HeaderMap,
    auth_mode: crate::runtime::config::AuthMode,
) -> Option<&'static str> {
    match auth_mode {
        crate::runtime::config::AuthMode::Oidc => {
            normalized_header_value(headers, "x-forwarded-user", 256)
        }
            .or_else(|| normalized_header_value(headers, "x-atlas-oidc-subject", 256))
            .map(|_| "user"),
        crate::runtime::config::AuthMode::Mtls => {
            normalized_header_value(headers, "x-forwarded-client-cert", 512)
                .or_else(|| normalized_header_value(headers, "x-atlas-mtls-subject", 256))
                .map(|_| "service-account")
        }
        _ => None,
    }
}

fn build_hmac_signature(secret: &str, method: &str, uri: &str, ts: &str) -> Option<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
    let payload = format!("{method}\n{uri}\n{ts}\n");
    mac.update(payload.as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}

async fn record_policy_violation(state: &AppState, policy: &str) {
    state
        .cache
        .metrics
        .policy_violations_total
        .fetch_add(1, Ordering::Relaxed);
    let mut by = state.cache.metrics.policy_violations_by_policy.lock().await;
    *by.entry(policy.to_string()).or_insert(0) += 1;
}

async fn record_auth_failure(state: &AppState, reason: &str, route: &str) {
    record_policy_violation(state, reason).await;
    let key = format!("auth.{reason}");
    let mut by = state.cache.metrics.policy_violations_by_policy.lock().await;
    let count = by.entry(key).or_insert(0);
    *count += 1;
    if *count % 50 == 0 {
        warn!(
            event_id = "authentication_failure_alert",
            event = "authentication_failure_alert",
            route = route,
            reason = reason,
            count = *count,
            "authentication failure threshold reached"
        );
    }
}

async fn record_authorization_denial(state: &AppState, route: &str, action: &str, resource_kind: &str) {
    record_policy_violation(state, "authorization.denied").await;
    let mut by = state.cache.metrics.policy_violations_by_policy.lock().await;
    let count = by.entry("authorization.denied".to_string()).or_insert(0);
    *count += 1;
    warn!(
        event_id = "authorization_denied",
        event = "authorization_denied",
        route = route,
        action = action,
        resource_kind = resource_kind,
        denial_count = *count,
        "authorization denied"
    );
}

pub async fn record_shed_reason(state: &AppState, reason: &str) {
    let mut by = state.cache.metrics.shed_total_by_reason.lock().await;
    *by.entry(reason.to_string()).or_insert(0) += 1;
}

#[allow(dead_code)] // ATLAS-EXC-0001
pub(crate) async fn record_invariant_violation(state: &AppState, invariant: &str) {
    let mut by = state.cache.metrics.invariant_violations_by_name.lock().await;
    *by.entry(invariant.to_string()).or_insert(0) += 1;
}

pub(crate) async fn security_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let uri_text = req.uri().to_string();
    let route = req.uri().path().to_string();
    let request_id = crate::adapters::inbound::http::handlers::propagated_request_id(req.headers(), &state);
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
    let header_bytes: usize = req
        .headers()
        .iter()
        .map(|(k, v)| k.as_str().len() + v.as_bytes().len())
        .sum();
    state
        .metrics
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
        .metrics
        .observe_client_fingerprint(client_type, ua_family)
        .await;

    let api_key = normalized_header_value(req.headers(), "x-api-key", 512);
    let api_key_store =
        ApiKeyStore::from_allowed_entries(&state.api.allowed_api_keys, state.api.api_key_expiration_days);
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
        if !api_key_store.records.is_empty()
            && api_key_store.validate(key, chrono_like_unix_secs()).is_err()
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
        crate::runtime::config::AuthMode::Token
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
            let now = chrono_like_unix_millis() / 1000;
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
        || state.api.auth_mode == crate::runtime::config::AuthMode::Disabled
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
        crate::runtime::config::AuthMode::Oidc
            | crate::runtime::config::AuthMode::Mtls
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
                &[("decision", "deny"), ("reason", "policy_denied"), ("route", route.as_str())],
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
    use std::time::Duration;
    use crate::runtime::config::AuthMode;

    #[test]
    fn health_endpoints_stay_auth_exempt_in_all_modes() {
        for mode in [
            AuthMode::Disabled,
            AuthMode::ApiKey,
            AuthMode::Token,
            AuthMode::Oidc,
            AuthMode::Mtls,
        ] {
            assert!(route_auth_exempt("/healthz"), "{mode:?} must allow /healthz");
            assert!(route_auth_exempt("/readyz"), "{mode:?} must allow /readyz");
            assert!(route_auth_exempt("/v1/version"), "{mode:?} must allow /v1/version");
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
        assert_eq!(proxy_authenticated_principal(&headers, AuthMode::Oidc), None);
        assert_eq!(proxy_authenticated_principal(&headers, AuthMode::Mtls), None);
        headers.insert("x-forwarded-user", HeaderValue::from_static("alice"));
        assert_eq!(proxy_authenticated_principal(&headers, AuthMode::Oidc), Some("user"));
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
            crate::runtime::config::AuditSink::Stdout,
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
            crate::runtime::config::AuditSink::Stdout,
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
        let valid_permissions = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../configs/security/permissions.yaml"
        ));
        let valid_roles = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../configs/security/roles.yaml"
        ));
        let valid_policy = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../configs/security/policy.yaml"
        ));

        let permissions_err =
            build_embedded_authorization_engine("permissions: [", valid_roles, valid_policy)
                .expect_err("bad permissions");
        let roles_err =
            build_embedded_authorization_engine(valid_permissions, "roles: [", valid_policy)
                .expect_err("bad roles");
        let policy_err =
            build_embedded_authorization_engine(valid_permissions, valid_roles, "rules: [")
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
                format!("hash={}|not_before={}", hash_api_key(&future), now.saturating_add(60)),
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
        let mut api = crate::runtime::config::ApiConfig {
            token_signing_secret: Some("token-secret".to_string()),
            token_required_issuer: Some("atlas-auth".to_string()),
            token_required_audience: Some("atlas-api".to_string()),
            token_required_scopes: vec!["dataset.read".to_string()],
            ..crate::runtime::config::ApiConfig::default()
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
        let api = crate::runtime::config::ApiConfig {
            token_signing_secret: Some("token-secret".to_string()),
            ..crate::runtime::config::ApiConfig::default()
        };
        assert_eq!(
            validate_signed_token("not.a.jwt", &api),
            Err(TokenValidationError::Malformed)
        );
    }

    #[test]
    fn token_validation_requires_a_non_empty_subject() {
        let now = chrono_like_unix_secs();
        let api = crate::runtime::config::ApiConfig {
            token_signing_secret: Some("token-secret".to_string()),
            ..crate::runtime::config::ApiConfig::default()
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

fn classify_client_type(user_agent: Option<&str>) -> &'static str {
    let Some(ua) = user_agent else {
        return "unknown";
    };
    let normalized = ua.to_ascii_lowercase();
    if normalized.contains("mozilla/")
        || normalized.contains("chrome/")
        || normalized.contains("safari/")
        || normalized.contains("firefox/")
    {
        "human"
    } else {
        "machine"
    }
}

fn classify_user_agent_family(user_agent: Option<&str>) -> &'static str {
    let Some(ua) = user_agent else {
        return "unknown";
    };
    let normalized = ua.to_ascii_lowercase();
    if normalized.contains("curl/") {
        "curl"
    } else if normalized.contains("k6/") {
        "k6"
    } else if normalized.contains("mozilla/") {
        "browser"
    } else if normalized.contains("python-requests") {
        "python-requests"
    } else {
        "other"
    }
}
