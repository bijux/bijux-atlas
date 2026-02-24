// SPDX-License-Identifier: Apache-2.0

impl DatasetCacheManager {
    async fn reverify_cached_datasets(&self) -> Result<(), CacheError> {
        let datasets: Vec<DatasetId> = {
            let entries = self.entries.lock().await;
            entries.keys().cloned().collect()
        };
        for dataset in datasets {
            if !self.verify_dataset_integrity_strict(&dataset).await? {
                warn!(dataset = ?dataset, "cached dataset failed re-verification");
                self.record_corruption_failure(&dataset).await;
                let mut entries = self.entries.lock().await;
                if let Some(entry) = entries.remove(&dataset) {
                    let _ = std::fs::remove_file(&entry.sqlite_path);
                    for shard in &entry.shard_sqlite_paths {
                        let _ = std::fs::remove_file(shard);
                    }
                }
            }
        }
        Ok(())
    }

    async fn check_quarantine(&self, dataset: &DatasetId) -> Result<(), CacheError> {
        if self.cfg.quarantine_retry_ttl > Duration::from_secs(0) {
            let mut breakers = self.breakers.lock().await;
            if let Some(state) = breakers.get_mut(dataset) {
                if let Some(until) = state.open_until {
                    if Instant::now() >= until {
                        state.open_until = None;
                        state.failure_count = 0;
                        self.quarantined.lock().await.remove(dataset);
                    }
                }
            }
        }
        let quarantined = self.quarantined.lock().await;
        if quarantined.contains(dataset) {
            return Err(CacheError("dataset is quarantined".to_string()));
        }
        Ok(())
    }

    async fn record_corruption_failure(&self, dataset: &DatasetId) {
        let mut failures = self.quarantine_failures.lock().await;
        let count = failures.entry(dataset.clone()).or_insert(0);
        *count += 1;
        if self.cfg.quarantine_after_corruption_failures > 0
            && *count >= self.cfg.quarantine_after_corruption_failures
        {
            drop(failures);
            self.quarantined.lock().await.insert(dataset.clone());
            let mut breakers = self.breakers.lock().await;
            let state = breakers.entry(dataset.clone()).or_default();
            state.open_until = Some(Instant::now() + self.cfg.quarantine_retry_ttl);
        }
    }

    async fn verify_dataset_integrity_strict(
        &self,
        dataset: &DatasetId,
    ) -> Result<bool, CacheError> {
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if !paths.sqlite.exists() || !paths.manifest.exists() {
            return Ok(false);
        }
        let manifest_raw = std::fs::read(&paths.manifest).map_err(|e| CacheError(e.to_string()))?;
        let manifest: ArtifactManifest =
            serde_json::from_slice(&manifest_raw).map_err(|e| CacheError(e.to_string()))?;
        self.metrics
            .verify_full_hash_checks
            .fetch_add(1, Ordering::Relaxed);
        let sqlite_hash =
            sha256_hex(&std::fs::read(&paths.sqlite).map_err(|e| CacheError(e.to_string()))?);
        Ok(sqlite_hash == manifest.checksums.sqlite_sha256)
    }

    async fn check_breaker(&self, dataset: &DatasetId) -> Result<(), CacheError> {
        let mut lock = self.breakers.lock().await;
        let state = lock.entry(dataset.clone()).or_default();
        if let Some(until) = state.open_until {
            if Instant::now() < until {
                return Err(CacheError("dataset circuit breaker open".to_string()));
            }
        }
        Ok(())
    }

    async fn record_open_failure(&self, dataset: &DatasetId) {
        let mut lock = self.breakers.lock().await;
        let state = lock.entry(dataset.clone()).or_default();
        state.failure_count += 1;
        if state.failure_count >= self.cfg.breaker_failure_threshold {
            state.open_until = Some(Instant::now() + self.cfg.breaker_open_duration);
        }
    }

    async fn reset_breaker(&self, dataset: &DatasetId) {
        let mut lock = self.breakers.lock().await;
        let state = lock.entry(dataset.clone()).or_default();
        state.failure_count = 0;
        state.open_until = None;
    }
}
