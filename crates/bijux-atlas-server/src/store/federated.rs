// SPDX-License-Identifier: Apache-2.0

use crate::{CacheError, CatalogFetch, DatasetStoreBackend, RegistrySourceHealth};
use async_trait::async_trait;
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactManifest, Catalog, CatalogEntry, DatasetId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RegistrySource {
    pub name: String,
    pub backend: Arc<dyn DatasetStoreBackend>,
    pub ttl: Duration,
    pub expected_catalog_signature: Option<String>,
}

impl RegistrySource {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        backend: Arc<dyn DatasetStoreBackend>,
        ttl: Duration,
        expected_catalog_signature: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            backend,
            ttl,
            expected_catalog_signature,
        }
    }
}

#[derive(Clone)]
struct SourceState {
    etag: Option<String>,
    catalog: Option<Catalog>,
    last_refresh: Option<Instant>,
    last_error: Option<String>,
    shadowed_datasets: u64,
}

impl SourceState {
    fn new() -> Self {
        Self {
            etag: None,
            catalog: None,
            last_refresh: None,
            last_error: None,
            shadowed_datasets: 0,
        }
    }
}

#[derive(Default)]
struct FederatedState {
    by_source: Vec<SourceState>,
    dataset_primary_source: HashMap<DatasetId, usize>,
}

pub struct FederatedBackend {
    sources: Vec<RegistrySource>,
    state: Mutex<FederatedState>,
}

impl FederatedBackend {
    #[must_use]
    pub fn new(sources: Vec<RegistrySource>) -> Self {
        let by_source = sources.iter().map(|_| SourceState::new()).collect();
        Self {
            sources,
            state: Mutex::new(FederatedState {
                by_source,
                ..FederatedState::default()
            }),
        }
    }

    fn sorted_entries(catalog: &Catalog) -> Vec<CatalogEntry> {
        let mut out = catalog.datasets.clone();
        out.sort_by(|a, b| {
            a.dataset
                .canonical_string()
                .cmp(&b.dataset.canonical_string())
                .then_with(|| a.manifest_path.cmp(&b.manifest_path))
                .then_with(|| a.sqlite_path.cmp(&b.sqlite_path))
        });
        out
    }

    fn merge_catalogs(
        &self,
        catalogs: &[(usize, Catalog)],
    ) -> (Catalog, HashMap<DatasetId, usize>, HashMap<usize, u64>) {
        let mut picked: HashMap<DatasetId, CatalogEntry> = HashMap::new();
        let mut owner: HashMap<DatasetId, usize> = HashMap::new();
        let mut shadowed: HashMap<usize, u64> = HashMap::new();
        for (source_idx, catalog) in catalogs {
            for entry in Self::sorted_entries(catalog) {
                if picked.contains_key(&entry.dataset) {
                    *shadowed.entry(*source_idx).or_insert(0) += 1;
                    continue;
                }
                owner.insert(entry.dataset.clone(), *source_idx);
                picked.insert(entry.dataset.clone(), entry);
            }
        }
        let mut merged = picked.into_values().collect::<Vec<_>>();
        merged.sort_by(|a, b| {
            a.dataset
                .canonical_string()
                .cmp(&b.dataset.canonical_string())
                .then_with(|| a.manifest_path.cmp(&b.manifest_path))
                .then_with(|| a.sqlite_path.cmp(&b.sqlite_path))
        });
        (Catalog::new(merged), owner, shadowed)
    }

    async fn get_primary_source_order(&self, dataset: &DatasetId) -> Vec<usize> {
        let state = self.state.lock().await;
        if let Some(primary) = state.dataset_primary_source.get(dataset) {
            let mut order = vec![*primary];
            order.extend((0..self.sources.len()).filter(|idx| *idx != *primary));
            order
        } else {
            (0..self.sources.len()).collect()
        }
    }

    async fn source_health(&self) -> Vec<RegistrySourceHealth> {
        let state = self.state.lock().await;
        self.sources
            .iter()
            .enumerate()
            .map(|(idx, src)| {
                let source_state = state
                    .by_source
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(SourceState::new);
                let healthy = source_state.last_error.is_none();
                RegistrySourceHealth {
                    name: src.name.clone(),
                    priority: idx as u32,
                    healthy,
                    last_error: source_state.last_error,
                    shadowed_datasets: source_state.shadowed_datasets,
                    ttl_seconds: src.ttl.as_secs(),
                }
            })
            .collect()
    }

    async fn fetch_from_sources(&self) -> Result<Catalog, CacheError> {
        let mut out = Vec::<(usize, Catalog)>::new();
        let now = Instant::now();

        for (idx, source) in self.sources.iter().enumerate() {
            let (maybe_cached, etag) = {
                let state = self.state.lock().await;
                let source_state = &state.by_source[idx];
                let cached_ok = source_state
                    .last_refresh
                    .is_some_and(|ts| now.duration_since(ts) <= source.ttl);
                let catalog = if cached_ok {
                    source_state.catalog.clone()
                } else {
                    None
                };
                (catalog, source_state.etag.clone())
            };

            if let Some(cached) = maybe_cached {
                out.push((idx, cached));
                continue;
            }

            let fetch = source.backend.fetch_catalog(etag.as_deref()).await;
            match fetch {
                Ok(CatalogFetch::NotModified) => {
                    let mut state = self.state.lock().await;
                    if let Some(source_state) = state.by_source.get_mut(idx) {
                        source_state.last_error = None;
                        source_state.last_refresh = Some(now);
                        if let Some(cached) = source_state.catalog.clone() {
                            out.push((idx, cached));
                        }
                    }
                }
                Ok(CatalogFetch::Updated { etag, catalog }) => {
                    if let Some(expected) = &source.expected_catalog_signature {
                        let digest =
                            sha256_hex(&serde_json::to_vec(&catalog).map_err(|e| {
                                CacheError(format!("catalog serialize failed: {e}"))
                            })?);
                        if digest != *expected {
                            let mut state = self.state.lock().await;
                            if let Some(source_state) = state.by_source.get_mut(idx) {
                                source_state.last_error = Some(format!(
                                    "catalog signature mismatch: expected {expected}, got {digest}"
                                ));
                                source_state.last_refresh = Some(now);
                            }
                            continue;
                        }
                    }
                    {
                        let mut state = self.state.lock().await;
                        if let Some(source_state) = state.by_source.get_mut(idx) {
                            source_state.etag = Some(etag);
                            source_state.catalog = Some(catalog.clone());
                            source_state.last_error = None;
                            source_state.last_refresh = Some(now);
                        }
                    }
                    out.push((idx, catalog));
                }
                Err(e) => {
                    let mut state = self.state.lock().await;
                    if let Some(source_state) = state.by_source.get_mut(idx) {
                        source_state.last_error = Some(e.to_string());
                        source_state.last_refresh = Some(now);
                    }
                }
            }
        }

        if out.is_empty() {
            return Err(CacheError(
                "all registries failed to return catalog".to_string(),
            ));
        }

        let (merged, owner, shadowed) = self.merge_catalogs(&out);
        {
            let mut state = self.state.lock().await;
            state.dataset_primary_source = owner;
            for (idx, src_state) in state.by_source.iter_mut().enumerate() {
                src_state.shadowed_datasets = shadowed.get(&idx).copied().unwrap_or(0);
            }
        }
        Ok(merged)
    }

    fn combined_fetch_errors(errors: Vec<String>) -> CacheError {
        CacheError(format!(
            "dataset fetch failed across registries: {}",
            errors.join(" | ")
        ))
    }
}

#[async_trait]
impl DatasetStoreBackend for FederatedBackend {
    fn backend_tag(&self) -> &'static str {
        "federated"
    }

    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError> {
        let merged = self.fetch_from_sources().await?;
        let payload = serde_json::to_vec(&merged)
            .map_err(|e| CacheError(format!("catalog serialize failed: {e}")))?;
        let etag = sha256_hex(&payload);
        if if_none_match == Some(etag.as_str()) {
            Ok(CatalogFetch::NotModified)
        } else {
            Ok(CatalogFetch::Updated {
                etag,
                catalog: merged,
            })
        }
    }

    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        let order = self.get_primary_source_order(dataset).await;
        let mut errors = Vec::new();
        for idx in order {
            let source = &self.sources[idx];
            match source.backend.fetch_manifest(dataset).await {
                Ok(v) => return Ok(v),
                Err(e) => errors.push(format!("{}: {}", source.name, e)),
            }
        }
        Err(Self::combined_fetch_errors(errors))
    }

    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let order = self.get_primary_source_order(dataset).await;
        let mut errors = Vec::new();
        for idx in order {
            let source = &self.sources[idx];
            match source.backend.fetch_sqlite_bytes(dataset).await {
                Ok(v) => return Ok(v),
                Err(e) => errors.push(format!("{}: {}", source.name, e)),
            }
        }
        Err(Self::combined_fetch_errors(errors))
    }

    async fn fetch_fasta_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let order = self.get_primary_source_order(dataset).await;
        let mut errors = Vec::new();
        for idx in order {
            let source = &self.sources[idx];
            match source.backend.fetch_fasta_bytes(dataset).await {
                Ok(v) => return Ok(v),
                Err(e) => errors.push(format!("{}: {}", source.name, e)),
            }
        }
        Err(Self::combined_fetch_errors(errors))
    }

    async fn fetch_fai_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let order = self.get_primary_source_order(dataset).await;
        let mut errors = Vec::new();
        for idx in order {
            let source = &self.sources[idx];
            match source.backend.fetch_fai_bytes(dataset).await {
                Ok(v) => return Ok(v),
                Err(e) => errors.push(format!("{}: {}", source.name, e)),
            }
        }
        Err(Self::combined_fetch_errors(errors))
    }

    async fn fetch_release_gene_index_bytes(
        &self,
        dataset: &DatasetId,
    ) -> Result<Vec<u8>, CacheError> {
        let order = self.get_primary_source_order(dataset).await;
        let mut errors = Vec::new();
        for idx in order {
            let source = &self.sources[idx];
            match source.backend.fetch_release_gene_index_bytes(dataset).await {
                Ok(v) => return Ok(v),
                Err(e) => errors.push(format!("{}: {}", source.name, e)),
            }
        }
        Err(Self::combined_fetch_errors(errors))
    }

    async fn registry_health(&self) -> Vec<RegistrySourceHealth> {
        self.source_health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::BTreeSet;

    #[derive(Clone)]
    struct MockBackend {
        catalog: Catalog,
    }

    #[async_trait]
    impl DatasetStoreBackend for MockBackend {
        async fn fetch_catalog(
            &self,
            _if_none_match: Option<&str>,
        ) -> Result<CatalogFetch, CacheError> {
            Ok(CatalogFetch::Updated {
                etag: "mock".to_string(),
                catalog: self.catalog.clone(),
            })
        }

        async fn fetch_manifest(
            &self,
            _dataset: &DatasetId,
        ) -> Result<ArtifactManifest, CacheError> {
            Err(CacheError("unused in test".to_string()))
        }

        async fn fetch_sqlite_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
            Err(CacheError("unused in test".to_string()))
        }

        async fn fetch_fasta_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
            Err(CacheError("unused in test".to_string()))
        }

        async fn fetch_fai_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
            Err(CacheError("unused in test".to_string()))
        }

        async fn fetch_release_gene_index_bytes(
            &self,
            _dataset: &DatasetId,
        ) -> Result<Vec<u8>, CacheError> {
            Err(CacheError("unused in test".to_string()))
        }
    }

    fn mk_dataset(i: usize) -> DatasetId {
        DatasetId::new(&format!("{i:03}"), "homo_sapiens", "GRCh38").expect("dataset")
    }

    fn mk_catalog(start: usize, len: usize) -> Catalog {
        let mut datasets = Vec::with_capacity(len);
        for i in start..(start + len) {
            let d = mk_dataset(i);
            datasets.push(CatalogEntry::new(
                d,
                format!("release={i:03}/manifest.json"),
                format!("release={i:03}/gene_summary.sqlite"),
            ));
        }
        Catalog::new(datasets)
    }

    #[tokio::test]
    async fn multi_catalog_merge_is_deterministic_at_scale() {
        let c1 = mk_catalog(0, 400);
        let c2 = mk_catalog(200, 400);
        let c3 = mk_catalog(350, 400);
        let backends = vec![
            RegistrySource::new(
                "a",
                Arc::new(MockBackend {
                    catalog: c1.clone(),
                }),
                Duration::from_secs(0),
                None,
            ),
            RegistrySource::new(
                "b",
                Arc::new(MockBackend {
                    catalog: c2.clone(),
                }),
                Duration::from_secs(0),
                None,
            ),
            RegistrySource::new(
                "c",
                Arc::new(MockBackend {
                    catalog: c3.clone(),
                }),
                Duration::from_secs(0),
                None,
            ),
        ];
        let fb = FederatedBackend::new(backends);
        let merged1 = match fb.fetch_catalog(None).await.expect("merge 1") {
            CatalogFetch::Updated { catalog, .. } => catalog,
            CatalogFetch::NotModified => panic!("unexpected not modified"),
        };
        let merged2 = match fb.fetch_catalog(None).await.expect("merge 2") {
            CatalogFetch::Updated { catalog, .. } => catalog,
            CatalogFetch::NotModified => panic!("unexpected not modified"),
        };

        let k1 = merged1
            .datasets
            .iter()
            .map(|x| x.dataset.canonical_string())
            .collect::<Vec<_>>();
        let k2 = merged2
            .datasets
            .iter()
            .map(|x| x.dataset.canonical_string())
            .collect::<Vec<_>>();
        assert_eq!(k1, k2, "merge ordering must be deterministic");
        let unique = k1.iter().cloned().collect::<BTreeSet<_>>();
        assert_eq!(
            k1.len(),
            unique.len(),
            "merged catalog must dedupe datasets"
        );
    }
}
