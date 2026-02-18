impl DatasetCacheManager {
    pub fn new(cfg: DatasetCacheConfig, store: Arc<dyn DatasetStoreBackend>) -> Arc<Self> {
        let max_concurrent_downloads = cfg.max_concurrent_downloads;
        let retry_budget = cfg.store_retry_budget as u64;
        Arc::new(Self {
            global_semaphore: Arc::new(Semaphore::new(cfg.max_total_connections)),
            shard_open_semaphore: Arc::new(Semaphore::new(cfg.max_open_shards_per_pod)),
            cfg,
            store,
            entries: Mutex::new(HashMap::new()),
            inflight: Mutex::new(HashMap::new()),
            breakers: Mutex::new(HashMap::new()),
            quarantine_failures: Mutex::new(HashMap::new()),
            quarantined: Mutex::new(HashSet::new()),
            store_breaker: Mutex::new(StoreBreakerState::default()),
            catalog_cache: Mutex::new(CatalogCache::default()),
            registry_health_cache: RwLock::new(Vec::new()),
            download_semaphore: Arc::new(Semaphore::new(max_concurrent_downloads)),
            retry_budget_remaining: AtomicU64::new(retry_budget),
            dataset_retry_budget: Mutex::new(HashMap::new()),
            metrics: Arc::new(CacheMetrics::default()),
        })
    }

    pub async fn startup_warmup(self: &Arc<Self>) -> Result<(), CacheError> {
        std::fs::create_dir_all(&self.cfg.disk_root).map_err(|e| CacheError(e.to_string()))?;
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
        let fetch_result = self.store.fetch_catalog(etag.as_deref()).await;
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

    pub async fn ensure_sequence_inputs_cached(
        &self,
        dataset: &DatasetId,
    ) -> Result<(PathBuf, PathBuf), CacheError> {
        self.ensure_dataset_cached(dataset).await?;
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if paths.fasta.exists() && paths.fai.exists() {
            return Ok((paths.fasta, paths.fai));
        }
        if self.cfg.cached_only_mode {
            return Err(CacheError(
                "sequence inputs missing from cache and cached-only mode is enabled".to_string(),
            ));
        }
        if self.cfg.read_only_fs {
            return Err(CacheError(
                "sequence inputs missing from cache and read-only filesystem mode is enabled"
                    .to_string(),
            ));
        }
        let manifest = self.store.fetch_manifest(dataset).await?;
        let fasta = self.store.fetch_fasta_bytes(dataset).await?;
        let fai = self.store.fetch_fai_bytes(dataset).await?;
        if sha256_hex(&fasta) != manifest.checksums.fasta_sha256 {
            return Err(CacheError("fasta checksum verification failed".to_string()));
        }
        if sha256_hex(&fai) != manifest.checksums.fai_sha256 {
            return Err(CacheError("fai checksum verification failed".to_string()));
        }
        std::fs::create_dir_all(&paths.inputs_dir).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.fasta, fasta).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.fai, fai).map_err(|e| CacheError(e.to_string()))?;
        Ok((paths.fasta, paths.fai))
    }

    pub async fn ensure_release_gene_index_cached(
        &self,
        dataset: &DatasetId,
    ) -> Result<PathBuf, CacheError> {
        self.ensure_dataset_cached(dataset).await?;
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if paths.release_gene_index.exists() {
            return Ok(paths.release_gene_index);
        }
        if self.cfg.cached_only_mode {
            return Err(CacheError(
                "release gene index missing from cache and cached-only mode is enabled".to_string(),
            ));
        }
        if self.cfg.read_only_fs {
            return Err(CacheError(
                "release gene index missing from cache and read-only filesystem mode is enabled"
                    .to_string(),
            ));
        }
        let bytes = self.store.fetch_release_gene_index_bytes(dataset).await?;
        std::fs::create_dir_all(&paths.derived_dir).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.release_gene_index, bytes).map_err(|e| CacheError(e.to_string()))?;
        Ok(paths.release_gene_index)
    }

    pub async fn open_dataset_connection(
        &self,
        dataset: &DatasetId,
    ) -> Result<DatasetConnection, CacheError> {
        info!(dataset = ?dataset, "dataset open start");
        let open_started = Instant::now();
        self.check_quarantine(dataset).await?;
        self.ensure_dataset_cached(dataset).await?;

        self.check_breaker(dataset).await?;

        let (sqlite_path, dataset_sem, query_sem) = {
            let mut entries = self.entries.lock().await;
            let entry = entries
                .get_mut(dataset)
                .ok_or_else(|| CacheError("dataset cache entry missing".to_string()))?;
            entry.last_access = Instant::now();
            (
                entry.sqlite_path.clone(),
                Arc::clone(&entry.dataset_semaphore),
                Arc::clone(&entry.query_semaphore),
            )
        };

        let global_permit = self
            .global_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        let dataset_permit = dataset_sem
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        let query_permit = query_sem
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;

        let open = timeout(self.cfg.dataset_open_timeout, async move {
            tokio::task::spawn_blocking(move || {
                crate::effect_adapters::sqlite_adapters::open_readonly_no_mutex(&sqlite_path)
            })
            .await
            .map_err(|e| CacheError(e.to_string()))?
            .map_err(|e| CacheError(e.to_string()))
        })
        .await;

        match open {
            Ok(Ok(conn)) => {
                conn.set_prepared_statement_cache_capacity(128);
                prime_prepared_statements(&conn);
                let _ = crate::effect_adapters::sqlite_adapters::apply_readonly_pragmas(
                    &conn,
                    self.cfg.sqlite_pragma_cache_kib,
                    self.cfg.sqlite_pragma_mmap_bytes,
                );
                self.reset_breaker(dataset).await;
                self.metrics
                    .store_open_latency_ns
                    .lock()
                    .await
                    .push(open_started.elapsed().as_nanos() as u64);
                Ok(DatasetConnection {
                    conn,
                    _global_permit: global_permit,
                    _dataset_permit: dataset_permit,
                    _query_permit: query_permit,
                })
            }
            Ok(Err(e)) => {
                self.record_open_failure(dataset).await;
                self.metrics
                    .store_open_failures
                    .fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(_) => {
                self.record_open_failure(dataset).await;
                self.metrics
                    .store_open_failures
                    .fetch_add(1, Ordering::Relaxed);
                Err(CacheError("dataset open timeout".to_string()))
            }
        }
    }

    async fn ensure_dataset_cached(&self, dataset: &DatasetId) -> Result<(), CacheError> {
        self.check_quarantine(dataset).await?;
        if self.is_cached_and_verified(dataset).await? {
            self.metrics
                .dataset_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }
        self.metrics
            .dataset_misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if self.cfg.cached_only_mode {
            return Err(CacheError(
                "dataset missing from cache and cached-only mode is enabled".to_string(),
            ));
        }
        if self.cfg.read_only_fs {
            return Err(CacheError(
                "dataset missing from cache and read-only filesystem mode is enabled".to_string(),
            ));
        }

        let lock = {
            let mut map = self.inflight.lock().await;
            Arc::clone(
                map.entry(dataset.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            )
        };

        let _guard = lock.lock().await;

        if self.is_cached_and_verified(dataset).await? {
            return Ok(());
        }

        self.check_store_breaker().await?;
        let remaining = self.retry_budget_remaining.load(Ordering::Relaxed);
        if remaining == 0 {
            self.metrics
                .store_retry_budget_exhausted_total
                .fetch_add(1, Ordering::Relaxed);
            return Err(CacheError(
                "store retry budget exhausted; refusing download".to_string(),
            ));
        }
        let mut dataset_budget = self.dataset_retry_budget.lock().await;
        let per_dataset_remaining = dataset_budget
            .entry(dataset.clone())
            .or_insert(self.cfg.store_retry_budget);
        if *per_dataset_remaining == 0 {
            self.metrics
                .store_retry_budget_exhausted_total
                .fetch_add(1, Ordering::Relaxed);
            return Err(CacheError(
                "dataset retry budget exhausted; refusing download".to_string(),
            ));
        }
        if *per_dataset_remaining < self.cfg.store_retry_budget {
            self.metrics
                .store_download_retry_total
                .fetch_add(1, Ordering::Relaxed);
        }
        *per_dataset_remaining = per_dataset_remaining.saturating_sub(1);
        drop(dataset_budget);

        info!(dataset = ?dataset, "dataset download path");
        let started = Instant::now();
        info!("dataset download start {:?}", dataset);
        let _download_permit = self
            .download_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        let manifest = match self.store.fetch_manifest(dataset).await {
            Ok(v) => v,
            Err(e) => {
                self.record_store_download_failure(&e.to_string()).await;
                return Err(e);
            }
        };
        self.metrics
            .store_download_ttfb_ns
            .lock()
            .await
            .push(started.elapsed().as_nanos() as u64);
        let sqlite = match self.store.fetch_sqlite_bytes(dataset).await {
            Ok(v) => v,
            Err(e) => {
                self.record_store_download_failure(&e.to_string()).await;
                return Err(e);
            }
        };
        let release_gene_index = self
            .store
            .fetch_release_gene_index_bytes(dataset)
            .await
            .ok();
        let sqlite_hash = sha256_hex(&sqlite);
        if sqlite_hash != manifest.checksums.sqlite_sha256 {
            error!("dataset verify failed {:?}", dataset);
            self.metrics
                .store_download_failures
                .fetch_add(1, Ordering::Relaxed);
            self.record_store_download_failure("checksum verification failed")
                .await;
            return Err(CacheError(
                "sqlite checksum verification failed".to_string(),
            ));
        }

        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        std::fs::create_dir_all(&paths.derived_dir).map_err(|e| CacheError(e.to_string()))?;

        let tmp_dir = self.cfg.disk_root.join(".tmp-atlas-download");
        std::fs::create_dir_all(&tmp_dir).map_err(|e| CacheError(e.to_string()))?;

        let tmp_sqlite = tmp_dir.join("gene_summary.sqlite.tmp");
        std::fs::write(&tmp_sqlite, &sqlite).map_err(|e| CacheError(e.to_string()))?;
        std::fs::rename(&tmp_sqlite, &paths.sqlite).map_err(|e| CacheError(e.to_string()))?;
        if let Some(index_bytes) = release_gene_index {
            std::fs::write(&paths.release_gene_index, index_bytes)
                .map_err(|e| CacheError(e.to_string()))?;
        }

        let manifest_bytes =
            serde_json::to_vec(&manifest).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.manifest, manifest_bytes).map_err(|e| CacheError(e.to_string()))?;

        let marker = format!(
            "{}:{}",
            manifest.checksums.sqlite_sha256, manifest.db_schema_version
        );
        std::fs::write(paths.derived_dir.join(".verified"), marker.as_bytes())
            .map_err(|e| CacheError(e.to_string()))?;

        let size_bytes = std::fs::metadata(&paths.sqlite)
            .map_err(|e| CacheError(e.to_string()))?
            .len();
        let (shard_sqlite_paths, shard_by_seqid) =
            dataset_shards::load_shard_catalog(&paths.derived_dir)?;
        let download_latency_ns = started.elapsed().as_nanos() as u64;

        {
            let mut entries = self.entries.lock().await;
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: paths.sqlite,
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes,
                    last_download_latency_ns: download_latency_ns,
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                    query_semaphore: Arc::new(Semaphore::new(self.cfg.max_connections_per_dataset)),
                },
            );
            self.metrics
                .dataset_count
                .store(entries.len() as u64, std::sync::atomic::Ordering::Relaxed);
            let usage = entries.values().map(|e| e.size_bytes).sum::<u64>();
            self.metrics
                .disk_usage_bytes
                .store(usage, std::sync::atomic::Ordering::Relaxed);
        }

        self.metrics
            .store_download_latency_ns
            .lock()
            .await
            .push(download_latency_ns);
        self.metrics
            .store_download_bytes_total
            .fetch_add(size_bytes, Ordering::Relaxed);
        self.retry_budget_remaining
            .store(self.cfg.store_retry_budget as u64, Ordering::Relaxed);
        let mut dataset_budget = self.dataset_retry_budget.lock().await;
        dataset_budget.insert(dataset.clone(), self.cfg.store_retry_budget);
        self.reset_store_breaker().await;
        info!("dataset download complete {:?}", dataset);
        Ok(())
    }

    async fn is_cached_and_verified(&self, dataset: &DatasetId) -> Result<bool, CacheError> {
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if !paths.sqlite.exists() || !paths.manifest.exists() {
            return Ok(false);
        }

        let manifest_raw = std::fs::read(&paths.manifest).map_err(|e| CacheError(e.to_string()))?;
        let manifest: ArtifactManifest =
            serde_json::from_slice(&manifest_raw).map_err(|e| CacheError(e.to_string()))?;

        let marker_expected = format!(
            "{}:{}",
            manifest.checksums.sqlite_sha256, manifest.db_schema_version
        );
        let marker_path = paths.derived_dir.join(".verified");
        let marker_ok = marker_path.exists()
            && std::fs::read_to_string(&marker_path)
                .map(|s| s == marker_expected)
                .unwrap_or(false);

        if marker_ok {
            self.metrics
                .verify_marker_fast_path_hits
                .fetch_add(1, Ordering::Relaxed);
            let (shard_sqlite_paths, shard_by_seqid) =
                dataset_shards::load_shard_catalog(&paths.derived_dir)?;
            let mut entries = self.entries.lock().await;
            entries
                .entry(dataset.clone())
                .or_insert_with(|| DatasetEntry {
                    sqlite_path: paths.sqlite.clone(),
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes: std::fs::metadata(&paths.sqlite)
                        .map(|m| m.len())
                        .unwrap_or(0),
                    last_download_latency_ns: 1_000_000,
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                    query_semaphore: Arc::new(Semaphore::new(self.cfg.max_connections_per_dataset)),
                });
            return Ok(true);
        }

        self.metrics
            .verify_full_hash_checks
            .fetch_add(1, Ordering::Relaxed);
        let sqlite_hash =
            sha256_hex(&std::fs::read(&paths.sqlite).map_err(|e| CacheError(e.to_string()))?);
        if sqlite_hash == manifest.checksums.sqlite_sha256 {
            std::fs::write(marker_path, marker_expected.as_bytes())
                .map_err(|e| CacheError(e.to_string()))?;
            let (shard_sqlite_paths, shard_by_seqid) =
                dataset_shards::load_shard_catalog(&paths.derived_dir)?;
            let mut entries = self.entries.lock().await;
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: paths.sqlite,
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes: std::fs::metadata(paths.derived_dir.join("gene_summary.sqlite"))
                        .map(|m| m.len())
                        .unwrap_or(0),
                    last_download_latency_ns: 1_000_000,
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                    query_semaphore: Arc::new(Semaphore::new(self.cfg.max_connections_per_dataset)),
                },
            );
            return Ok(true);
        }
        Ok(false)
    }

    async fn evict_background(&self) -> Result<(), CacheError> {
        let disk_io_started = Instant::now();
        let now = Instant::now();
        let mut entries = self.entries.lock().await;

        let mut victims: Vec<DatasetId> = entries
            .iter()
            .filter_map(|(id, e)| {
                if self.cfg.pinned_datasets.contains(id) {
                    return None;
                }
                if now.duration_since(e.last_access) > self.cfg.idle_ttl {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut total_size: u64 = entries.values().map(|e| e.size_bytes).sum();
        if entries.len() > self.cfg.max_dataset_count || total_size > self.cfg.max_disk_bytes {
            let mut ranked: Vec<(DatasetId, f64)> = entries
                .iter()
                .filter(|(id, _)| !self.cfg.pinned_datasets.contains(*id))
                .map(|(id, e)| {
                    let age = now.duration_since(e.last_access).as_secs_f64().max(1.0);
                    let redownload_cost = (e.last_download_latency_ns as f64).max(1.0);
                    let score = age * (e.size_bytes as f64) / redownload_cost;
                    (id.clone(), score)
                })
                .collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            for (id, _) in ranked {
                if entries.len() <= self.cfg.max_dataset_count
                    && total_size <= self.cfg.max_disk_bytes
                {
                    break;
                }
                victims.push(id.clone());
                if let Some(e) = entries.get(&id) {
                    total_size = total_size.saturating_sub(e.size_bytes);
                }
            }
        }

        victims.sort();
        victims.dedup();
        for id in victims {
            if let Some(entry) = entries.remove(&id) {
                let _ = std::fs::remove_file(&entry.sqlite_path);
                for shard in &entry.shard_sqlite_paths {
                    let _ = std::fs::remove_file(shard);
                }
                let _ = std::fs::remove_file(
                    entry
                        .sqlite_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join("manifest.json"),
                );
                let _ = std::fs::remove_file(
                    entry
                        .sqlite_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(".verified"),
                );
                info!("dataset evicted {:?}", id);
            }
        }

        self.metrics
            .dataset_count
            .store(entries.len() as u64, std::sync::atomic::Ordering::Relaxed);
        self.metrics.disk_usage_bytes.store(
            entries.values().map(|e| e.size_bytes).sum::<u64>(),
            std::sync::atomic::Ordering::Relaxed,
        );
        let usage = self.metrics.disk_usage_bytes.load(Ordering::Relaxed);
        if self.cfg.max_disk_bytes > 0 && usage.saturating_mul(100) / self.cfg.max_disk_bytes >= 90
        {
            self.metrics
                .fs_space_pressure_events_total
                .fetch_add(1, Ordering::Relaxed);
        }
        self.metrics
            .disk_io_latency_ns
            .lock()
            .await
            .push(disk_io_started.elapsed().as_nanos() as u64);

        Ok(())
    }
}

fn prime_prepared_statements(conn: &Connection) {
    let hot_sql = [
        "SELECT gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length FROM gene_summary WHERE gene_id = ?1 ORDER BY gene_id LIMIT ?2",
        "SELECT gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length FROM gene_summary WHERE biotype = ?1 ORDER BY gene_id LIMIT ?2",
        "SELECT g.gene_id, g.name, g.seqid, g.start, g.end, g.biotype, g.transcript_count, g.sequence_length FROM gene_summary g JOIN gene_summary_rtree r ON r.gene_rowid = g.id WHERE g.seqid = ?1 AND r.start <= ?2 AND r.end >= ?3 ORDER BY g.seqid, g.start, g.gene_id LIMIT ?4",
        "SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary WHERE transcript_id=?1 LIMIT 1",
        "SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary WHERE parent_gene_id = ? ORDER BY seqid ASC, start ASC, transcript_id ASC LIMIT ?",
        "SELECT seqid,start,end FROM gene_summary WHERE gene_id = ?1 LIMIT 1",
    ];
    for sql in hot_sql {
        let _ = conn.prepare_cached(sql);
    }
}
