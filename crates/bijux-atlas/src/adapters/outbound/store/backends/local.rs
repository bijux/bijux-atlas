// SPDX-License-Identifier: Apache-2.0

use super::super::catalog::validate_catalog_strict;
use super::super::manifest::ManifestLock;
use super::super::paths::{
    dataset_artifact_paths, manifest_lock_path, publish_lock_path, CATALOG_FILE,
};
use crate::app::ports::store::{
    ArtifactStore, NoopInstrumentation, PublishLockGuard, StoreError, StoreErrorCode,
    StoreInstrumentation,
};
use crate::domain::dataset::{ArtifactManifest, Catalog, DatasetId};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

pub struct LocalFsStore {
    pub root: PathBuf,
    instrumentation: Arc<dyn StoreInstrumentation>,
}

impl LocalFsStore {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            instrumentation: Arc::new(NoopInstrumentation),
        }
    }

    #[must_use]
    pub fn with_instrumentation(mut self, instrumentation: Arc<dyn StoreInstrumentation>) -> Self {
        self.instrumentation = instrumentation;
        self
    }
}

impl ArtifactStore for LocalFsStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError> {
        let catalog_path = self.root.join(CATALOG_FILE);
        if !catalog_path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(catalog_path)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        let catalog: Catalog = serde_json::from_str(&raw)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        validate_catalog_strict(&catalog)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        Ok(catalog.datasets.into_iter().map(|x| x.dataset).collect())
    }

    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        let lock_path = manifest_lock_path(Path::new(&self.root), dataset);
        let raw = fs::read(&paths.manifest)
            .map_err(|e| StoreError::new(StoreErrorCode::NotFound, e.to_string()))?;
        let sqlite = fs::read(&paths.sqlite)
            .map_err(|e| StoreError::new(StoreErrorCode::NotFound, e.to_string()))?;

        let lock_raw = fs::read_to_string(&lock_path).map_err(|e| {
            StoreError::new(
                StoreErrorCode::Validation,
                format!("missing manifest.lock: {e}"),
            )
        })?;
        let lock: ManifestLock = serde_json::from_str(&lock_raw)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        lock.validate(&raw, &sqlite)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;

        let manifest: ArtifactManifest = serde_json::from_slice(&raw)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        manifest
            .validate_strict()
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        Ok(manifest)
    }

    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        fs::read(paths.sqlite).map_err(|e| StoreError::new(StoreErrorCode::NotFound, e.to_string()))
    }

    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        let started = Instant::now();
        let _guard = self.acquire_publish_lock(dataset)?;
        enforce_dataset_immutability(&self.root, dataset)?;

        super::super::manifest::verify_expected_sha256(manifest_bytes, expected_manifest_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        super::super::manifest::verify_expected_sha256(sqlite_bytes, expected_sqlite_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;

        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        fs::create_dir_all(&paths.derived_dir)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;

        let manifest_tmp = paths.derived_dir.join("manifest.json.tmp");
        let sqlite_tmp = paths.derived_dir.join("gene_summary.sqlite.tmp");
        let lock_tmp = paths.derived_dir.join("manifest.lock.tmp");

        write_and_sync(&manifest_tmp, manifest_bytes)?;
        write_and_sync(&sqlite_tmp, sqlite_bytes)?;
        let lock = ManifestLock::from_bytes(manifest_bytes, sqlite_bytes);
        let lock_bytes = serde_json::to_vec(&lock)
            .map_err(|e| StoreError::new(StoreErrorCode::Internal, e.to_string()))?;
        write_and_sync(&lock_tmp, &lock_bytes)?;

        fs::rename(&manifest_tmp, &paths.manifest)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        fs::rename(&sqlite_tmp, &paths.sqlite)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        fs::rename(
            &lock_tmp,
            manifest_lock_path(Path::new(&self.root), dataset),
        )
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;

        sync_dir(&paths.derived_dir)?;

        self.instrumentation.observe_upload(
            "localfs",
            manifest_bytes.len() + sqlite_bytes.len(),
            started.elapsed(),
        );
        Ok(())
    }

    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        Ok(paths.manifest.exists() && paths.sqlite.exists())
    }

    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        fs::create_dir_all(&paths.derived_dir)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        let lock_path = publish_lock_path(Path::new(&self.root), dataset);
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(_) => Ok(PublishLockGuard::new(lock_path)),
            Err(e) => Err(StoreError::new(
                StoreErrorCode::Conflict,
                format!("failed to acquire publish lock: {e}"),
            )),
        }
    }
}

fn enforce_dataset_immutability(root: &Path, dataset: &DatasetId) -> Result<(), StoreError> {
    let paths = dataset_artifact_paths(root, dataset);
    if paths.manifest.exists() || paths.sqlite.exists() {
        return Err(StoreError::new(
            StoreErrorCode::Conflict,
            "dataset already published and immutable; existing artifacts must not be overwritten",
        ));
    }
    Ok(())
}

fn write_and_sync(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut file = std::fs::File::create(path)
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    file.write_all(bytes)
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    file.sync_all()
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    Ok(())
}

fn sync_dir(dir: &Path) -> Result<(), StoreError> {
    let file = OpenOptions::new()
        .read(true)
        .open(dir)
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    file.sync_all()
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    Ok(())
}
