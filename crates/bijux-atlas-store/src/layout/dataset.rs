// SPDX-License-Identifier: Apache-2.0

use super::{MANIFEST_FILE, MANIFEST_LOCK_FILE, SQLITE_FILE};
use bijux_atlas_model::{artifact_paths, ArtifactPaths, DatasetId};
use std::path::Path;

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
