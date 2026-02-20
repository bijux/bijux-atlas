use crate::{CacheError, DatasetCacheManager};
use bijux_atlas_model::DatasetId;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl DatasetCacheManager {
    pub async fn prefetch_dataset(&self, dataset: DatasetId) -> Result<(), CacheError> {
        self.ensure_dataset_cached(&dataset).await
    }

    pub fn is_pinned_dataset(&self, dataset: &DatasetId) -> bool {
        self.cfg.pinned_datasets.contains(dataset)
    }

    pub(super) async fn check_store_breaker(&self) -> Result<(), CacheError> {
        let mut lock = self.store_breaker.lock().await;
        if let Some(until) = lock.open_until {
            if Instant::now() < until {
                return Err(CacheError("store circuit breaker open".to_string()));
            }
            lock.open_until = None;
            self.metrics
                .store_breaker_half_open_total
                .fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    pub async fn store_breaker_is_open(&self) -> bool {
        let lock = self.store_breaker.lock().await;
        lock.open_until
            .map(|until| Instant::now() < until)
            .unwrap_or(false)
    }

    pub(super) async fn record_store_download_failure(&self, backend: &str, reason: &str) {
        self.metrics
            .store_download_failures
            .fetch_add(1, Ordering::Relaxed);
        let reason_lower = reason.to_ascii_lowercase();
        let class = if reason_lower.contains("checksum") {
            self.metrics
                .store_error_checksum_total
                .fetch_add(1, Ordering::Relaxed);
            "checksum"
        } else if reason_lower.contains("timeout") {
            self.metrics
                .store_error_timeout_total
                .fetch_add(1, Ordering::Relaxed);
            "timeout"
        } else if reason_lower.contains("network")
            || reason_lower.contains("connection")
            || reason_lower.contains("download")
        {
            self.metrics
                .store_error_network_total
                .fetch_add(1, Ordering::Relaxed);
            "network"
        } else {
            self.metrics
                .store_error_other_total
                .fetch_add(1, Ordering::Relaxed);
            "other"
        };
        let mut by = self.metrics.store_errors_by_backend_and_class.lock().await;
        *by.entry((backend.to_string(), class.to_string()))
            .or_insert(0) += 1;
        let remaining = self.retry_budget_remaining.load(Ordering::Relaxed);
        if remaining > 0 {
            self.retry_budget_remaining
                .store(remaining.saturating_sub(1), Ordering::Relaxed);
        }
        let mut lock = self.store_breaker.lock().await;
        lock.failure_count += 1;
        if lock.failure_count >= self.cfg.store_breaker_failure_threshold {
            lock.open_until = Some(Instant::now() + self.cfg.store_breaker_open_duration);
            self.metrics
                .store_breaker_open_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(super) async fn reset_store_breaker(&self) {
        let mut lock = self.store_breaker.lock().await;
        lock.failure_count = 0;
        lock.open_until = None;
    }
}
