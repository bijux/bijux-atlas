// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::dataset::DatasetId;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatasetCacheConfig {
    pub disk_root: PathBuf,
    pub max_disk_bytes: u64,
    pub disk_high_watermark_pct: u8,
    pub disk_low_watermark_pct: u8,
    pub max_dataset_count: usize,
    pub idle_ttl: Duration,
    pub pinned_datasets: HashSet<DatasetId>,
    pub read_only_fs: bool,
    pub cached_only_mode: bool,
    pub startup_warmup: Vec<DatasetId>,
    pub startup_warmup_limit: usize,
    pub fail_readiness_on_missing_warmup: bool,
    pub max_connections_per_dataset: usize,
    pub max_total_connections: usize,
    pub dataset_open_timeout: Duration,
    pub breaker_failure_threshold: u32,
    pub breaker_open_duration: Duration,
    pub store_breaker_failure_threshold: u32,
    pub store_breaker_open_duration: Duration,
    pub store_retry_budget: u32,
    pub max_concurrent_downloads: usize,
    pub max_concurrent_downloads_node: Option<usize>,
    pub eviction_check_interval: Duration,
    pub integrity_reverify_interval: Duration,
    pub sqlite_pragma_cache_kib: i64,
    pub sqlite_pragma_mmap_bytes: i64,
    pub max_open_shards_per_pod: usize,
    pub startup_warmup_jitter_max_ms: u64,
    pub catalog_backoff_base_ms: u64,
    pub catalog_breaker_failure_threshold: u32,
    pub catalog_breaker_open_ms: u64,
    pub quarantine_after_corruption_failures: u32,
    pub quarantine_retry_ttl: Duration,
    pub registry_ttl: Duration,
    pub registry_freeze_mode: bool,
}

impl Default for DatasetCacheConfig {
    fn default() -> Self {
        Self {
            disk_root: super::default_runtime_cache_root(),
            max_disk_bytes: 4 * 1024 * 1024 * 1024,
            disk_high_watermark_pct: 90,
            disk_low_watermark_pct: 75,
            max_dataset_count: 8,
            idle_ttl: Duration::from_secs(1800),
            pinned_datasets: HashSet::new(),
            read_only_fs: false,
            cached_only_mode: false,
            startup_warmup: Vec::new(),
            startup_warmup_limit: 8,
            fail_readiness_on_missing_warmup: false,
            max_connections_per_dataset: 8,
            max_total_connections: 64,
            dataset_open_timeout: Duration::from_secs(3),
            breaker_failure_threshold: 3,
            breaker_open_duration: Duration::from_secs(30),
            store_breaker_failure_threshold: 5,
            store_breaker_open_duration: Duration::from_secs(20),
            store_retry_budget: 20,
            max_concurrent_downloads: 3,
            max_concurrent_downloads_node: None,
            eviction_check_interval: Duration::from_secs(30),
            integrity_reverify_interval: Duration::from_secs(300),
            sqlite_pragma_cache_kib: 32 * 1024,
            sqlite_pragma_mmap_bytes: 256 * 1024 * 1024,
            max_open_shards_per_pod: 16,
            startup_warmup_jitter_max_ms: 0,
            catalog_backoff_base_ms: 250,
            catalog_breaker_failure_threshold: 5,
            catalog_breaker_open_ms: 5000,
            quarantine_after_corruption_failures: 3,
            quarantine_retry_ttl: Duration::from_secs(300),
            registry_ttl: Duration::from_secs(15),
            registry_freeze_mode: false,
        }
    }
}
