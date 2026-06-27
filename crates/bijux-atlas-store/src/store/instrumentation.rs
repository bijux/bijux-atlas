// SPDX-License-Identifier: Apache-2.0

use super::StoreErrorCode;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct StoreMetrics {
    pub bytes_downloaded: u64,
    pub bytes_uploaded: u64,
    pub request_count: u64,
    pub latency_ms_total: u128,
    pub failures_by_class: BTreeMap<String, u64>,
}

#[derive(Default)]
pub struct StoreMetricsCollector {
    inner: Mutex<StoreMetrics>,
}

impl StoreMetricsCollector {
    #[must_use]
    pub fn snapshot(&self) -> StoreMetrics {
        self.inner.lock().map(|m| m.clone()).unwrap_or_default()
    }
}

pub trait StoreInstrumentation: Send + Sync + 'static {
    fn observe_download(&self, _backend: &str, _bytes: usize, _latency: Duration) {}
    fn observe_upload(&self, _backend: &str, _bytes: usize, _latency: Duration) {}
    fn observe_error(&self, _backend: &str, _code: StoreErrorCode) {}
}

#[derive(Default)]
pub struct NoopInstrumentation;

impl StoreInstrumentation for NoopInstrumentation {}

impl StoreInstrumentation for StoreMetricsCollector {
    fn observe_download(&self, _backend: &str, bytes: usize, latency: Duration) {
        if let Ok(mut m) = self.inner.lock() {
            m.bytes_downloaded = m.bytes_downloaded.saturating_add(bytes as u64);
            m.request_count = m.request_count.saturating_add(1);
            m.latency_ms_total = m.latency_ms_total.saturating_add(latency.as_millis());
        }
    }

    fn observe_upload(&self, _backend: &str, bytes: usize, latency: Duration) {
        if let Ok(mut m) = self.inner.lock() {
            m.bytes_uploaded = m.bytes_uploaded.saturating_add(bytes as u64);
            m.request_count = m.request_count.saturating_add(1);
            m.latency_ms_total = m.latency_ms_total.saturating_add(latency.as_millis());
        }
    }

    fn observe_error(&self, _backend: &str, code: StoreErrorCode) {
        if let Ok(mut m) = self.inner.lock() {
            m.request_count = m.request_count.saturating_add(1);
            *m.failures_by_class
                .entry(code.as_str().to_string())
                .or_insert(0) += 1;
        }
    }
}
