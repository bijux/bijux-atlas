// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryLimits {
    pub max_limit: usize,
    pub max_transcript_limit: usize,
    pub max_region_span: u64,
    pub max_region_estimated_rows: u64,
    pub max_prefix_cost_units: u64,
    pub heavy_projection_limit: usize,
    pub min_prefix_len: usize,
    pub max_prefix_len: usize,
    pub max_work_units: u64,
    pub max_serialization_bytes: usize,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_limit: 500,
            max_transcript_limit: 500,
            max_region_span: 5_000_000,
            max_region_estimated_rows: 250_000,
            max_prefix_cost_units: 80_000,
            heavy_projection_limit: 200,
            min_prefix_len: 2,
            max_prefix_len: 64,
            max_work_units: 2_000,
            max_serialization_bytes: 512 * 1024,
        }
    }
}
