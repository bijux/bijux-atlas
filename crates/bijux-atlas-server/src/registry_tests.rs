use crate::{
    CacheError, CatalogFetch, DatasetCacheConfig, DatasetCacheManager, DatasetStoreBackend,
    FederatedBackend, RegistrySource,
};
use async_trait::async_trait;
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ManifestStats,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Clone)]
struct MiniStore {
    catalog: Catalog,
    manifest: Option<ArtifactManifest>,
    sqlite: Option<Vec<u8>>,
}

#[derive(Clone)]
struct FlakyCatalogStore {
    catalog: Catalog,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl DatasetStoreBackend for FlakyCatalogStore {
    async fn fetch_catalog(
        &self,
        _if_none_match: Option<&str>,
    ) -> Result<CatalogFetch, CacheError> {
        let call = self.calls.fetch_add(1, Ordering::Relaxed);
        if call == 0 {
            return Ok(CatalogFetch::Updated {
                etag: sha256_hex(
                    &serde_json::to_vec(&self.catalog).map_err(|e| CacheError(e.to_string()))?,
                ),
                catalog: self.catalog.clone(),
            });
        }
        Err(CacheError(
            "catalog corruption simulation: invalid catalog payload".to_string(),
        ))
    }

    async fn fetch_manifest(&self, _dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        Err(CacheError("manifest not needed".to_string()))
    }

    async fn fetch_sqlite_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        Err(CacheError("sqlite not needed".to_string()))
    }

    async fn fetch_fasta_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        Err(CacheError("fasta not needed".to_string()))
    }

    async fn fetch_fai_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        Err(CacheError("fai not needed".to_string()))
    }

    async fn fetch_release_gene_index_bytes(
        &self,
        _dataset: &DatasetId,
    ) -> Result<Vec<u8>, CacheError> {
        Err(CacheError("index not needed".to_string()))
    }
}

impl MiniStore {
    fn new(catalog: Catalog, manifest: Option<ArtifactManifest>, sqlite: Option<Vec<u8>>) -> Self {
        Self {
            catalog,
            manifest,
            sqlite,
        }
    }
}

#[async_trait]
impl DatasetStoreBackend for MiniStore {
    async fn fetch_catalog(
        &self,
        _if_none_match: Option<&str>,
    ) -> Result<CatalogFetch, CacheError> {
        Ok(CatalogFetch::Updated {
            etag: sha256_hex(
                &serde_json::to_vec(&self.catalog).map_err(|e| CacheError(e.to_string()))?,
            ),
            catalog: self.catalog.clone(),
        })
    }

    async fn fetch_manifest(&self, _dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        self.manifest
            .clone()
            .ok_or_else(|| CacheError("manifest missing".to_string()))
    }

    async fn fetch_sqlite_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        self.sqlite
            .clone()
            .ok_or_else(|| CacheError("sqlite missing".to_string()))
    }

    async fn fetch_fasta_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        Err(CacheError("no fasta".to_string()))
    }

    async fn fetch_fai_bytes(&self, _dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        Err(CacheError("no fai".to_string()))
    }

    async fn fetch_release_gene_index_bytes(
        &self,
        _dataset: &DatasetId,
    ) -> Result<Vec<u8>, CacheError> {
        Err(CacheError("no release gene index".to_string()))
    }
}

fn ds(release: &str, species: &str, assembly: &str) -> DatasetId {
    DatasetId::new(release, species, assembly).expect("dataset id")
}

fn make_manifest(sqlite_sha: &str) -> ArtifactManifest {
    ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds("110", "homo_sapiens", "GRCh38"),
        ArtifactChecksums::new(
            "gff".to_string(),
            "fasta".to_string(),
            "fai".to_string(),
            sqlite_sha.to_string(),
        ),
        ManifestStats::new(1, 1, 1),
    )
}

#[tokio::test]
async fn federated_catalog_merge_is_deterministic_and_tracks_shadowing() {
    let a = ds("110", "homo_sapiens", "GRCh38");
    let b = ds("111", "homo_sapiens", "GRCh38");
    let c1 = Catalog::new(vec![CatalogEntry::new(
        a.clone(),
        "p1/manifest.json".to_string(),
        "p1/gene_summary.sqlite".to_string(),
    )]);
    let c2 = Catalog::new(vec![
        CatalogEntry::new(
            a.clone(),
            "p2/manifest.json".to_string(),
            "p2/gene_summary.sqlite".to_string(),
        ),
        CatalogEntry::new(
            b.clone(),
            "p2b/manifest.json".to_string(),
            "p2b/gene_summary.sqlite".to_string(),
        ),
    ]);
    let s1 = Arc::new(MiniStore::new(c1, None, None));
    let s2 = Arc::new(MiniStore::new(c2, None, None));
    let backend = FederatedBackend::new(vec![
        RegistrySource::new("primary", s1, Duration::from_secs(5), None),
        RegistrySource::new("mirror", s2, Duration::from_secs(5), None),
    ]);

    let merged = match backend.fetch_catalog(None).await.expect("fetch catalog") {
        CatalogFetch::Updated { catalog, .. } => catalog,
        CatalogFetch::NotModified => panic!("expected updated"),
    };
    let ids = merged
        .datasets
        .iter()
        .map(|x| x.dataset.canonical_string())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![a.canonical_string(), b.canonical_string()]);

    let health = backend.registry_health().await;
    assert_eq!(health.len(), 2);
    assert_eq!(health[1].shadowed_datasets, 1);
}

#[tokio::test]
async fn federated_fetch_uses_priority_then_fallback() {
    let dataset = ds("110", "homo_sapiens", "GRCh38");
    let sqlite = b"sqlite-body".to_vec();
    let manifest = make_manifest(&sha256_hex(&sqlite));
    let empty = Catalog::new(vec![CatalogEntry::new(
        dataset.clone(),
        "a/manifest.json".to_string(),
        "a/gene_summary.sqlite".to_string(),
    )]);

    let primary = Arc::new(MiniStore::new(empty.clone(), None, None));
    let mirror = Arc::new(MiniStore::new(
        empty,
        Some(manifest.clone()),
        Some(sqlite.clone()),
    ));
    let backend = FederatedBackend::new(vec![
        RegistrySource::new("primary", primary, Duration::from_secs(5), None),
        RegistrySource::new("mirror", mirror, Duration::from_secs(5), None),
    ]);

    let got_manifest = backend
        .fetch_manifest(&dataset)
        .await
        .expect("fallback manifest");
    assert_eq!(
        got_manifest.checksums.sqlite_sha256,
        manifest.checksums.sqlite_sha256
    );
    let got_sqlite = backend
        .fetch_sqlite_bytes(&dataset)
        .await
        .expect("fallback sqlite");
    assert_eq!(got_sqlite, sqlite);
}

#[tokio::test]
async fn federated_signature_validation_drops_mismatched_registry() {
    let dataset = ds("110", "homo_sapiens", "GRCh38");
    let trusted_catalog = Catalog::new(vec![CatalogEntry::new(
        dataset,
        "trusted/manifest.json".to_string(),
        "trusted/gene_summary.sqlite".to_string(),
    )]);
    let trusted_sig = sha256_hex(
        &serde_json::to_vec(&trusted_catalog).expect("serialize trusted catalog for signature"),
    );

    let bad_catalog = Catalog::new(vec![CatalogEntry::new(
        ds("999", "mus_musculus", "GRCm39"),
        "bad/manifest.json".to_string(),
        "bad/gene_summary.sqlite".to_string(),
    )]);

    let backend = FederatedBackend::new(vec![
        RegistrySource::new(
            "bad",
            Arc::new(MiniStore::new(bad_catalog, None, None)),
            Duration::from_secs(5),
            Some("deadbeef".to_string()),
        ),
        RegistrySource::new(
            "trusted",
            Arc::new(MiniStore::new(trusted_catalog.clone(), None, None)),
            Duration::from_secs(5),
            Some(trusted_sig),
        ),
    ]);

    let merged = match backend.fetch_catalog(None).await.expect("fetch catalog") {
        CatalogFetch::Updated { catalog, .. } => catalog,
        CatalogFetch::NotModified => panic!("expected updated"),
    };
    assert_eq!(merged.datasets, trusted_catalog.datasets);

    let health = backend.registry_health().await;
    let bad = health.iter().find(|h| h.name == "bad").expect("bad source");
    assert!(!bad.healthy);
    assert!(bad
        .last_error
        .as_deref()
        .is_some_and(|msg| msg.contains("signature mismatch")));
}

#[tokio::test]
async fn cache_manager_registry_freeze_mode_skips_refresh() {
    let dataset = ds("110", "homo_sapiens", "GRCh38");
    let catalog = Catalog::new(vec![CatalogEntry::new(
        dataset,
        "trusted/manifest.json".to_string(),
        "trusted/gene_summary.sqlite".to_string(),
    )]);
    let store = Arc::new(MiniStore::new(catalog, None, None));
    let cfg = DatasetCacheConfig {
        registry_freeze_mode: true,
        ..DatasetCacheConfig::default()
    };
    let cache = DatasetCacheManager::new(cfg, store);
    cache
        .refresh_catalog()
        .await
        .expect("frozen refresh is no-op");
    assert!(cache.current_catalog().await.is_none());
}

#[tokio::test]
async fn catalog_corruption_keeps_last_good_catalog_as_fallback() {
    let dataset = ds("110", "homo_sapiens", "GRCh38");
    let catalog = Catalog::new(vec![CatalogEntry::new(
        dataset.clone(),
        "trusted/manifest.json".to_string(),
        "trusted/gene_summary.sqlite".to_string(),
    )]);
    let store = Arc::new(FlakyCatalogStore {
        catalog: catalog.clone(),
        calls: Arc::new(AtomicUsize::new(0)),
    });
    let cfg = DatasetCacheConfig {
        registry_ttl: Duration::from_millis(0),
        ..DatasetCacheConfig::default()
    };
    let cache = DatasetCacheManager::new(cfg, store);

    cache.refresh_catalog().await.expect("initial catalog refresh");
    let first = cache.current_catalog().await.expect("catalog after first refresh");
    assert_eq!(first.datasets, catalog.datasets);

    let err = cache
        .refresh_catalog()
        .await
        .expect_err("second refresh should simulate corruption");
    assert!(err.to_string().contains("catalog corruption simulation"));

    let fallback = cache
        .current_catalog()
        .await
        .expect("fallback catalog should remain available");
    assert_eq!(
        fallback.datasets, catalog.datasets,
        "cache manager must keep last-good catalog on refresh failure"
    );
}
