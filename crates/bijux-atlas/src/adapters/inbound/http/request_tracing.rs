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
    let correlation_id = normalized_trace_value(headers, "x-correlation-id", 128);
    let run_id = normalized_trace_value(headers, "x-run-id", 128);
    let traceparent = normalized_traceparent(headers);
    let request_origin = normalized_trace_value(headers, "x-request-origin", 256);

    let request_id = normalized_request_id(headers).unwrap_or_else(|| {
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

fn normalized_trace_value(headers: &HeaderMap, key: &str, max_len: usize) -> Option<String> {
    let raw = headers.get(key)?.to_str().ok()?.trim();
    if raw.is_empty() || raw.len() > max_len {
        return None;
    }
    if raw.bytes().all(is_trace_char) {
        Some(raw.to_string())
    } else {
        None
    }
}

fn normalized_request_id(headers: &HeaderMap) -> Option<String> {
    normalized_trace_value(headers, "x-request-id", 128)
}

fn normalized_traceparent(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("traceparent")?.to_str().ok()?.trim();
    if raw.is_empty() || raw.len() > 128 {
        return None;
    }
    let mut parts = raw.split('-');
    let (Some(version), Some(trace_id), Some(parent_id), Some(flags), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return None;
    };
    if version.len() != 2 || trace_id.len() != 32 || parent_id.len() != 16 || flags.len() != 2 {
        return None;
    }
    let valid_hex = |value: &str| value.bytes().all(|b| b.is_ascii_hexdigit());
    if valid_hex(version) && valid_hex(trace_id) && valid_hex(parent_id) && valid_hex(flags) {
        Some(raw.to_ascii_lowercase())
    } else {
        None
    }
}

fn is_trace_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, b'.' | b'_' | b':' | b'/' | b'-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::store::testing::FakeStore;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn extracts_request_trace_fields() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("req-abc"));
        headers.insert("x-correlation-id", HeaderValue::from_static("corr-1"));
        headers.insert("x-run-id", HeaderValue::from_static("run-1"));

        let state = crate::AppState::new(crate::DatasetCacheManager::new(
            crate::DatasetCacheConfig::default(),
            std::sync::Arc::new(FakeStore::default()),
        ));
        let trace = extract_request_trace(&headers, &state);
        assert_eq!(trace.request_id, "req-abc");
        assert_eq!(trace.correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(trace.run_id.as_deref(), Some("run-1"));
        assert_eq!(trace.traceparent, None);
        assert_eq!(trace.request_origin, None);
    }

    #[test]
    fn invalid_or_oversized_trace_headers_are_dropped() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("bad id with spaces"));
        headers.insert(
            "x-correlation-id",
            HeaderValue::from_str(&"x".repeat(129)).expect("header value"),
        );
        headers.insert("traceparent", HeaderValue::from_static("00-not-a-trace"));
        headers.insert("x-request-origin", HeaderValue::from_static("origin\tbad"));

        let state = crate::AppState::new(crate::DatasetCacheManager::new(
            crate::DatasetCacheConfig::default(),
            std::sync::Arc::new(FakeStore::default()),
        ));
        let trace = extract_request_trace(&headers, &state);
        assert!(trace.request_id.starts_with("req-"));
        assert_eq!(trace.correlation_id, None);
        assert_eq!(trace.traceparent, None);
        assert_eq!(trace.request_origin, None);
    }

    #[test]
    fn traceparent_is_normalized_to_lowercase_when_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "traceparent",
            HeaderValue::from_static(
                "00-4BF92F3577B34DA6A3CE929D0E0E4736-00F067AA0BA902B7-01",
            ),
        );

        let state = crate::AppState::new(crate::DatasetCacheManager::new(
            crate::DatasetCacheConfig::default(),
            std::sync::Arc::new(FakeStore::default()),
        ));
        let trace = extract_request_trace(&headers, &state);
        assert_eq!(
            trace.traceparent.as_deref(),
            Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
        );
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
            std::sync::Arc::new(FakeStore::default()),
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
