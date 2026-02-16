use crate::{CacheError, DatasetCacheManager};
use bijux_atlas_model::{DatasetId, ShardCatalog};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::OwnedSemaphorePermit;

pub(crate) fn load_shard_catalog(
    derived_dir: &Path,
) -> Result<(Vec<PathBuf>, HashMap<String, Vec<PathBuf>>), CacheError> {
    let path = derived_dir.join("catalog_shards.json");
    if !path.exists() {
        return Ok((Vec::new(), HashMap::new()));
    }
    let raw = std::fs::read(path).map_err(|e| CacheError(e.to_string()))?;
    let catalog: ShardCatalog = serde_json::from_slice(&raw).map_err(|e| CacheError(e.to_string()))?;
    let mut all = Vec::new();
    let mut by_seqid: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for shard in catalog.shards {
        let shard_path = derived_dir.join(shard.sqlite_path);
        all.push(shard_path.clone());
        for seqid in shard.seqids {
            by_seqid.entry(seqid).or_default().push(shard_path.clone());
        }
    }
    all.sort();
    all.dedup();
    Ok((all, by_seqid))
}

impl DatasetCacheManager {
    pub async fn selected_shards_for_region(
        &self,
        dataset: &DatasetId,
        seqid: Option<&str>,
    ) -> Result<Vec<PathBuf>, CacheError> {
        self.ensure_dataset_cached(dataset).await?;
        let entries = self.entries.lock().await;
        let Some(entry) = entries.get(dataset) else {
            return Ok(Vec::new());
        };
        if let Some(seq) = seqid {
            return Ok(entry.shard_by_seqid.get(seq).cloned().unwrap_or_default());
        }
        Ok(entry.shard_sqlite_paths.clone())
    }

    pub fn disk_root(&self) -> &Path {
        &self.cfg.disk_root
    }

    pub fn max_open_shards_per_pod(&self) -> usize {
        self.cfg.max_open_shards_per_pod
    }

    pub fn sqlite_pragmas_for_shard_open(&self) -> (i64, i64) {
        (
            self.cfg.sqlite_pragma_cache_kib,
            (self.cfg.sqlite_pragma_mmap_bytes / 4).max(64 * 1024 * 1024),
        )
    }

    pub async fn acquire_shard_permit(&self) -> Result<OwnedSemaphorePermit, CacheError> {
        self.shard_open_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))
    }

    pub fn try_acquire_shard_permit(&self) -> Result<OwnedSemaphorePermit, CacheError> {
        self.shard_open_semaphore
            .clone()
            .try_acquire_owned()
            .map_err(|e| CacheError(e.to_string()))
    }
}
