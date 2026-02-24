// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::{ArtifactManifest, DatasetId};

use crate::backend::{ArtifactStore, PublishLockGuard, StoreError};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct StorePath(String);

impl StorePath {
    pub fn parse(value: impl Into<String>) -> Result<Self, StoreError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(StoreError::new(
                crate::backend::StoreErrorCode::Validation,
                "store path must not be empty",
            ));
        }
        if value.starts_with('/') || value.contains("..") {
            return Err(StoreError::new(
                crate::backend::StoreErrorCode::Validation,
                "store path must be relative and normalized",
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    pub dataset: DatasetId,
    pub manifest_path: StorePath,
    pub sqlite_path: StorePath,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogRef {
    pub catalog_path: StorePath,
}

pub trait StoreRead {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError>;
    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError>;
    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError>;
    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError>;
}

pub trait StoreWrite {
    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError>;
}

pub trait StoreAdmin {
    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError>;
}

impl<T: ArtifactStore + ?Sized> StoreRead for T {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError> {
        ArtifactStore::list_datasets(self)
    }

    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        ArtifactStore::get_manifest(self, dataset)
    }

    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        ArtifactStore::get_sqlite_bytes(self, dataset)
    }

    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError> {
        ArtifactStore::exists(self, dataset)
    }
}

impl<T: ArtifactStore + ?Sized> StoreWrite for T {
    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        ArtifactStore::put_dataset(
            self,
            dataset,
            manifest_bytes,
            sqlite_bytes,
            expected_manifest_sha256,
            expected_sqlite_sha256,
        )
    }
}

impl<T: ArtifactStore + ?Sized> StoreAdmin for T {
    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        ArtifactStore::acquire_publish_lock(self, dataset)
    }
}
