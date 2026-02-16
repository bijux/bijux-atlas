#![forbid(unsafe_code)]

use bijux_atlas_model::{ArtifactManifest, DatasetId};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

pub const CRATE_NAME: &str = "bijux-atlas-store";

#[derive(Debug)]
pub struct StoreError(pub String);

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StoreError {}

pub trait ArtifactStore {
    fn read_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError>;
    fn publish_atomic(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError>;
    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError>;
}

pub struct LocalFsStore {
    pub root: PathBuf,
}

impl LocalFsStore {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl ArtifactStore for LocalFsStore {
    fn read_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        let path = bijux_atlas_model::artifact_paths(Path::new(&self.root), dataset).manifest;
        let raw = fs::read_to_string(path).map_err(|e| StoreError(e.to_string()))?;
        let manifest: ArtifactManifest =
            serde_json::from_str(&raw).map_err(|e| StoreError(e.to_string()))?;
        manifest
            .validate_strict()
            .map_err(|e| StoreError(e.to_string()))?;
        Ok(manifest)
    }

    fn publish_atomic(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        let _lock = self.acquire_publish_lock(dataset)?;
        enforce_dataset_immutability(&self.root, dataset)?;

        let paths = bijux_atlas_model::artifact_paths(Path::new(&self.root), dataset);
        fs::create_dir_all(&paths.derived_dir).map_err(|e| StoreError(e.to_string()))?;

        let manifest_tmp = paths.derived_dir.join("manifest.json.tmp");
        let sqlite_tmp = paths.derived_dir.join("gene_summary.sqlite.tmp");

        fs::write(&manifest_tmp, manifest_bytes).map_err(|e| StoreError(e.to_string()))?;
        fs::write(&sqlite_tmp, sqlite_bytes).map_err(|e| StoreError(e.to_string()))?;

        let manifest_actual = bijux_atlas_core::sha256_hex(
            &fs::read(&manifest_tmp).map_err(|e| StoreError(e.to_string()))?,
        );
        let sqlite_actual = bijux_atlas_core::sha256_hex(
            &fs::read(&sqlite_tmp).map_err(|e| StoreError(e.to_string()))?,
        );
        if manifest_actual != expected_manifest_sha256 {
            return Err(StoreError(
                "manifest checksum verification failed during atomic publish".to_string(),
            ));
        }
        if sqlite_actual != expected_sqlite_sha256 {
            return Err(StoreError(
                "sqlite checksum verification failed during atomic publish".to_string(),
            ));
        }

        fs::rename(&manifest_tmp, &paths.manifest).map_err(|e| StoreError(e.to_string()))?;
        fs::rename(&sqlite_tmp, &paths.sqlite).map_err(|e| StoreError(e.to_string()))?;
        Ok(())
    }

    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        let paths = bijux_atlas_model::artifact_paths(Path::new(&self.root), dataset);
        fs::create_dir_all(&paths.derived_dir).map_err(|e| StoreError(e.to_string()))?;
        let lock_path = paths.derived_dir.join(".publish.lock");
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(_) => Ok(PublishLockGuard { lock_path }),
            Err(e) => Err(StoreError(format!(
                "failed to acquire single-writer publish lock: {e}"
            ))),
        }
    }
}

pub struct S3LikeStore {
    pub bucket: String,
}

impl S3LikeStore {
    #[must_use]
    pub fn new(bucket: String) -> Self {
        Self { bucket }
    }
}

impl ArtifactStore for S3LikeStore {
    fn read_manifest(&self, _dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        Err(StoreError(format!(
            "s3-like backend is not wired yet for bucket {}",
            self.bucket
        )))
    }

    fn publish_atomic(
        &self,
        _dataset: &DatasetId,
        _manifest_bytes: &[u8],
        _sqlite_bytes: &[u8],
        _expected_manifest_sha256: &str,
        _expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        Err(StoreError(
            "s3-like publish is not wired yet (reserved backend surface)".to_string(),
        ))
    }

    fn acquire_publish_lock(&self, _dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        Err(StoreError(
            "s3-like locking is not wired yet (reserved backend surface)".to_string(),
        ))
    }
}

pub struct HttpReadonlyStore {
    pub base_url: String,
}

impl HttpReadonlyStore {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl ArtifactStore for HttpReadonlyStore {
    fn read_manifest(&self, _dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        Err(StoreError(format!(
            "http readonly backend is not wired yet for base URL {}",
            self.base_url
        )))
    }

    fn publish_atomic(
        &self,
        _dataset: &DatasetId,
        _manifest_bytes: &[u8],
        _sqlite_bytes: &[u8],
        _expected_manifest_sha256: &str,
        _expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        Err(StoreError(
            "http readonly backend cannot publish artifacts".to_string(),
        ))
    }

    fn acquire_publish_lock(&self, _dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        Err(StoreError(
            "http readonly backend cannot acquire publish lock".to_string(),
        ))
    }
}

pub fn enforce_dataset_immutability(root: &Path, dataset: &DatasetId) -> Result<(), StoreError> {
    let paths = bijux_atlas_model::artifact_paths(root, dataset);
    if paths.manifest.exists() || paths.sqlite.exists() {
        return Err(StoreError(
            "dataset already published; immutable artifacts must not be overwritten".to_string(),
        ));
    }
    Ok(())
}

pub struct PublishLockGuard {
    lock_path: PathBuf,
}

impl Drop for PublishLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}
