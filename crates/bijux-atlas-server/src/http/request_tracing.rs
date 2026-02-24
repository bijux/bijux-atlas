// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use axum::http::HeaderMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestTrace {
    pub request_id: String,
    pub correlation_id: Option<String>,
    pub run_id: Option<String>,
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
    }
}
