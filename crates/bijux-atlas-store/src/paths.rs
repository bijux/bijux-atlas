use bijux_atlas_model::{artifact_paths, ArtifactPaths, DatasetId};
use std::path::{Path, PathBuf};

pub const CATALOG_FILE: &str = "catalog.json";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const SQLITE_FILE: &str = "gene_summary.sqlite";
pub const MANIFEST_LOCK_FILE: &str = "manifest.lock";
pub const PUBLISH_LOCK_FILE: &str = ".publish.lock";

#[must_use]
pub fn dataset_artifact_paths(root: &Path, dataset: &DatasetId) -> ArtifactPaths {
    artifact_paths(root, dataset)
}

#[must_use]
pub fn dataset_key_prefix(dataset: &DatasetId) -> String {
    dataset.canonical_string()
}

#[must_use]
pub fn dataset_manifest_key(dataset: &DatasetId) -> String {
    format!("{}/{}", dataset_key_prefix(dataset), MANIFEST_FILE)
}

#[must_use]
pub fn dataset_sqlite_key(dataset: &DatasetId) -> String {
    format!("{}/{}", dataset_key_prefix(dataset), SQLITE_FILE)
}

#[must_use]
pub fn dataset_manifest_lock_key(dataset: &DatasetId) -> String {
    format!("{}/{}", dataset_key_prefix(dataset), MANIFEST_LOCK_FILE)
}

#[must_use]
pub fn manifest_lock_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    dataset_artifact_paths(root, dataset)
        .derived_dir
        .join(MANIFEST_LOCK_FILE)
}

#[must_use]
pub fn publish_lock_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    dataset_artifact_paths(root, dataset)
        .derived_dir
        .join(PUBLISH_LOCK_FILE)
}
