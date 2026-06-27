// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    NodeUnreachable,
    ShardCorruption,
    ReplicaLag,
    NetworkPartition,
    ResourceExhaustion,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureDetectionPolicy {
    pub node_timeout_ms: u64,
    pub replica_lag_threshold_ms: u64,
    pub recovery_retry_budget: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPolicy {
    pub auto_recovery_enabled: bool,
    pub shard_failover_enabled: bool,
    pub replica_failover_enabled: bool,
    pub rebalance_after_recovery: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResilienceGuarantees {
    pub failover_within_ms: u64,
    pub diagnostics_available: bool,
    pub event_logging_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureEvent {
    pub event_id: String,
    pub category: FailureCategory,
    pub target_id: String,
    pub detected_at_unix_ms: u64,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryEvent {
    pub event_id: String,
    pub target_id: String,
    pub action: String,
    pub started_at_unix_ms: u64,
    pub completed_at_unix_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResilienceMetrics {
    pub failure_events_total: u64,
    pub recovery_events_total: u64,
    pub successful_recoveries_total: u64,
    pub failed_recoveries_total: u64,
    pub recovery_latency_avg_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryDiagnostics {
    pub detection_policy: FailureDetectionPolicy,
    pub recovery_policy: RecoveryPolicy,
    pub guarantees: ResilienceGuarantees,
    pub metrics: ResilienceMetrics,
    pub recent_failures: Vec<FailureEvent>,
    pub recent_recoveries: Vec<RecoveryEvent>,
}

#[derive(Debug, Clone)]
pub struct FailureRecoveryRegistry {
    detection_policy: FailureDetectionPolicy,
    recovery_policy: RecoveryPolicy,
    guarantees: ResilienceGuarantees,
    failures: BTreeMap<String, FailureEvent>,
    recoveries: BTreeMap<String, RecoveryEvent>,
    sequence: u64,
}

impl FailureRecoveryRegistry {
    #[must_use]
    pub fn new(
        detection_policy: FailureDetectionPolicy,
        recovery_policy: RecoveryPolicy,
        guarantees: ResilienceGuarantees,
    ) -> Self {
        Self {
            detection_policy,
            recovery_policy,
            guarantees,
            failures: BTreeMap::new(),
            recoveries: BTreeMap::new(),
            sequence: 0,
        }
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.sequence = self.sequence.saturating_add(1);
        format!("{prefix}-{:06}", self.sequence)
    }

    pub fn record_failure(
        &mut self,
        category: FailureCategory,
        target_id: impl Into<String>,
        detected_at_unix_ms: u64,
        detail: impl Into<String>,
    ) -> String {
        let event_id = self.next_id("failure");
        let event = FailureEvent {
            event_id: event_id.clone(),
            category,
            target_id: target_id.into(),
            detected_at_unix_ms,
            detail: detail.into(),
        };
        self.failures.insert(event_id.clone(), event);
        event_id
    }

    pub fn record_recovery(
        &mut self,
        target_id: impl Into<String>,
        action: impl Into<String>,
        started_at_unix_ms: u64,
        completed_at_unix_ms: u64,
        success: bool,
    ) -> String {
        let event_id = self.next_id("recovery");
        let event = RecoveryEvent {
            event_id: event_id.clone(),
            target_id: target_id.into(),
            action: action.into(),
            started_at_unix_ms,
            completed_at_unix_ms,
            success,
        };
        self.recoveries.insert(event_id.clone(), event);
        event_id
    }

    #[must_use]
    pub fn detection_policy(&self) -> &FailureDetectionPolicy {
        &self.detection_policy
    }

    #[must_use]
    pub fn recovery_policy(&self) -> &RecoveryPolicy {
        &self.recovery_policy
    }

    #[must_use]
    pub fn guarantees(&self) -> &ResilienceGuarantees {
        &self.guarantees
    }

    #[must_use]
    pub fn metrics(&self) -> ResilienceMetrics {
        let failure_events_total = self.failures.len() as u64;
        let recovery_events_total = self.recoveries.len() as u64;
        let successful_recoveries_total =
            self.recoveries.values().filter(|e| e.success).count() as u64;
        let failed_recoveries_total =
            recovery_events_total.saturating_sub(successful_recoveries_total);
        let total_latency_ms = self
            .recoveries
            .values()
            .map(|event| {
                event
                    .completed_at_unix_ms
                    .saturating_sub(event.started_at_unix_ms)
            })
            .sum::<u64>();
        let recovery_latency_avg_ms = if recovery_events_total == 0 {
            0
        } else {
            total_latency_ms / recovery_events_total
        };
        ResilienceMetrics {
            failure_events_total,
            recovery_events_total,
            successful_recoveries_total,
            failed_recoveries_total,
            recovery_latency_avg_ms,
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> RecoveryDiagnostics {
        let mut recent_failures = self.failures.values().cloned().collect::<Vec<_>>();
        recent_failures.sort_by(|a, b| b.detected_at_unix_ms.cmp(&a.detected_at_unix_ms));
        recent_failures.truncate(20);
        let mut recent_recoveries = self.recoveries.values().cloned().collect::<Vec<_>>();
        recent_recoveries.sort_by(|a, b| b.completed_at_unix_ms.cmp(&a.completed_at_unix_ms));
        recent_recoveries.truncate(20);
        RecoveryDiagnostics {
            detection_policy: self.detection_policy.clone(),
            recovery_policy: self.recovery_policy.clone(),
            guarantees: self.guarantees.clone(),
            metrics: self.metrics(),
            recent_failures,
            recent_recoveries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FailureCategory, FailureDetectionPolicy, FailureRecoveryRegistry, RecoveryPolicy,
        ResilienceGuarantees,
    };

    fn registry() -> FailureRecoveryRegistry {
        FailureRecoveryRegistry::new(
            FailureDetectionPolicy {
                node_timeout_ms: 5_000,
                replica_lag_threshold_ms: 2_000,
                recovery_retry_budget: 3,
            },
            RecoveryPolicy {
                auto_recovery_enabled: true,
                shard_failover_enabled: true,
                replica_failover_enabled: true,
                rebalance_after_recovery: true,
            },
            ResilienceGuarantees {
                failover_within_ms: 10_000,
                diagnostics_available: true,
                event_logging_required: true,
            },
        )
    }

    #[test]
    fn resilience_registry_tracks_failure_and_recovery_metrics() {
        let mut registry = registry();
        registry.record_failure(
            FailureCategory::NodeUnreachable,
            "node-a",
            1_000,
            "heartbeat timeout",
        );
        registry.record_recovery("node-a", "automatic_recovery", 1_010, 1_300, true);
        let metrics = registry.metrics();
        assert_eq!(metrics.failure_events_total, 1);
        assert_eq!(metrics.recovery_events_total, 1);
        assert_eq!(metrics.successful_recoveries_total, 1);
        assert_eq!(metrics.failed_recoveries_total, 0);
        assert_eq!(metrics.recovery_latency_avg_ms, 290);
    }

    #[test]
    fn resilience_registry_exposes_diagnostics_payload() {
        let mut registry = registry();
        registry.record_failure(
            FailureCategory::NetworkPartition,
            "node-b",
            5_000,
            "simulated partition",
        );
        let diagnostics = registry.diagnostics();
        assert_eq!(diagnostics.metrics.failure_events_total, 1);
        assert_eq!(diagnostics.recent_failures.len(), 1);
        assert_eq!(diagnostics.recent_failures[0].target_id, "node-b");
    }

    #[test]
    fn resilience_failure_recovery_sequence_records_expected_counts() {
        let mut registry = registry();
        registry.record_failure(
            FailureCategory::ShardCorruption,
            "atlas-default-s001",
            10,
            "corruption",
        );
        registry.record_recovery("atlas-default-s001", "shard_failover", 11, 20, true);
        let metrics = registry.metrics();
        assert_eq!(metrics.failure_events_total, 1);
        assert_eq!(metrics.recovery_events_total, 1);
    }

    #[test]
    fn resilience_recovery_correctness_tracks_failed_recovery() {
        let mut registry = registry();
        registry.record_failure(FailureCategory::NodeUnreachable, "node-a", 100, "timeout");
        registry.record_recovery("node-a", "node_recovery", 101, 190, false);
        let metrics = registry.metrics();
        assert_eq!(metrics.failed_recoveries_total, 1);
        assert_eq!(metrics.successful_recoveries_total, 0);
    }

    #[test]
    fn resilience_network_partition_event_is_captured() {
        let mut registry = registry();
        registry.record_failure(
            FailureCategory::NetworkPartition,
            "node-b",
            1_000,
            "partition between node-a and node-b",
        );
        let diagnostics = registry.diagnostics();
        assert!(diagnostics
            .recent_failures
            .iter()
            .any(|event| event.target_id == "node-b"));
    }

    #[test]
    fn resilience_stress_records_many_failure_and_recovery_events() {
        let mut registry = registry();
        for i in 0..1_000_u64 {
            registry.record_failure(
                FailureCategory::Unknown,
                format!("target-{i}"),
                i,
                "load test",
            );
            registry.record_recovery(
                format!("target-{i}"),
                "automatic_recovery",
                i,
                i.saturating_add(5),
                true,
            );
        }
        let metrics = registry.metrics();
        assert_eq!(metrics.failure_events_total, 1_000);
        assert_eq!(metrics.recovery_events_total, 1_000);
    }
}
