// SPDX-License-Identifier: Apache-2.0

use super::{PublishLockGuard, StoreError, StoreErrorCode};
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactManifest, DatasetId};

pub trait ArtifactStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError>;
    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError>;
    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError>;
    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError>;
    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError>;

    fn read_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        self.get_manifest(dataset)
    }

    fn get_sqlite_bytes_verified(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        let manifest = self.get_manifest(dataset)?;
        let sqlite_bytes = self.get_sqlite_bytes(dataset)?;
        let actual = sha256_hex(&sqlite_bytes);
        if actual != manifest.checksums.sqlite_sha256 {
            return Err(StoreError::new(
                StoreErrorCode::Validation,
                format!(
                    "sha256 mismatch expected={} actual={actual}",
                    manifest.checksums.sqlite_sha256
                ),
            ));
        }
        Ok(sqlite_bytes)
    }

    fn publish_atomic(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        self.put_dataset(
            dataset,
            manifest_bytes,
            sqlite_bytes,
            expected_manifest_sha256,
            expected_sqlite_sha256,
        )
    }

    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError>;
}
