impl DatasetCacheManager {
    pub fn new(cfg: DatasetCacheConfig, store: Arc<dyn DatasetStoreBackend>) -> Arc<Self> {
        let max_concurrent_downloads = cfg
            .max_concurrent_downloads_node
            .map(|node| node.min(cfg.max_concurrent_downloads))
            .unwrap_or(cfg.max_concurrent_downloads);
        let retry_budget = cfg.store_retry_budget as u64;
        let _ = ensure_secure_dir(&cfg.disk_root);
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

    pub async fn ensure_sequence_inputs_cached(
        &self,
        dataset: &DatasetId,
    ) -> Result<(PathBuf, PathBuf), CacheError> {
        self.ensure_dataset_cached(dataset).await?;
        let paths = self.resolve_cache_paths(dataset).await?;
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
        write_atomic_file(&paths.fasta, &fasta)?;
        write_atomic_file(&paths.fai, &fai)?;
        Ok((paths.fasta, paths.fai))
    }

    pub async fn ensure_release_gene_index_cached(
        &self,
        dataset: &DatasetId,
    ) -> Result<PathBuf, CacheError> {
        self.ensure_dataset_cached(dataset).await?;
        let paths = self.resolve_cache_paths(dataset).await?;
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
        ensure_secure_dir(&paths.derived_dir)?;
        write_atomic_file(&paths.release_gene_index, &bytes)?;
        Ok(paths.release_gene_index)
    }

    pub async fn open_dataset_connection(
        &self,
        dataset: &DatasetId,
    ) -> Result<DatasetConnection, CacheError> {
        info!(dataset = ?dataset, "dataset open start");
        let open_started = Instant::now();
        self.check_quarantine(dataset).await?;
        async { self.ensure_dataset_cached(dataset).await }
            .instrument(tracing::info_span!(
                "cache_lookup",
                dataset = %dataset.canonical_string()
            ))
            .await?;

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
        .instrument(tracing::info_span!(
            "open_db",
            dataset = %dataset.canonical_string()
        ))
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
        let missing_bucket = sha256_hex(dataset.canonical_string().as_bytes())
            .chars()
            .take(8)
            .collect::<String>();
        let mut by_hash = self.metrics.dataset_missing_by_hash_bucket.lock().await;
        *by_hash.entry(missing_bucket).or_insert(0) += 1;
        drop(by_hash);

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
        info!(dataset = ?dataset, "dataset download start");
        let _download_permit = self
            .download_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        let (manifest, sqlite, release_gene_index) = async {
            let manifest = match self.store.fetch_manifest(dataset).await {
                Ok(v) => v,
                Err(e) => {
                    self.record_store_download_failure(self.store.backend_tag(), &e.to_string())
                        .await;
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
                    self.record_store_download_failure(self.store.backend_tag(), &e.to_string())
                        .await;
                    return Err(e);
                }
            };
            let release_gene_index = self
                .store
                .fetch_release_gene_index_bytes(dataset)
                .await
                .ok();
            Ok::<_, CacheError>((manifest, sqlite, release_gene_index))
        }
        .instrument(tracing::info_span!(
            "store_fetch",
            dataset = %dataset.canonical_string(),
            backend = self.store.backend_tag()
        ))
        .await?;
        let sqlite_hash = async { sha256_hex(&sqlite) }
            .instrument(tracing::info_span!(
                "verify",
                dataset = %dataset.canonical_string()
            ))
            .await;
        if sqlite_hash != manifest.checksums.sqlite_sha256 {
            error!("dataset verify failed {:?}", dataset);
            self.metrics
                .store_download_failures
                .fetch_add(1, Ordering::Relaxed);
            self.record_store_download_failure(
                self.store.backend_tag(),
                "checksum verification failed",
            )
                .await;
            return Err(CacheError(
                "sqlite checksum verification failed".to_string(),
            ));
        }

        let cache_key = manifest_cache_key(&manifest);
        safe_cache_key(&cache_key)?;
        let paths = local_cache_paths(Path::new(&self.cfg.disk_root), &cache_key);
        std::fs::create_dir_all(&paths.derived_dir).map_err(|e| CacheError(e.to_string()))?;
        let lease_path = paths.cache_root.join(".lease.lock");
        let _lease = acquire_artifact_lease(&lease_path, Duration::from_secs(10))?;

        let tmp_dir = self.cfg.disk_root.join(".tmp-atlas-download");
        ensure_secure_dir(&tmp_dir)?;

        let tmp_sqlite = tmp_dir.join(format!("gene_summary.sqlite.{}.tmp", std::process::id()));
        write_atomic_file(&tmp_sqlite, &sqlite)?;
        std::fs::rename(&tmp_sqlite, &paths.sqlite).map_err(|e| CacheError(e.to_string()))?;
        if let Some(parent) = paths.sqlite.parent() {
            if let Ok(dir) = std::fs::File::open(parent) {
                let _ = dir.sync_all();
            }
        }
        if let Some(index_bytes) = release_gene_index {
            write_atomic_file(&paths.release_gene_index, &index_bytes)?;
        }

        let manifest_bytes =
            serde_json::to_vec(&manifest).map_err(|e| CacheError(e.to_string()))?;
        write_atomic_file(&paths.manifest, &manifest_bytes)?;

        let marker = format!(
            "{}:{}",
            manifest.checksums.sqlite_sha256, manifest.db_schema_version
        );
        write_atomic_file(&paths.derived_dir.join(".verified"), marker.as_bytes())?;
        write_atomic_file(
            &dataset_index_path(Path::new(&self.cfg.disk_root), dataset),
            cache_key.as_bytes(),
        )?;

        let size_bytes = std::fs::metadata(&paths.sqlite)
            .map_err(|e| CacheError(e.to_string()))?
            .len();
        let (shard_sqlite_paths, shard_by_seqid) =
            dataset_shards::load_shard_catalog(&paths.derived_dir)?;
        let download_latency_ns = started.elapsed().as_nanos() as u64;

        {
            let mut entries = self.entries.lock().await;
            let sqlite_path = paths.sqlite.clone();
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: sqlite_path.clone(),
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes,
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
        info!(
            dataset = ?dataset,
            cache_key = %cache_key,
            bytes = size_bytes,
            "dataset download complete"
        );
        let _ = std::fs::remove_file(lease_path);
        Ok(())
    }

    async fn is_cached_and_verified(&self, dataset: &DatasetId) -> Result<bool, CacheError> {
        let paths = self.resolve_cache_paths(dataset).await?;
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
            let sqlite_path = paths.sqlite.clone();
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: sqlite_path.clone(),
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes: std::fs::metadata(&sqlite_path)
                        .map(|m| m.len())
                        .unwrap_or(0),
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
        let high_bytes = self
            .cfg
            .max_disk_bytes
            .saturating_mul(self.cfg.disk_high_watermark_pct as u64)
            / 100;
        let low_bytes = self
            .cfg
            .max_disk_bytes
            .saturating_mul(self.cfg.disk_low_watermark_pct as u64)
            / 100;
        if entries.len() > self.cfg.max_dataset_count || total_size > high_bytes {
            let mut ranked: Vec<(DatasetId, std::time::Duration)> = entries
                .iter()
                .filter(|(id, _)| !self.cfg.pinned_datasets.contains(*id))
                .map(|(id, e)| (id.clone(), now.duration_since(e.last_access)))
                .collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1));
            for (id, _) in ranked {
                if entries.len() <= self.cfg.max_dataset_count && total_size <= low_bytes {
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
                let _ =
                    std::fs::remove_file(dataset_index_path(Path::new(&self.cfg.disk_root), &id));
                self.metrics
                    .cache_evictions_total
                    .fetch_add(1, Ordering::Relaxed);
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

    pub async fn dataset_derived_dir_for(
        &self,
        dataset: &DatasetId,
    ) -> Result<PathBuf, CacheError> {
        let paths = self.resolve_cache_paths(dataset).await?;
        Ok(paths.derived_dir)
    }

    async fn resolve_cache_paths(
        &self,
        dataset: &DatasetId,
    ) -> Result<LocalCachePaths, CacheError> {
        {
            let entries = self.entries.lock().await;
            if let Some(entry) = entries.get(dataset) {
                let derived_dir = entry
                    .sqlite_path
                    .parent()
                    .ok_or_else(|| CacheError("invalid cached sqlite path".to_string()))?
                    .to_path_buf();
                let cache_root = derived_dir
                    .parent()
                    .ok_or_else(|| CacheError("invalid cached derived path".to_string()))?
                    .to_path_buf();
                let inputs_dir = cache_root.join("inputs");
                return Ok(LocalCachePaths {
                    cache_root,
                    inputs_dir: inputs_dir.clone(),
                    derived_dir: derived_dir.clone(),
                    fasta: inputs_dir.join("genome.fa.bgz"),
                    fai: inputs_dir.join("genome.fa.bgz.fai"),
                    sqlite: derived_dir.join("gene_summary.sqlite"),
                    manifest: derived_dir.join("manifest.json"),
                    release_gene_index: derived_dir.join("release_gene_index.json"),
                });
            }
        }

        let idx = dataset_index_path(Path::new(&self.cfg.disk_root), dataset);
        if let Ok(key) = std::fs::read_to_string(&idx) {
            let cache_key = key.trim();
            if !cache_key.is_empty() {
                return Ok(local_cache_paths(Path::new(&self.cfg.disk_root), cache_key));
            }
        }

        let legacy = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        Ok(LocalCachePaths {
            cache_root: legacy.dataset_root,
            inputs_dir: legacy.inputs_dir,
            derived_dir: legacy.derived_dir,
            fasta: legacy.fasta,
            fai: legacy.fai,
            sqlite: legacy.sqlite,
            manifest: legacy.manifest,
            release_gene_index: legacy.release_gene_index,
        })
    }
}
