// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use bijux_atlas::{core as bijux_atlas_core, model as bijux_atlas_model};

use bijux_atlas_model::DatasetId;

#[must_use]
#[allow(dead_code)]
pub(crate) fn dataset_route_path(dataset: &DatasetId) -> String {
    format!(
        "/v1/datasets/{}/{}/{}",
        dataset.release, dataset.species, dataset.assembly
    )
}
