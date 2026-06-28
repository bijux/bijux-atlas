// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::route_support::*;
use crate::*;
use serde_json::json;

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
