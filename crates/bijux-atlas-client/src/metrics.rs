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
    pub fn snapshot(&self) -> Vec<(String, u128, bool)> {
        match self.inner.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    }
}

impl ClientMetrics for InMemoryMetrics {
    fn observe_request(&self, endpoint: &str, elapsed_millis: u128, ok: bool) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.push((endpoint.to_string(), elapsed_millis, ok));
        }
    }
}
