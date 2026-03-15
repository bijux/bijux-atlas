// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use axum::http::HeaderMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestTrace {
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub run_id: Option<String>,
    pub traceparent: Option<String>,
    pub request_origin: Option<String>,
}

#[must_use]
pub(crate) fn extract_request_trace(headers: &HeaderMap, state: &AppState) -> RequestTrace {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string);
    let run_id = headers
        .get("x-run-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string);
    let traceparent = headers
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string);
    let request_origin = headers
        .get("x-request-origin")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string);

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            let id = state
                .request_id_seed
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("req-{id:016x}")
        });

    RequestTrace {
        request_id,
        correlation_id,
        run_id,
        traceparent,
        request_origin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn extracts_request_trace_fields() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("req-abc"));
        headers.insert("x-correlation-id", HeaderValue::from_static("corr-1"));
        headers.insert("x-run-id", HeaderValue::from_static("run-1"));

        let state = crate::AppState::new(crate::DatasetCacheManager::new(
            crate::DatasetCacheConfig::default(),
            std::sync::Arc::new(crate::FakeStore::default()),
        ));
        let trace = extract_request_trace(&headers, &state);
        assert_eq!(trace.request_id, "req-abc");
        assert_eq!(trace.correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(trace.run_id.as_deref(), Some("run-1"));
        assert_eq!(trace.traceparent, None);
        assert_eq!(trace.request_origin, None);
    }

    #[test]
    fn trace_extraction_overhead_stays_within_budget() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("req-budget"));
        headers.insert(
            "traceparent",
            HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"),
        );
        headers.insert("x-request-origin", HeaderValue::from_static("perf-test"));

        let state = crate::AppState::new(crate::DatasetCacheManager::new(
            crate::DatasetCacheConfig::default(),
            std::sync::Arc::new(crate::FakeStore::default()),
        ));
        let started = std::time::Instant::now();
        for _ in 0..10_000 {
            let trace = extract_request_trace(&headers, &state);
            assert_eq!(trace.request_id, "req-budget");
        }
        let elapsed = started.elapsed();
        assert!(
            elapsed < std::time::Duration::from_secs(2),
            "trace extraction overhead exceeded budget: {elapsed:?}"
        );
    }
}
