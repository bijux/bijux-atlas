use bijux_atlas_model::{artifact_paths, ArtifactPaths, DatasetId};
use std::path::{Path, PathBuf};

#[must_use]
pub fn dataset_artifact_paths(root: &Path, dataset: &DatasetId) -> ArtifactPaths {
    artifact_paths(root, dataset)
}

#[must_use]
pub fn manifest_lock_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    dataset_artifact_paths(root, dataset)
        .derived_dir
        .join("manifest.lock")
}

#[must_use]
pub fn publish_lock_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    dataset_artifact_paths(root, dataset)
        .derived_dir
        .join(".publish.lock")
}
