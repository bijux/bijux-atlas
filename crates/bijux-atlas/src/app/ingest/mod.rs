// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub use crate::domain::ingest::{IngestError, IngestOptions, IngestResult, TimestampPolicy};

pub fn ingest_dataset(options: &IngestOptions) -> Result<IngestResult, IngestError> {
    let mut runtime_options = options.clone();
    if runtime_options.build_hash.is_empty() {
        runtime_options.build_hash = crate::runtime::config::runtime_build_hash().to_string();
    }
    crate::domain::ingest::ingest_dataset(&runtime_options)
}

pub fn replay_normalized_counts(
    path: &Path,
) -> Result<crate::domain::ingest::ReplayCounts, IngestError> {
    crate::domain::ingest::replay_normalized_counts(path)
}

pub fn diff_normalized_ids(
    base_path: &Path,
    target_path: &Path,
) -> Result<(Vec<String>, Vec<String>), IngestError> {
    crate::domain::ingest::diff_normalized_ids(base_path, target_path)
}
