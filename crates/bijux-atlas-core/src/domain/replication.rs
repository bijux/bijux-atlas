// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationPolicy {
    pub replication_factor: usize,
    pub primary_required: bool,
    pub max_replication_lag_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyLevel {
    Eventual,
    Quorum,
    Strong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsistencyGuarantee {
    pub read_consistency: ConsistencyLevel,
    pub write_consistency: ConsistencyLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaMetadata {
    pub dataset_id: String,
    pub shard_id: String,
    pub primary_node_id: String,
    pub replica_node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaSyncState {
    pub last_applied_lsn: u64,
    pub primary_lsn: u64,
    pub lag_ms: u64,
    pub sync_throughput_rows_per_second: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaHealth {
    pub healthy: bool,
    pub failed_checks: u64,
    pub last_failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaRecord {
    pub metadata: ReplicaMetadata,
    pub sync: ReplicaSyncState,
    pub health: ReplicaHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationMetrics {
    pub replica_groups_total: usize,
    pub healthy_replica_groups_total: usize,
    pub replica_failures_total: u64,
    pub average_lag_ms: u64,
    pub total_sync_throughput_rows_per_second: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaDiagnostics {
    pub replica: ReplicaRecord,
    pub consistency: ConsistencyGuarantee,
    pub policy: ReplicationPolicy,
}

#[derive(Debug, Clone)]
pub struct ReplicaRegistry {
    policy: ReplicationPolicy,
    consistency: ConsistencyGuarantee,
    replicas: BTreeMap<String, ReplicaRecord>,
    failover_events_total: u64,
}

impl ReplicaRegistry {
    #[must_use]
    pub fn new(policy: ReplicationPolicy, consistency: ConsistencyGuarantee) -> Self {
        Self {
            policy,
            consistency,
            replicas: BTreeMap::new(),
            failover_events_total: 0,
        }
    }

    fn key(dataset_id: &str, shard_id: &str) -> String {
        format!("{dataset_id}:{shard_id}")
    }

    pub fn upsert_replica(&mut self, replica: ReplicaRecord) {
        self.replicas.insert(
            Self::key(&replica.metadata.dataset_id, &replica.metadata.shard_id),
            replica,
        );
    }

    #[must_use]
    pub fn list(&self) -> Vec<&ReplicaRecord> {
        self.replicas.values().collect()
    }

    #[must_use]
    pub fn get(&self, dataset_id: &str, shard_id: &str) -> Option<&ReplicaRecord> {
        self.replicas.get(&Self::key(dataset_id, shard_id))
    }

    #[must_use]
    pub fn consistency(&self) -> &ConsistencyGuarantee {
        &self.consistency
    }

    #[must_use]
    pub fn policy(&self) -> &ReplicationPolicy {
        &self.policy
    }

    pub fn update_sync_progress(
        &mut self,
        dataset_id: &str,
        shard_id: &str,
        primary_lsn: u64,
        replica_lsn: u64,
        lag_ms: u64,
        throughput_rows_per_second: u64,
    ) -> bool {
        let Some(replica) = self.replicas.get_mut(&Self::key(dataset_id, shard_id)) else {
            return false;
        };
        replica.sync.primary_lsn = primary_lsn;
        replica.sync.last_applied_lsn = replica_lsn;
        replica.sync.lag_ms = lag_ms;
        replica.sync.sync_throughput_rows_per_second = throughput_rows_per_second;
        true
    }

    pub fn mark_replica_failure(
        &mut self,
        dataset_id: &str,
        shard_id: &str,
        reason: impl Into<String>,
    ) -> bool {
        let Some(replica) = self.replicas.get_mut(&Self::key(dataset_id, shard_id)) else {
            return false;
        };
        replica.health.healthy = false;
        replica.health.failed_checks = replica.health.failed_checks.saturating_add(1);
        replica.health.last_failure_reason = Some(reason.into());
        true
    }

    pub fn mark_replica_healthy(&mut self, dataset_id: &str, shard_id: &str) -> bool {
        let Some(replica) = self.replicas.get_mut(&Self::key(dataset_id, shard_id)) else {
            return false;
        };
        replica.health.healthy = true;
        replica.health.last_failure_reason = None;
        true
    }

    pub fn failover(&mut self, dataset_id: &str, shard_id: &str, promote_node_id: &str) -> bool {
        let Some(replica) = self.replicas.get_mut(&Self::key(dataset_id, shard_id)) else {
            return false;
        };
        if !replica
            .metadata
            .replica_node_ids
            .iter()
            .any(|node| node == promote_node_id)
        {
            return false;
        }
        let old_primary = replica.metadata.primary_node_id.clone();
        replica.metadata.primary_node_id = promote_node_id.to_string();
        replica.metadata.replica_node_ids.retain(|node| node != promote_node_id);
        replica.metadata.replica_node_ids.push(old_primary);
        replica.health.healthy = true;
        replica.health.last_failure_reason = None;
        self.failover_events_total = self.failover_events_total.saturating_add(1);
        true
    }

    #[must_use]
    pub fn diagnostics(&self, dataset_id: &str, shard_id: &str) -> Option<ReplicaDiagnostics> {
        self.get(dataset_id, shard_id).map(|replica| ReplicaDiagnostics {
            replica: replica.clone(),
            consistency: self.consistency.clone(),
            policy: self.policy.clone(),
        })
    }

    #[must_use]
    pub fn metrics(&self) -> ReplicationMetrics {
        let replica_groups_total = self.replicas.len();
        let healthy_replica_groups_total = self
            .replicas
            .values()
            .filter(|replica| replica.health.healthy)
            .count();
        let replica_failures_total = self
            .replicas
            .values()
            .map(|replica| replica.health.failed_checks)
            .sum::<u64>();
        let total_lag_ms = self
            .replicas
            .values()
            .map(|replica| replica.sync.lag_ms)
            .sum::<u64>();
        let total_sync_throughput_rows_per_second = self
            .replicas
            .values()
            .map(|replica| replica.sync.sync_throughput_rows_per_second)
            .sum::<u64>();
        let average_lag_ms = if replica_groups_total == 0 {
            0
        } else {
            total_lag_ms / replica_groups_total as u64
        };
        ReplicationMetrics {
            replica_groups_total,
            healthy_replica_groups_total,
            replica_failures_total: replica_failures_total
                .saturating_add(self.failover_events_total),
            average_lag_ms,
            total_sync_throughput_rows_per_second,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConsistencyGuarantee, ConsistencyLevel, ReplicaHealth, ReplicaMetadata, ReplicaRecord,
        ReplicaRegistry, ReplicaSyncState, ReplicationPolicy,
    };

    fn registry() -> ReplicaRegistry {
        ReplicaRegistry::new(
            ReplicationPolicy {
                replication_factor: 2,
                primary_required: true,
                max_replication_lag_ms: 2_000,
            },
            ConsistencyGuarantee {
                read_consistency: ConsistencyLevel::Quorum,
                write_consistency: ConsistencyLevel::Quorum,
            },
        )
    }

    fn record() -> ReplicaRecord {
        ReplicaRecord {
            metadata: ReplicaMetadata {
                dataset_id: "atlas-default".to_string(),
                shard_id: "atlas-default-s001".to_string(),
                primary_node_id: "node-a".to_string(),
                replica_node_ids: vec!["node-b".to_string()],
            },
            sync: ReplicaSyncState {
                last_applied_lsn: 1_000,
                primary_lsn: 1_050,
                lag_ms: 25,
                sync_throughput_rows_per_second: 15_000,
            },
            health: ReplicaHealth {
                healthy: true,
                failed_checks: 0,
                last_failure_reason: None,
            },
        }
    }

    #[test]
    fn replication_registry_supports_sync_health_and_failover() {
        let mut registry = registry();
        registry.upsert_replica(record());

        let updated = registry.update_sync_progress(
            "atlas-default",
            "atlas-default-s001",
            1_100,
            1_090,
            10,
            16_000,
        );
        assert!(updated);

        let failed = registry.mark_replica_failure(
            "atlas-default",
            "atlas-default-s001",
            "replica timeout on storage link",
        );
        assert!(failed);
        let promoted = registry.failover("atlas-default", "atlas-default-s001", "node-b");
        assert!(promoted);
        let current = registry
            .get("atlas-default", "atlas-default-s001")
            .expect("replica should exist");
        assert_eq!(current.metadata.primary_node_id, "node-b");
        assert!(current.health.healthy);
    }

    #[test]
    fn replication_metrics_capture_lag_throughput_and_failure_counts() {
        let mut registry = registry();
        registry.upsert_replica(record());
        registry.mark_replica_failure("atlas-default", "atlas-default-s001", "io timeout");
        let metrics = registry.metrics();
        assert_eq!(metrics.replica_groups_total, 1);
        assert_eq!(metrics.healthy_replica_groups_total, 0);
        assert_eq!(metrics.average_lag_ms, 25);
        assert_eq!(metrics.total_sync_throughput_rows_per_second, 15_000);
        assert_eq!(metrics.replica_failures_total, 1);
    }
}
