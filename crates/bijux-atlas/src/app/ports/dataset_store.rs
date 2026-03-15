// SPDX-License-Identifier: Apache-2.0

use crate::app::cache::{CacheError, RegistrySourceHealth};
use crate::domain::dataset::{ArtifactManifest, Catalog, DatasetId};
use async_trait::async_trait;

/// Runtime read port used by the server cache and query-serving path.
///
/// This port is intentionally narrower than the repository-wide artifact publishing
/// interfaces under [`crate::app::ports::store`]: it models only the read operations
/// required by a running Atlas node to discover catalogs and hydrate cached datasets.
#[non_exhaustive]
pub enum CatalogFetch {
    NotModified,
    Updated { etag: String, catalog: Catalog },
}

/// Runtime-facing dataset source abstraction owned by the application layer.
///
/// Implementations may be backed by local files, S3-like object storage, or federated
/// registries, but application services depend only on this read-oriented contract.
#[async_trait]
pub trait DatasetStoreBackend: Send + Sync + 'static {
    fn backend_tag(&self) -> &'static str {
        "custom"
    }

    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError>;
    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError>;
    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError>;
    async fn fetch_fasta_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError>;
    async fn fetch_fai_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError>;
    async fn fetch_release_gene_index_bytes(
        &self,
        dataset: &DatasetId,
    ) -> Result<Vec<u8>, CacheError>;

    async fn registry_health(&self) -> Vec<RegistrySourceHealth> {
        vec![RegistrySourceHealth {
            name: "primary".to_string(),
            priority: 0,
            healthy: true,
            last_error: None,
            shadowed_datasets: 0,
            ttl_seconds: 0,
        }]
    }
}
