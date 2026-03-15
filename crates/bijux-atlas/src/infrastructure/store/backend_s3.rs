// SPDX-License-Identifier: Apache-2.0

pub struct S3LikeStore {
    pub endpoint: String,
    pub presigned_endpoint: Option<String>,
    pub bucket: String,
    pub bearer_token: Option<String>,
    pub retry: RetryPolicy,
    pub cached_only_mode: bool,
    pub cache_root: Option<PathBuf>,
    client: Client,
    instrumentation: Arc<dyn StoreInstrumentation>,
}

impl S3LikeStore {
    #[must_use]
    pub fn new(endpoint: String, bucket: String) -> Self {
        Self {
            endpoint,
            presigned_endpoint: None,
            bucket,
            bearer_token: None,
            retry: RetryPolicy::default(),
            cached_only_mode: false,
            cache_root: None,
            client: Client::new(),
            instrumentation: Arc::new(NoopInstrumentation),
        }
    }

    #[must_use]
    pub fn with_bearer_token(mut self, token: Option<String>) -> Self {
        self.bearer_token = token;
        self
    }

    #[must_use]
    pub fn with_presigned_endpoint(mut self, endpoint: Option<String>) -> Self {
        self.presigned_endpoint = endpoint
            .map(|x| x.trim_end_matches('/').to_string())
            .filter(|x| !x.is_empty());
        self
    }

    #[must_use]
    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    #[must_use]
    pub fn with_cache(mut self, cache_root: PathBuf, cached_only_mode: bool) -> Self {
        self.cache_root = Some(cache_root);
        self.cached_only_mode = cached_only_mode;
        self
    }

    #[must_use]
    pub fn with_instrumentation(mut self, instrumentation: Arc<dyn StoreInstrumentation>) -> Self {
        self.instrumentation = instrumentation;
        self
    }

    fn object_url(&self, key: &str) -> String {
        let base = self.presigned_endpoint.as_deref().unwrap_or(&self.endpoint);
        format!(
            "{}/{}/{}",
            base.trim_end_matches('/'),
            self.bucket,
            key.trim_start_matches('/')
        )
    }

    fn get_with_retry(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        if let Some(cache_root) = &self.cache_root {
            let cached = cache_root.join(key.replace('/', "__"));
            if cached.exists() {
                return fs::read(&cached)
                    .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()));
            }
            if self.cached_only_mode {
                return Err(StoreError::new(
                    StoreErrorCode::CachedOnly,
                    "cached-only mode enabled and object missing from cache",
                ));
            }
        } else if self.cached_only_mode {
            return Err(StoreError::new(
                StoreErrorCode::CachedOnly,
                "cached-only mode enabled without cache root",
            ));
        }

        let mut attempt = 0usize;
        let mut buf: Vec<u8> = Vec::new();
        loop {
            let started = Instant::now();
            let mut req = self.client.get(self.object_url(key));
            if !buf.is_empty() {
                req = req.header(reqwest::header::RANGE, format!("bytes={}-", buf.len()));
            }
            if let Some(token) = &self.bearer_token {
                req = req.bearer_auth(token);
            }
            match req.send() {
                Ok(resp) => {
                    if resp.status().is_success() || resp.status().as_u16() == 206 {
                        let total = resp
                            .headers()
                            .get("content-range")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.split('/').nth(1))
                            .and_then(|v| v.parse::<usize>().ok());
                        let mut part = resp
                            .bytes()
                            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?
                            .to_vec();
                        if part.is_empty() {
                            return Ok(buf);
                        }
                        buf.append(&mut part);
                        if let Some(total) = total {
                            if buf.len() < total {
                                attempt += 1;
                                if attempt >= self.retry.max_attempts {
                                    return Err(StoreError::new(
                                        StoreErrorCode::Network,
                                        "partial content did not complete within retry budget",
                                    ));
                                }
                                thread::sleep(self.retry.delay_for_attempt(attempt));
                                continue;
                            }
                        }
                        let bytes = buf.clone();
                        if let Some(root) = &self.cache_root {
                            fs::create_dir_all(root)
                                .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
                            let target = root.join(key.replace('/', "__"));
                            fs::write(target, &bytes)
                                .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
                        }
                        self.instrumentation.observe_download(
                            "s3like",
                            bytes.len(),
                            started.elapsed(),
                        );
                        return Ok(bytes);
                    }
                    if resp.status().as_u16() == 404 {
                        return Err(StoreError::new(
                            StoreErrorCode::NotFound,
                            "object not found",
                        ));
                    }
                }
                Err(err) => {
                    self.instrumentation
                        .observe_error("s3like", StoreErrorCode::Network);
                    if attempt + 1 >= self.retry.max_attempts {
                        return Err(StoreError::new(StoreErrorCode::Network, err.to_string()));
                    }
                }
            }
            attempt += 1;
            thread::sleep(self.retry.delay_for_attempt(attempt));
        }
    }

    fn put_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        let started = Instant::now();
        let mut req = self.client.put(self.object_url(key)).body(bytes.to_vec());
        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?;
        if !resp.status().is_success() {
            return Err(StoreError::new(
                StoreErrorCode::Network,
                format!("s3-like put failed: {}", resp.status()),
            ));
        }
        self.instrumentation
            .observe_upload("s3like", bytes.len(), started.elapsed());
        Ok(())
    }
}

impl ArtifactStore for S3LikeStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError> {
        let bytes = self.get_with_retry(CATALOG_FILE)?;
        let catalog: Catalog = serde_json::from_slice(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        validate_catalog_strict(&catalog)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        Ok(catalog.datasets.into_iter().map(|x| x.dataset).collect())
    }

    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        let key = dataset_manifest_key(dataset);
        let lock_key = dataset_manifest_lock_key(dataset);
        let bytes = self.get_with_retry(&key)?;
        let lock_bytes = self.get_with_retry(&lock_key)?;
        let lock: ManifestLock = serde_json::from_slice(&lock_bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        lock.validate_manifest_only(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        manifest
            .validate_strict()
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        Ok(manifest)
    }

    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        self.get_with_retry(&dataset_sqlite_key(dataset))
    }

    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        if self.exists(dataset)? {
            return Err(StoreError::new(
                StoreErrorCode::Conflict,
                "dataset already exists and cannot be overwritten",
            ));
        }

        verify_expected_sha256(manifest_bytes, expected_manifest_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        verify_expected_sha256(sqlite_bytes, expected_sqlite_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;

        let prefix = dataset_key_prefix(dataset);
        self.put_bytes(&format!("{prefix}/manifest.json.tmp"), manifest_bytes)?;
        self.put_bytes(&format!("{prefix}/gene_summary.sqlite.tmp"), sqlite_bytes)?;

        let lock = ManifestLock::from_bytes(manifest_bytes, sqlite_bytes);
        let lock_json = serde_json::to_vec(&lock)
            .map_err(|e| StoreError::new(StoreErrorCode::Internal, e.to_string()))?;
        self.put_bytes(&format!("{prefix}/manifest.lock"), &lock_json)?;

        // S3 has no native rename; publish by writing final keys after temp upload verification.
        self.put_bytes(&format!("{prefix}/manifest.json"), manifest_bytes)?;
        self.put_bytes(&format!("{prefix}/gene_summary.sqlite"), sqlite_bytes)?;
        Ok(())
    }

    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError> {
        match self.get_manifest(dataset) {
            Ok(_) => Ok(true),
            Err(err) if err.code == StoreErrorCode::NotFound => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn acquire_publish_lock(&self, _dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        Err(StoreError::new(
            StoreErrorCode::Unsupported,
            "s3-like backend does not support local publish lock guard",
        ))
    }
}

fn handle_etag_response(
    response: Response,
    key: &str,
    etags: &Arc<Mutex<HashMap<String, String>>>,
    cache_root: &Option<PathBuf>,
) -> Result<Vec<u8>, StoreError> {
    if response.status().as_u16() == 304 {
        if let Some(root) = cache_root {
            let target = root.join(key.replace('/', "__"));
            return fs::read(target)
                .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()));
        }
        return Err(StoreError::new(
            StoreErrorCode::Internal,
            "received 304 without cache root",
        ));
    }

    if response.status().as_u16() == 404 {
        return Err(StoreError::new(
            StoreErrorCode::NotFound,
            "resource not found",
        ));
    }

    if !response.status().is_success() {
        return Err(StoreError::new(
            StoreErrorCode::Network,
            format!("http fetch failed: {}", response.status()),
        ));
    }

    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|h| h.to_str().ok())
        .map(ToString::to_string);

    let bytes = response
        .bytes()
        .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?
        .to_vec();

    if let Some(root) = cache_root {
        fs::create_dir_all(root).map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        let target = root.join(key.replace('/', "__"));
        fs::write(target, &bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    }

    if let Some(tag) = etag {
        if let Ok(mut map) = etags.lock() {
            map.insert(key.to_string(), tag);
        }
    }

    Ok(bytes)
}
