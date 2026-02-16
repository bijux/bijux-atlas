use crate::{CacheError, CatalogFetch, DatasetStoreBackend};
use async_trait::async_trait;
use bijux_atlas_model::{ArtifactManifest, Catalog, DatasetId};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct FakeStore {
    pub catalog: Mutex<Catalog>,
    pub manifest: Mutex<HashMap<DatasetId, ArtifactManifest>>,
    pub sqlite: Mutex<HashMap<DatasetId, Vec<u8>>>,
    pub fetch_calls: std::sync::atomic::AtomicU64,
    pub etag: Mutex<String>,
    pub slow_read: bool,
    pub slow_read_delay: Duration,
}

impl Default for FakeStore {
    fn default() -> Self {
        Self {
            catalog: Mutex::new(Catalog::new(Vec::new())),
            manifest: Mutex::new(HashMap::new()),
            sqlite: Mutex::new(HashMap::new()),
            fetch_calls: std::sync::atomic::AtomicU64::new(0),
            etag: Mutex::new(String::new()),
            slow_read: false,
            slow_read_delay: Duration::from_millis(0),
        }
    }
}

#[async_trait]
impl DatasetStoreBackend for FakeStore {
    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError> {
        let etag = self.etag.lock().await.clone();
        if if_none_match == Some(etag.as_str()) {
            return Ok(CatalogFetch::NotModified);
        }
        Ok(CatalogFetch::Updated {
            etag,
            catalog: self.catalog.lock().await.clone(),
        })
    }

    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        self.fetch_calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.slow_read {
            let delay = if self.slow_read_delay.is_zero() {
                Duration::from_millis(200)
            } else {
                self.slow_read_delay
            };
            tokio::time::sleep(delay).await;
        }
        self.manifest
            .lock()
            .await
            .get(dataset)
            .cloned()
            .ok_or_else(|| CacheError("manifest missing".to_string()))
    }

    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        if self.slow_read {
            let delay = if self.slow_read_delay.is_zero() {
                Duration::from_millis(200)
            } else {
                self.slow_read_delay
            };
            tokio::time::sleep(delay).await;
        }
        self.sqlite
            .lock()
            .await
            .get(dataset)
            .cloned()
            .ok_or_else(|| CacheError("sqlite missing".to_string()))
    }
}
