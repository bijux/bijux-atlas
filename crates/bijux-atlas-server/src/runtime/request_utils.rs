// SPDX-License-Identifier: Apache-2.0

fn chrono_like_unix_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

fn chrono_like_unix_secs() -> u64 {
    (chrono_like_unix_millis() / 1000) as u64
}

pub(crate) fn route_sli_class(route: &str) -> &'static str {
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

fn embedded_policy_allows(
    principal: &str,
    action: &str,
    resource_kind: &str,
    route: &str,
) -> bool {
    static POLICY: std::sync::OnceLock<serde_yaml::Value> = std::sync::OnceLock::new();
    let policy = POLICY.get_or_init(|| {
        serde_yaml::from_str(include_str!("../../../../configs/security/policy.yaml"))
            .unwrap_or_else(|err| panic!("embedded auth policy: {err}"))
    });
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

fn emit_auth_policy_decision(
    auth_mode: crate::config::AuthMode,
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
    sink: crate::config::AuditSink,
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
    audit: &crate::config::AuditConfig,
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
    if matches!(audit.sink, crate::config::AuditSink::File) {
        let _ = write_audit_file_record(&audit.file_path, audit.max_bytes, &payload);
    }
    info!(
        target: "atlas_audit",
        event_id = format!("audit_{event_name}"),
        audit_payload = %payload,
        "audit event"
    );
}

fn write_audit_file_record(
    file_path: &str,
    max_bytes: u64,
    payload: &serde_json::Value,
) -> std::io::Result<()> {
    use std::fs::{self, OpenOptions};
    use std::io;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut encoded = serde_json::to_vec(payload)
        .map_err(|err| io::Error::other(format!("encode audit payload failed: {err}")))?;
    encoded.push(b'\n');
    let current_len = fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    if current_len.saturating_add(encoded.len() as u64) > max_bytes {
        let rotated = PathBuf::from(format!("{file_path}.1"));
        if rotated.exists() {
            fs::remove_file(&rotated)?;
        }
        if path.exists() {
            fs::rename(path, &rotated)?;
        }
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(&encoded)?;
    file.flush()?;
    Ok(())
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

async fn provenance_headers_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let dataset = parse_dataset_from_uri(req.uri());
    let mut resp = next.run(req).await;

    let (dataset_hash, release, artifact_hash) = if let Some(ds) = dataset {
        let artifact_hash = state
            .cache
            .fetch_manifest_summary(&ds)
            .await
            .ok()
            .map(|m| m.dataset_signature_sha256);
        (
            sha256_hex(ds.canonical_string().as_bytes()),
            ds.release.to_string(),
            artifact_hash,
        )
    } else {
        ("unknown".to_string(), "unknown".to_string(), None)
    };

    if let Ok(v) = HeaderValue::from_str(&dataset_hash) {
        resp.headers_mut().insert("x-atlas-dataset-hash", v);
    }
    if let Some(artifact_hash) = artifact_hash {
        if let Ok(v) = HeaderValue::from_str(&artifact_hash) {
            resp.headers_mut().insert("x-atlas-artifact-hash", v);
        }
    }
    if let Ok(v) = HeaderValue::from_str(&release) {
        resp.headers_mut().insert("x-atlas-release", v);
    }
    resp
}

async fn resilience_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
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
            "req-unknown",
        ));
        return (StatusCode::SERVICE_UNAVAILABLE, err).into_response();
    }
    if state.api.disable_heavy_endpoints && is_heavy_endpoint_path(&path) {
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "heavy endpoints are temporarily disabled by safety valve policy",
            serde_json::json!({"policy":"disable_heavy_endpoints"}),
            "req-unknown",
        ));
        return (StatusCode::SERVICE_UNAVAILABLE, err).into_response();
    }
    let mut resp = next.run(req).await;
    if crate::middleware::shedding::overloaded(&state).await {
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

fn normalized_header_value(headers: &HeaderMap, key: &str, max_len: usize) -> Option<String> {
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
    auth_mode: crate::config::AuthMode,
) -> Option<&'static str> {
    match auth_mode {
        crate::config::AuthMode::Oidc => normalized_header_value(headers, "x-forwarded-user", 256)
            .or_else(|| normalized_header_value(headers, "x-atlas-oidc-subject", 256))
            .map(|_| "user"),
        crate::config::AuthMode::Mtls => {
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

pub(crate) async fn record_shed_reason(state: &AppState, reason: &str) {
    let mut by = state.cache.metrics.shed_total_by_reason.lock().await;
    *by.entry(reason.to_string()).or_insert(0) += 1;
}

#[allow(dead_code)] // ATLAS-EXC-0001
pub(crate) async fn record_invariant_violation(state: &AppState, invariant: &str) {
    let mut by = state.cache.metrics.invariant_violations_by_name.lock().await;
    *by.entry(invariant.to_string()).or_insert(0) += 1;
}

async fn security_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let uri_text = req.uri().to_string();
    let route = req.uri().path().to_string();
    let auth_exempt = route_auth_exempt(&route);
    if route_is_admin_endpoint(&route) && !state.api.enable_admin_endpoints {
        emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
        let err = Json(ApiError::new(
            ApiErrorCode::DatasetNotFound,
            "admin endpoints are disabled",
            serde_json::json!({}),
            "req-unknown",
        ));
        return (StatusCode::NOT_FOUND, err).into_response();
    }
    if uri_text.len() > state.api.max_uri_bytes {
        record_policy_violation(&state, "uri_bytes").await;
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "request URI too large",
            serde_json::json!({"max_uri_bytes": state.api.max_uri_bytes, "actual": uri_text.len()}),
            "req-unknown",
        ));
        return (StatusCode::BAD_REQUEST, err).into_response();
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
            "req-unknown",
        ));
        return (StatusCode::BAD_REQUEST, err).into_response();
    }

    let user_agent = normalized_header_value(req.headers(), "user-agent", 512);
    let client_type = classify_client_type(user_agent.as_deref());
    let ua_family = classify_user_agent_family(user_agent.as_deref());
    state
        .metrics
        .observe_client_fingerprint(client_type, ua_family)
        .await;

    let api_key = normalized_header_value(req.headers(), "x-api-key", 256);
    if !auth_exempt && state.api.require_api_key && api_key.is_none() {
        emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
        record_policy_violation(&state, "api_key_required").await;
        let err = Json(ApiError::new(
            auth_error_code(StatusCode::UNAUTHORIZED),
            "api key required",
            serde_json::json!({}),
            "req-unknown",
        ));
        return (StatusCode::UNAUTHORIZED, err).into_response();
    }
    if let Some(key) = &api_key {
        if !state.api.allowed_api_keys.is_empty()
            && !state.api.allowed_api_keys.iter().any(|k| k == key)
        {
            emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
            record_policy_violation(&state, "api_key_invalid").await;
            let err = Json(ApiError::new(
                auth_error_code(StatusCode::UNAUTHORIZED),
                "invalid api key",
                serde_json::json!({}),
                "req-unknown",
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
    }

    if let Some(secret) = &state.api.hmac_secret {
        let ts = normalized_header_value(req.headers(), "x-bijux-timestamp", 64);
        let sig = normalized_header_value(req.headers(), "x-bijux-signature", 128);
        if !auth_exempt && state.api.hmac_required && (ts.is_none() || sig.is_none()) {
            emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
            record_policy_violation(&state, "hmac_missing_headers").await;
            let err = Json(ApiError::new(
                auth_error_code(StatusCode::UNAUTHORIZED),
                "missing required HMAC headers",
                serde_json::json!({}),
                "req-unknown",
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
        if let (Some(ts_value), Some(sig_value)) = (ts, sig) {
            let now = chrono_like_unix_millis() / 1000;
            let Some(parsed_ts) = ts_value.parse::<u128>().ok() else {
                emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
                record_policy_violation(&state, "hmac_invalid_timestamp").await;
                let err = Json(ApiError::new(
                    auth_error_code(StatusCode::UNAUTHORIZED),
                    "invalid hmac timestamp",
                    serde_json::json!({}),
                    "req-unknown",
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            };
            let skew = now.abs_diff(parsed_ts);
            if skew > state.api.hmac_max_skew_secs as u128 {
                emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
                record_policy_violation(&state, "hmac_skew").await;
                let err = Json(ApiError::new(
                    auth_error_code(StatusCode::UNAUTHORIZED),
                    "hmac timestamp outside allowed skew",
                    serde_json::json!({"max_skew_secs": state.api.hmac_max_skew_secs}),
                    "req-unknown",
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            }
            let method = req.method().as_str();
            let uri = req.uri().path_and_query().map_or("", |pq| pq.as_str());
            if build_hmac_signature(secret, method, uri, &ts_value).as_deref()
                != Some(sig_value.as_str())
            {
                emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
                record_policy_violation(&state, "hmac_signature").await;
                let err = Json(ApiError::new(
                    auth_error_code(StatusCode::UNAUTHORIZED),
                    "invalid hmac signature",
                    serde_json::json!({}),
                    "req-unknown",
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            }
        }
    }

    let principal = if route_is_admin_endpoint(&route) {
        "operator"
    } else if auth_exempt || state.api.auth_mode == crate::config::AuthMode::Disabled {
        "user"
    } else if matches!(
        state.api.auth_mode,
        crate::config::AuthMode::Oidc | crate::config::AuthMode::Mtls
    ) {
        let Some(principal) = proxy_authenticated_principal(req.headers(), state.api.auth_mode)
        else {
            emit_auth_policy_decision(state.api.auth_mode, "user", &route, false);
            record_policy_violation(&state, "proxy_identity_missing").await;
            let err = Json(ApiError::new(
                auth_error_code(StatusCode::UNAUTHORIZED),
                "trusted auth proxy identity header required",
                serde_json::json!({"auth_mode": state.api.auth_mode.as_str()}),
                "req-unknown",
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        };
        principal
    } else {
        "service-account"
    };
    let policy_allowed = embedded_policy_allows(
        principal,
        route_action_id(&route),
        route_resource_kind(&route),
        &route,
    );
    emit_auth_policy_decision(state.api.auth_mode, principal, &route, policy_allowed);
    if !policy_allowed {
        record_policy_violation(&state, "auth_policy_denied").await;
        let err = Json(ApiError::new(
            auth_error_code(StatusCode::FORBIDDEN),
            "request denied by access policy",
            serde_json::json!({
                "action": route_action_id(&route),
                "resource_kind": route_resource_kind(&route)
            }),
            "req-unknown",
        ));
        return (StatusCode::FORBIDDEN, err).into_response();
    }

    let started = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let request_id =
        normalized_header_value(req.headers(), "x-request-id", 128).unwrap_or_default();
    let client_ip =
        normalized_forwarded_for(req.headers()).unwrap_or_else(|| "unknown".to_string());
    let resp = next.run(req).await;
    if state.api.audit.enabled {
        let event_name = if route_is_admin_endpoint(&path) {
            "admin_action"
        } else {
            "query_executed"
        };
        let status_text = resp.status().as_u16().to_string();
        let latency_ms = started.elapsed().as_millis().to_string();
        emit_audit_event(
            &state.api.audit,
            event_name,
            Some(principal),
            route_action_id(&path),
            route_resource_kind(&path),
            &path,
            &[
                ("method", method.as_str()),
                ("status", status_text.as_str()),
                ("request_id", request_id.as_str()),
                ("client_ip", client_ip.as_str()),
                ("latency_ms", latency_ms.as_str()),
            ],
        );
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthMode;

    #[test]
    fn health_endpoints_stay_auth_exempt_in_all_modes() {
        for mode in [
            AuthMode::Disabled,
            AuthMode::ApiKey,
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
            crate::config::AuditSink::Stdout,
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
    fn audit_file_rotation_happens_at_max_bytes() {
        let root = std::env::temp_dir().join(format!(
            "atlas-audit-rotation-{}",
            chrono_like_unix_millis()
        ));
        std::fs::create_dir_all(&root).expect("create temp dir");
        let log_path = root.join("audit.log");
        let payload = serde_json::json!({
            "event_id": "audit_query_executed",
            "event_name": "query_executed",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 1,
            "sink": "file",
            "action": "dataset.read",
            "resource_kind": "dataset-id",
            "resource_id": "/v1/datasets"
        });
        write_audit_file_record(log_path.to_str().unwrap_or_default(), 64, &payload)
            .expect("first write");
        write_audit_file_record(log_path.to_str().unwrap_or_default(), 64, &payload)
            .expect("second write");
        assert!(log_path.exists());
        assert!(root.join("audit.log.1").exists());
        let _ = std::fs::remove_file(root.join("audit.log.1"));
        let _ = std::fs::remove_file(&log_path);
        let _ = std::fs::remove_dir(&root);
    }

    #[test]
    fn audit_event_drops_unknown_or_sensitive_dynamic_fields() {
        let event = build_audit_event(
            "query_executed",
            Some("service-account"),
            "dataset.read",
            "dataset-id",
            "/v1/datasets",
            crate::config::AuditSink::Stdout,
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
