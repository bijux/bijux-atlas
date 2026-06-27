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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkSummary {
    pub schema_version: u32,
    pub benchmark_id: String,
    pub sample_count: usize,
    pub latency_p99_ms: f64,
    pub throughput_ops_per_sec: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkHistoryEntry {
    pub run_id: String,
    pub result: BenchmarkResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkDiff {
    pub benchmark_id: String,
    pub dataset_id: String,
    pub latency_p99_delta_ms: f64,
    pub throughput_delta_ops_per_sec: f64,
    pub regressed: bool,
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

    pub fn to_summary(&self, sample_count: usize) -> BenchmarkSummary {
        BenchmarkSummary {
            schema_version: 1,
            benchmark_id: self.benchmark_id.clone(),
            sample_count,
            latency_p99_ms: self.metrics.latency_p99_ms,
            throughput_ops_per_sec: self.metrics.throughput_ops_per_sec,
            status: "ok".to_string(),
        }
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{:.3},{:.3},{:.3},{:.3}",
            self.benchmark_id,
            self.dataset_id,
            self.metrics.latency_p50_ms,
            self.metrics.latency_p95_ms,
            self.metrics.latency_p99_ms,
            self.metrics.throughput_ops_per_sec
        )
    }

    pub fn csv_header() -> &'static str {
        "benchmark_id,dataset_id,latency_p50_ms,latency_p95_ms,latency_p99_ms,throughput_ops_per_sec"
    }
}

pub fn compare_results(baseline: &BenchmarkResult, candidate: &BenchmarkResult) -> BenchmarkDiff {
    let latency_delta = candidate.metrics.latency_p99_ms - baseline.metrics.latency_p99_ms;
    let throughput_delta =
        candidate.metrics.throughput_ops_per_sec - baseline.metrics.throughput_ops_per_sec;
    BenchmarkDiff {
        benchmark_id: candidate.benchmark_id.clone(),
        dataset_id: candidate.dataset_id.clone(),
        latency_p99_delta_ms: latency_delta,
        throughput_delta_ops_per_sec: throughput_delta,
        regressed: latency_delta > 0.0 || throughput_delta < 0.0,
    }
}

pub fn reproducibility_ok(results: &[BenchmarkResult], max_relative_delta_percent: f64) -> bool {
    if results.len() < 2 {
        return false;
    }
    let mut min_p99 = f64::MAX;
    let mut max_p99 = f64::MIN;
    for result in results {
        min_p99 = min_p99.min(result.metrics.latency_p99_ms);
        max_p99 = max_p99.max(result.metrics.latency_p99_ms);
    }
    if min_p99 <= 0.0 {
        return false;
    }
    ((max_p99 - min_p99) / min_p99) * 100.0 <= max_relative_delta_percent
}
