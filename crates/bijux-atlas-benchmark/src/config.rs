// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolationConfig {
    pub cpu_set: String,
    pub memory_limit_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReproducibilityConfig {
    pub fixed_seed: u64,
    pub min_repeat_runs: u32,
    pub max_relative_delta_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkConfig {
    pub schema_version: u32,
    pub namespace: String,
    pub latency_unit: String,
    pub throughput_unit: String,
    pub isolation: IsolationConfig,
    pub reproducibility: ReproducibilityConfig,
}

impl BenchmarkConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("benchmark config must use schema_version=1".to_string());
        }
        if self.namespace.trim().is_empty() {
            return Err("benchmark namespace must not be empty".to_string());
        }
        if self.latency_unit != "milliseconds" {
            return Err("latency_unit must be `milliseconds`".to_string());
        }
        if self.throughput_unit != "operations_per_second" {
            return Err("throughput_unit must be `operations_per_second`".to_string());
        }
        if self.isolation.cpu_set.trim().is_empty() {
            return Err("isolation.cpu_set must not be empty".to_string());
        }
        if self.isolation.memory_limit_mb == 0 {
            return Err("isolation.memory_limit_mb must be greater than zero".to_string());
        }
        if self.reproducibility.min_repeat_runs < 2 {
            return Err("reproducibility.min_repeat_runs must be >= 2".to_string());
        }
        if !(0.0..=10.0).contains(&self.reproducibility.max_relative_delta_percent) {
            return Err("reproducibility.max_relative_delta_percent must be between 0 and 10"
                .to_string());
        }
        Ok(())
    }
}
