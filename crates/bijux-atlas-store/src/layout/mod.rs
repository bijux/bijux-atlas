// SPDX-License-Identifier: Apache-2.0

mod dataset;
mod file_names;

pub use dataset::{
    dataset_artifact_paths, dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key,
    dataset_sqlite_key,
};
pub use file_names::{
    CATALOG_FILE, IMMUTABILITY_MARKER_FILE, LIFECYCLE_STATE_FILE, LIFECYCLE_TRANSITIONS_FILE,
    MANIFEST_FILE, MANIFEST_LOCK_FILE, PUBLISH_LOCK_FILE, SQLITE_FILE,
};

use bijux_atlas_model::DatasetId;
use std::path::{Path, PathBuf};

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

#[must_use]
pub fn immutability_marker_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    dataset_artifact_paths(root, dataset)
        .derived_dir
        .join(IMMUTABILITY_MARKER_FILE)
}

#[must_use]
pub fn lifecycle_state_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    dataset_artifact_paths(root, dataset)
        .derived_dir
        .join(LIFECYCLE_STATE_FILE)
}

#[must_use]
pub fn lifecycle_transitions_path(root: &Path, dataset: &DatasetId) -> PathBuf {
    dataset_artifact_paths(root, dataset)
        .derived_dir
        .join(LIFECYCLE_TRANSITIONS_FILE)
}
