// SPDX-License-Identifier: Apache-2.0

use crate::app::server::state::AppState;
use crate::sha256_hex;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bijux_atlas_api::{ApiError, ApiErrorCode};
use bijux_atlas_model::dataset::DatasetId;

pub(super) fn parse_dataset_from_uri(uri: &Uri) -> Option<DatasetId> {
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
                    resp.headers_mut()
                        .insert("access-control-allow-origin", value);
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
                resp.headers_mut()
                    .insert("access-control-allow-origin", value);
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

pub(crate) async fn debug_route_hardening_middleware(
    State(_state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let mut resp = next.run(req).await;
    if path.starts_with("/debug/") || path.starts_with("/v1/_debug/") {
        resp.headers_mut().insert(
            "cache-control",
            HeaderValue::from_static("no-store, max-age=0"),
        );
        resp.headers_mut()
            .insert("pragma", HeaderValue::from_static("no-cache"));
        resp.headers_mut().insert(
            "x-robots-tag",
            HeaderValue::from_static("noindex, nofollow"),
        );
    }
    resp
}

pub(crate) async fn resilience_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let request_id =
        crate::adapters::inbound::http::handlers::propagated_request_id(req.headers(), &state);
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

pub(super) fn normalized_forwarded_for(headers: &HeaderMap) -> Option<String> {
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

pub(super) fn classify_client_type(user_agent: Option<&str>) -> &'static str {
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

pub(super) fn classify_user_agent_family(user_agent: Option<&str>) -> &'static str {
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
