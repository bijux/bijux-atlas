use bijux_atlas_policies::PolicyConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryLimits {
    pub max_limit: usize,
    pub max_region_span: u64,
    pub min_prefix_len: usize,
    pub max_prefix_len: usize,
    pub max_work_units: u64,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_limit: 500,
            max_region_span: 5_000_000,
            min_prefix_len: 1,
            max_prefix_len: 64,
            max_work_units: 2_000,
        }
    }
}

impl QueryLimits {
    #[must_use]
    pub fn from_policy(policy: &PolicyConfig) -> Self {
        Self {
            max_limit: policy.query_budget.max_limit as usize,
            max_region_span: policy.query_budget.max_region_span,
            min_prefix_len: 1,
            max_prefix_len: policy.query_budget.max_prefix_length as usize,
            max_work_units: 2_000,
        }
    }
}
