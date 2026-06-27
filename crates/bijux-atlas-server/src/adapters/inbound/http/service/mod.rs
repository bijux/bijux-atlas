// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::route_support::*;
use crate::*;
use serde_json::json;

mod cluster_membership;
mod cluster_replicas;
mod recovery_controls;

pub(crate) use cluster_membership::{
    cluster_heartbeat_handler, cluster_mode_handler, cluster_nodes_handler,
    cluster_register_handler, cluster_status_handler,
};
pub(crate) use cluster_replicas::{
    cluster_replica_diagnostics_handler, cluster_replica_failover_handler,
    cluster_replica_health_handler, cluster_replica_list_handler,
};
pub(crate) use recovery_controls::{
    chaos_run_handler, cluster_recovery_run_handler, failure_injection_handler,
    recovery_diagnostics_handler,
};

pub(crate) async fn landing_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| Catalog::new(vec![]));
    let mut list = String::new();
    for entry in &catalog.datasets {
        let ds = &entry.dataset;
        let canon = ds.canonical_string();
        list.push_str(&format!(
            "<li><code>{canon}</code> - <a href=\"/v1/genes/count?release={}&species={}&assembly={}\">genes/count</a></li>",
            ds.release, ds.species, ds.assembly
        ));
    }
    if list.is_empty() {
        list.push_str("<li>No datasets published yet.</li>");
    }
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Bijux Atlas</title></head><body>\
<h1>Bijux Atlas Dataset Browser</h1>\
<p>Version: <code>{}</code></p>\
<h2>Datasets</h2><ul>{}</ul>\
<h2>Example Queries</h2>\
<ul>\
<li><a href=\"/v1/datasets\">/v1/datasets</a></li>\
<li><a href=\"/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5\">/v1/genes?...&limit=5</a></li>\
<li><a href=\"/v1/diff/genes?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&limit=10\">/v1/diff/genes?...&limit=10</a></li>\
</ul>\
</body></html>",
        crate::version::runtime_version(),
        list
    );
    let mut resp = Response::new(Body::from(html));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        "content-type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    state
        .metrics()
        .observe_request("/", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn healthz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let resp = (StatusCode::OK, "ok").into_response();
    state
        .metrics()
        .observe_request_with_trace(
            "/healthz",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let resp = (StatusCode::OK, "ok").into_response();
    state
        .metrics()
        .observe_request_with_trace(
            "/health",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn overload_health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let overloaded = crate::adapters::inbound::http::middleware::shedding::overloaded(&state).await;
    let live = state.accepting_requests.load(Ordering::Relaxed);
    let catalog_present = state.cache.current_catalog().await.is_some();
    let ready = state.ready.load(Ordering::Relaxed)
        && readyz_catalog_ready(
            state.api.readiness_requires_catalog,
            state.cache.cached_only_mode(),
            catalog_present,
        );
    let status = if overloaded {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };
    let resp = (
        status,
        Json(json!({
            "overloaded": overloaded,
            "ready": ready,
            "live": live,
            "draining": !live,
            "cached_only_mode": state.cache.cached_only_mode(),
            "emergency_breaker": state.api.emergency_global_breaker
        })),
    )
        .into_response();
    state
        .metrics()
        .observe_request_with_trace(
            "/healthz/overload",
            status,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn version_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let payload = json!({
        "api_version": "v1",
        "contract_version": "v1",
        "plugin": {
            "name": "bijux-atlas",
            "version": crate::version::runtime_version(),
            "compatible_umbrella": ">=0.3.0,<0.4.0",
            "build_hash": bijux_atlas_runtime::runtime::config::runtime_build_hash(),
        },
        "server": {
            "crate": CRATE_NAME,
            "config_schema_version": bijux_atlas_runtime::runtime::config::CONFIG_SCHEMA_VERSION,
            "api_version": "v1",
            "api_contract_version": "v1",
            "runtime_policy_hash": &*state.runtime_policy_hash,
            "artifact_schema_versions": {
                "manifest_schema_version": "1",
                "sqlite_schema_version": "4"
            }
        }
    });
    let mut response = Json(payload).into_response();
    if let Ok(value) = HeaderValue::from_str("public, max-age=30") {
        response.headers_mut().insert("cache-control", value);
    }
    state
        .metrics()
        .observe_request("/v1/version", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn openapi_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let mut spec = bijux_atlas_api::openapi_v1_spec();
    if let Some(info) = spec
        .get_mut("info")
        .and_then(serde_json::Value::as_object_mut)
    {
        info.insert(
            "x-build-id".to_string(),
            serde_json::Value::String(
                bijux_atlas_runtime::runtime::config::runtime_build_hash().to_string(),
            ),
        );
    }
    let mut response = Json(spec).into_response();
    if let Ok(value) = HeaderValue::from_str("public, max-age=30") {
        response.headers_mut().insert("cache-control", value);
    }
    state
        .metrics()
        .observe_request("/v1/openapi.json", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn readyz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let catalog_present = state.cache.current_catalog().await.is_some();
    let catalog_ready = readyz_catalog_ready(
        state.api.readiness_requires_catalog,
        state.cache.cached_only_mode(),
        catalog_present,
    );
    if state.ready.load(Ordering::Relaxed) && catalog_ready {
        let resp = (StatusCode::OK, "ready").into_response();
        state
            .metrics()
            .observe_request_with_trace(
                "/readyz",
                StatusCode::OK,
                started.elapsed(),
                Some(&request_id),
            )
            .await;
        with_request_id(resp, &request_id)
    } else {
        let resp = (StatusCode::SERVICE_UNAVAILABLE, "not-ready").into_response();
        state
            .metrics()
            .observe_request_with_trace(
                "/readyz",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
                Some(&request_id),
            )
            .await;
        with_request_id(resp, &request_id)
    }
}

pub(crate) async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let catalog_present = state.cache.current_catalog().await.is_some();
    let catalog_ready = readyz_catalog_ready(
        state.api.readiness_requires_catalog,
        state.cache.cached_only_mode(),
        catalog_present,
    );
    let status = if state.ready.load(Ordering::Relaxed) && catalog_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = if status == StatusCode::OK {
        "ready"
    } else {
        "not-ready"
    };
    let resp = (status, body).into_response();
    state
        .metrics()
        .observe_request_with_trace("/ready", status, started.elapsed(), Some(&request_id))
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn live_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let is_live = state.accepting_requests.load(Ordering::Relaxed);
    let status = if is_live {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let resp = (
        status,
        Json(json!({
            "live": is_live,
            "draining": !is_live
        })),
    )
        .into_response();
    state
        .metrics()
        .observe_request_with_trace("/live", status, started.elapsed(), Some(&request_id))
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) fn readyz_catalog_ready(
    readiness_requires_catalog: bool,
    cached_only_mode: bool,
    catalog_present: bool,
) -> bool {
    if readiness_requires_catalog && !cached_only_mode {
        catalog_present
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::readyz_catalog_ready;

    #[test]
    fn readyz_requires_catalog_when_enabled_and_not_cached_only() {
        assert!(!readyz_catalog_ready(true, false, false));
        assert!(readyz_catalog_ready(true, false, true));
    }

    #[test]
    fn readyz_ignores_catalog_when_cached_only_or_not_required() {
        assert!(readyz_catalog_ready(true, true, false));
        assert!(readyz_catalog_ready(false, false, false));
        assert!(readyz_catalog_ready(false, true, false));
    }

    #[test]
    fn readyz_offline_profile_stays_ready_without_catalog() {
        assert!(readyz_catalog_ready(true, true, false));
    }

    #[test]
    fn readyz_baseline_and_perf_profiles_require_catalog_when_enabled() {
        assert!(!readyz_catalog_ready(true, false, false));
        assert!(readyz_catalog_ready(true, false, true));
    }
}
