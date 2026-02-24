// SPDX-License-Identifier: Apache-2.0

use crate::dto::DatasetKeyDto;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiCompatibilityChannel {
    StableV1,
    LegacyV0Redirect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityRedirect {
    pub from_path: String,
    pub to_path: String,
    pub status_code: u16,
}

#[must_use]
pub fn legacy_dataset_path_redirect(dataset: &DatasetKeyDto) -> CompatibilityRedirect {
    CompatibilityRedirect {
        from_path: format!(
            "/v1/releases/{}/species/{}/assemblies/{}",
            dataset.release, dataset.species, dataset.assembly
        ),
        to_path: format!(
            "/v1/datasets/{}/{}/{}",
            dataset.release, dataset.species, dataset.assembly
        ),
        status_code: 308,
    }
}

#[must_use]
pub fn compatibility_channel(path: &str) -> ApiCompatibilityChannel {
    if path.starts_with("/v1/releases/") {
        ApiCompatibilityChannel::LegacyV0Redirect
    } else {
        ApiCompatibilityChannel::StableV1
    }
}
