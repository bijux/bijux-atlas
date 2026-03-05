// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout_millis: u64,
    pub retry_attempts: u32,
    pub retry_backoff_millis: u64,
    pub default_headers: BTreeMap<String, String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
            timeout_millis: 5_000,
            retry_attempts: 2,
            retry_backoff_millis: 150,
            default_headers: BTreeMap::new(),
        }
    }
}

impl ClientConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err("base_url must start with http:// or https://".to_string());
        }
        if self.timeout_millis == 0 {
            return Err("timeout_millis must be > 0".to_string());
        }
        Ok(())
    }
}
