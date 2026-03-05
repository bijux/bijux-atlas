// SPDX-License-Identifier: Apache-2.0
//! Deterministic load harness model for query, ingest, and mixed workload generation.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadKind {
    Query,
    Ingest,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcurrencyProfile {
    SingleClient,
    MultiClient,
    Saturation,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadHarnessSpec {
    pub schema_version: u32,
    pub kind: WorkloadKind,
    pub concurrency_profile: ConcurrencyProfile,
    pub duration_secs: u32,
    pub target_rps: u32,
    pub ingest_ops_per_sec: u32,
    pub query_mix_read_ratio: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioPlan {
    pub id: &'static str,
    pub title: &'static str,
    pub kind: WorkloadKind,
    pub concurrency_profile: ConcurrencyProfile,
    pub summary: &'static str,
}

pub fn harness_spec(
    kind: WorkloadKind,
    concurrency_profile: ConcurrencyProfile,
    duration_secs: u32,
) -> LoadHarnessSpec {
    match kind {
        WorkloadKind::Query => LoadHarnessSpec {
            schema_version: 1,
            kind,
            concurrency_profile,
            duration_secs,
            target_rps: 1200,
            ingest_ops_per_sec: 0,
            query_mix_read_ratio: 1.0,
        },
        WorkloadKind::Ingest => LoadHarnessSpec {
            schema_version: 1,
            kind,
            concurrency_profile,
            duration_secs,
            target_rps: 0,
            ingest_ops_per_sec: 400,
            query_mix_read_ratio: 0.0,
        },
        WorkloadKind::Mixed => LoadHarnessSpec {
            schema_version: 1,
            kind,
            concurrency_profile,
            duration_secs,
            target_rps: 800,
            ingest_ops_per_sec: 120,
            query_mix_read_ratio: 0.8,
        },
    }
}

pub fn query_load_generator(duration_secs: u32) -> LoadHarnessSpec {
    harness_spec(
        WorkloadKind::Query,
        ConcurrencyProfile::MultiClient,
        duration_secs,
    )
}

pub fn ingest_load_generator(duration_secs: u32) -> LoadHarnessSpec {
    harness_spec(
        WorkloadKind::Ingest,
        ConcurrencyProfile::MultiClient,
        duration_secs,
    )
}

pub fn mixed_workload_generator(duration_secs: u32) -> LoadHarnessSpec {
    harness_spec(
        WorkloadKind::Mixed,
        ConcurrencyProfile::MultiClient,
        duration_secs,
    )
}

pub fn concurrency_stress_scenarios() -> Vec<ScenarioPlan> {
    vec![
        ScenarioPlan {
            id: "load-single-client-baseline",
            title: "Single client baseline",
            kind: WorkloadKind::Query,
            concurrency_profile: ConcurrencyProfile::SingleClient,
            summary: "Measure minimal overhead with one client and deterministic request pacing.",
        },
        ScenarioPlan {
            id: "load-multi-client-concurrency",
            title: "Multi client concurrency",
            kind: WorkloadKind::Mixed,
            concurrency_profile: ConcurrencyProfile::MultiClient,
            summary: "Exercise shared runtime resources with multi-client query and ingest overlap.",
        },
        ScenarioPlan {
            id: "load-saturation-stress",
            title: "Saturation stress",
            kind: WorkloadKind::Mixed,
            concurrency_profile: ConcurrencyProfile::Saturation,
            summary: "Push to controlled saturation to validate performance guardrails and queue behavior.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workload_generators_emit_expected_shapes() {
        let query = query_load_generator(300);
        assert_eq!(query.kind, WorkloadKind::Query);
        assert_eq!(query.target_rps, 1200);

        let ingest = ingest_load_generator(300);
        assert_eq!(ingest.kind, WorkloadKind::Ingest);
        assert_eq!(ingest.ingest_ops_per_sec, 400);

        let mixed = mixed_workload_generator(300);
        assert_eq!(mixed.kind, WorkloadKind::Mixed);
        assert!(mixed.query_mix_read_ratio > 0.0);
    }

    #[test]
    fn stress_scenarios_are_stable_and_complete() {
        let rows = concurrency_stress_scenarios();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].id, "load-single-client-baseline");
        assert_eq!(rows[1].id, "load-multi-client-concurrency");
        assert_eq!(rows[2].id, "load-saturation-stress");
    }
}
