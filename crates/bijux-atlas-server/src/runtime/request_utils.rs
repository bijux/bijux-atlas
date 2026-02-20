fn chrono_like_unix_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

pub(crate) fn route_sli_class(route: &str) -> &'static str {
    if matches!(route, "/healthz" | "/readyz" | "/metrics" | "/v1/version") {
        return "cheap";
    }
    if route.contains("/diff") || route.contains("/region") || route.contains("/sequence") {
        return "heavy";
    }
    "standard"
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
    if state.api.require_api_key && api_key.is_none() {
        record_policy_violation(&state, "api_key_required").await;
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
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
            record_policy_violation(&state, "api_key_invalid").await;
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
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
        if state.api.hmac_required && (ts.is_none() || sig.is_none()) {
            record_policy_violation(&state, "hmac_missing_headers").await;
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
                "missing required HMAC headers",
                serde_json::json!({}),
                "req-unknown",
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
        if let (Some(ts_value), Some(sig_value)) = (ts, sig) {
            let now = chrono_like_unix_millis() / 1000;
            let Some(parsed_ts) = ts_value.parse::<u128>().ok() else {
                record_policy_violation(&state, "hmac_invalid_timestamp").await;
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "invalid hmac timestamp",
                    serde_json::json!({}),
                    "req-unknown",
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            };
            let skew = now.abs_diff(parsed_ts);
            if skew > state.api.hmac_max_skew_secs as u128 {
                record_policy_violation(&state, "hmac_skew").await;
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
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
                record_policy_violation(&state, "hmac_signature").await;
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "invalid hmac signature",
                    serde_json::json!({}),
                    "req-unknown",
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            }
        }
    }

    let started = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let request_id =
        normalized_header_value(req.headers(), "x-request-id", 128).unwrap_or_default();
    let client_ip =
        normalized_forwarded_for(req.headers()).unwrap_or_else(|| "unknown".to_string());
    let resp = next.run(req).await;
    if state.api.enable_audit_log {
        info!(
            target: "atlas_audit",
            method = %method,
            path = %path,
            status = resp.status().as_u16(),
            request_id = %request_id,
            client_ip = %client_ip,
            latency_ms = started.elapsed().as_millis() as u64,
            "audit"
        );
    }
    resp
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

