// SPDX-License-Identifier: Apache-2.0

use crate::app::server::AppState;

pub fn unix_time_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

pub fn chrono_like_unix_millis() -> u128 {
    unix_time_millis()
}

pub fn route_sli_class(route: &str) -> &'static str {
    if matches!(
        route,
        "/health" | "/healthz" | "/ready" | "/readyz" | "/live" | "/metrics" | "/v1/version"
    ) {
        return "cheap";
    }
    if route.contains("/diff") || route.contains("/region") || route.contains("/sequence") {
        return "heavy";
    }
    "standard"
}

pub async fn record_shed_reason(state: &AppState, reason: &str) {
    let mut by = state.cache.metrics.shed_total_by_reason.lock().await;
    *by.entry(reason.to_string()).or_insert(0) += 1;
}
