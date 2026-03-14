// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};

pub trait ClientMetrics: Send + Sync {
    fn observe_request(&self, endpoint: &str, elapsed_millis: u128, ok: bool);
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMetrics {
    inner: Arc<Mutex<Vec<(String, u128, bool)>>>,
}

impl InMemoryMetrics {
    #[must_use]
    pub fn snapshot(&self) -> Vec<(String, u128, bool)> {
        match self.inner.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    }

    #[must_use]
    pub fn export_json(&self) -> serde_json::Value {
        let rows = self
            .snapshot()
            .into_iter()
            .map(|(endpoint, elapsed_millis, ok)| {
                serde_json::json!({
                    "endpoint": endpoint,
                    "elapsed_millis": elapsed_millis,
                    "ok": ok,
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "schema_version": 1,
            "kind": "rust_client_request_metrics",
            "rows": rows
        })
    }
}

impl ClientMetrics for InMemoryMetrics {
    fn observe_request(&self, endpoint: &str, elapsed_millis: u128, ok: bool) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.push((endpoint.to_string(), elapsed_millis, ok));
        }
    }
}
