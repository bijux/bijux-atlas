use super::*;

impl DatasetCacheManager {
    pub async fn startup_warmup(self: &Arc<Self>) -> Result<(), CacheError> {
        ensure_secure_dir(&self.cfg.disk_root)?;
        let mut warm = self.cfg.startup_warmup.clone();
        warm.sort_by_key(DatasetId::canonical_string);
        warm.dedup();
        let bounded = warm
            .into_iter()
            .take(
                self.cfg
                    .startup_warmup_limit
                    .min(self.cfg.max_dataset_count),
            )
            .collect::<Vec<_>>();
        for ds in &bounded {
            let result = self.ensure_dataset_cached(ds).await;
            if let Err(e) = result {
                if self.cfg.fail_readiness_on_missing_warmup {
                    return Err(CacheError(format!("warmup failed for {:?}: {}", ds, e)));
                }
                error!("warmup error for {:?}: {}", ds, e);
            }
        }
        Ok(())
    }

    pub fn spawn_background_tasks(self: &Arc<Self>) {
        let me = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(me.cfg.eviction_check_interval);
            loop {
                interval.tick().await;
                if let Err(e) = me.evict_background().await {
                    error!("eviction error: {e}");
                }
            }
        });
        let me = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(me.cfg.integrity_reverify_interval);
            loop {
                interval.tick().await;
                if let Err(e) = me.reverify_cached_datasets().await {
                    error!("reverify error: {e}");
                }
            }
        });
    }

    pub async fn refresh_catalog(&self) -> Result<(), CacheError> {
        if self.cfg.registry_freeze_mode {
            return Ok(());
        }
        let etag = {
            let cache = self.catalog_cache.lock().await;
            let now = Instant::now();
            if let Some(last) = cache.refreshed_at {
                if now.duration_since(last) < self.cfg.registry_ttl {
                    return Ok(());
                }
            }
            if let Some(until) = cache.breaker_open_until {
                if now < until {
                    return Err(CacheError("catalog circuit breaker open".to_string()));
                }
            }
            if let Some(until) = cache.backoff_until {
                if now < until {
                    return Err(CacheError("catalog backoff active".to_string()));
                }
            }
            cache.etag.clone()
        };
        let fetch_result = self
            .store
            .fetch_catalog(etag.as_deref())
            .instrument(tracing::info_span!(
                "store_catalog_fetch",
                backend = self.store.backend_tag()
            ))
            .await;
        let result = match fetch_result {
            Err(e) => Err(e),
            Ok(CatalogFetch::NotModified) => {
                let mut lock = self.catalog_cache.lock().await;
                lock.consecutive_errors = 0;
                lock.backoff_until = None;
                lock.breaker_open_until = None;
                lock.refreshed_at = Some(Instant::now());
                drop(lock);
                let health = self.store.registry_health().await;
                let mut h = self.registry_health_cache.write().await;
                *h = health;
                Ok(())
            }
            Ok(CatalogFetch::Updated { etag, catalog }) => {
                let epoch_hash = sha256_hex(
                    &serde_json::to_vec(&catalog).map_err(|e| CacheError(e.to_string()))?,
                );
                let old_epoch = self.metrics.catalog_epoch_hash.read().await.clone();
                {
                    let mut lock = self.catalog_cache.lock().await;
                    lock.etag = Some(etag);
                    lock.catalog = Some(catalog);
                    lock.consecutive_errors = 0;
                    lock.backoff_until = None;
                    lock.breaker_open_until = None;
                    lock.refreshed_at = Some(Instant::now());
                }
                {
                    let mut e = self.metrics.catalog_epoch_hash.write().await;
                    *e = epoch_hash.clone();
                }
                if !old_epoch.is_empty() && old_epoch != epoch_hash {
                    self.metrics
                        .registry_invalidation_events_total
                        .fetch_add(1, Ordering::Relaxed);
                }
                let health = self.store.registry_health().await;
                let mut h = self.registry_health_cache.write().await;
                *h = health;
                info!("catalog epoch updated: {epoch_hash}");
                Ok(())
            }
        };

        if let Err(err) = result {
            self.metrics
                .registry_refresh_failures_total
                .fetch_add(1, Ordering::Relaxed);
            let mut lock = self.catalog_cache.lock().await;
            lock.consecutive_errors = lock.consecutive_errors.saturating_add(1);
            let backoff_ms = self
                .cfg
                .catalog_backoff_base_ms
                .saturating_mul(lock.consecutive_errors as u64)
                .min(5_000);
            lock.backoff_until = Some(Instant::now() + Duration::from_millis(backoff_ms));
            if lock.consecutive_errors >= self.cfg.catalog_breaker_failure_threshold {
                lock.breaker_open_until =
                    Some(Instant::now() + Duration::from_millis(self.cfg.catalog_breaker_open_ms));
            }
            return Err(err);
        }

        Ok(())
    }

    pub async fn catalog_epoch(&self) -> String {
        self.metrics.catalog_epoch_hash.read().await.clone()
    }

    pub fn cached_only_mode(&self) -> bool {
        self.cfg.cached_only_mode
    }

    pub fn registry_freeze_mode(&self) -> bool {
        self.cfg.registry_freeze_mode
    }

    pub fn registry_ttl_seconds(&self) -> u64 {
        self.cfg.registry_ttl.as_secs()
    }

    pub async fn current_catalog(&self) -> Option<Catalog> {
        self.catalog_cache.lock().await.catalog.clone()
    }

    pub async fn registry_health(&self) -> Vec<RegistrySourceHealth> {
        self.registry_health_cache.read().await.clone()
    }

    pub async fn registry_refresh_age_seconds(&self) -> u64 {
        let cache = self.catalog_cache.lock().await;
        match cache.refreshed_at {
            Some(last) => Instant::now().duration_since(last).as_secs(),
            None => u64::MAX,
        }
    }

    pub async fn cached_datasets_debug(&self) -> Vec<(String, u64)> {
        let entries = self.entries.lock().await;
        let mut out: Vec<(String, u64)> = entries
            .iter()
            .map(|(id, e)| {
                (
                    format!("{}/{}/{}", id.release, id.species, id.assembly),
                    e.size_bytes,
                )
            })
            .collect();
        out.sort();
        out
    }

    pub async fn fetch_manifest_summary(
        &self,
        dataset: &DatasetId,
    ) -> Result<ArtifactManifest, CacheError> {
        self.store.fetch_manifest(dataset).await
    }

    pub async fn dataset_health_snapshot(
        &self,
        dataset: &DatasetId,
    ) -> Result<DatasetHealthSnapshot, CacheError> {
        let open_failures = {
            let breakers = self.breakers.lock().await;
            breakers.get(dataset).map_or(0, |b| b.failure_count)
        };
        let quarantined = {
            let q = self.quarantined.lock().await;
            q.contains(dataset)
        };
        let (cached, last_open_seconds_ago, size_bytes) = {
            let entries = self.entries.lock().await;
            if let Some(entry) = entries.get(dataset) {
                (
                    true,
                    Some(entry.last_access.elapsed().as_secs()),
                    Some(entry.size_bytes),
                )
            } else {
                (false, None, None)
            }
        };
        let checksum_verified = if cached {
            self.verify_dataset_integrity_strict(dataset).await?
        } else {
            false
        };
        Ok(DatasetHealthSnapshot {
            cached,
            checksum_verified,
            last_open_seconds_ago,
            size_bytes,
            open_failures,
            quarantined,
        })
    }
}
