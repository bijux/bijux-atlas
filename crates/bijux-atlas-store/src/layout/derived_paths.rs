// SPDX-License-Identifier: Apache-2.0

use super::{
    dataset_artifact_paths, IMMUTABILITY_MARKER_FILE, LIFECYCLE_STATE_FILE,
    LIFECYCLE_TRANSITIONS_FILE, MANIFEST_LOCK_FILE, PUBLISH_LOCK_FILE,
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
