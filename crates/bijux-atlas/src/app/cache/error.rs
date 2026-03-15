// SPDX-License-Identifier: Apache-2.0

#[derive(Debug)]
pub struct CacheError(pub String);

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CacheError {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistrySourceHealth {
    pub name: String,
    pub priority: u32,
    pub healthy: bool,
    pub last_error: Option<String>,
    pub shadowed_datasets: u64,
    pub ttl_seconds: u64,
}
