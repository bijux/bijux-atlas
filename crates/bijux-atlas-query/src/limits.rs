// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_policies::PolicyConfig;
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

impl QueryLimits {
    #[must_use]
    pub fn from_policy(policy: &PolicyConfig) -> Self {
        Self {
            max_limit: policy.query_budget.max_limit as usize,
            max_transcript_limit: policy.query_budget.max_transcript_limit as usize,
            max_region_span: policy.query_budget.medium.max_region_span,
            max_region_estimated_rows: policy.query_budget.medium.max_region_estimated_rows,
            max_prefix_cost_units: policy.query_budget.medium.max_prefix_cost_units,
            heavy_projection_limit: policy.query_budget.heavy_projection_limit as usize,
            min_prefix_len: 2,
            max_prefix_len: policy.query_budget.max_prefix_length as usize,
            max_work_units: 2_000,
            max_serialization_bytes: policy.response_budget.max_serialization_bytes as usize,
        }
    }
}
