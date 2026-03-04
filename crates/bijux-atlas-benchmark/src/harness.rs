// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkMetrics {
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub throughput_ops_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkResult {
    pub schema_version: u32,
    pub benchmark_id: String,
    pub dataset_id: String,
    pub metrics: BenchmarkMetrics,
}

impl BenchmarkResult {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("benchmark result must use schema_version=1".to_string());
        }
        if self.benchmark_id.trim().is_empty() {
            return Err("benchmark_id must not be empty".to_string());
        }
        if self.dataset_id.trim().is_empty() {
            return Err("dataset_id must not be empty".to_string());
        }
        if self.metrics.latency_p50_ms < 0.0
            || self.metrics.latency_p95_ms < 0.0
            || self.metrics.latency_p99_ms < 0.0
            || self.metrics.throughput_ops_per_sec <= 0.0
        {
            return Err("benchmark metrics contain invalid values".to_string());
        }
        Ok(())
    }
}
