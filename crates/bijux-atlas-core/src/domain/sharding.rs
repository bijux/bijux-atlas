// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardKeyStrategy {
    pub key_field: String,
    pub hash_seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetShardLayout {
    pub dataset_id: String,
    pub shard_count: usize,
    pub partition_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardOwnershipRule {
    pub min_owner_count: usize,
    pub allow_owner_transfer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardMetadata {
    pub shard_id: String,
    pub dataset_id: String,
    pub key_range_start: String,
    pub key_range_end: String,
    pub owner_node_id: String,
    pub replica_node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardHealth {
    pub healthy: bool,
    pub open_errors: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardRuntimeStats {
    pub load: u64,
    pub access_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub latency_sum_ms: u64,
    pub latency_samples: u64,
}

impl ShardRuntimeStats {
    #[must_use]
    pub fn latency_avg_ms(&self) -> u64 {
        if self.latency_samples == 0 {
            0
        } else {
            self.latency_sum_ms / self.latency_samples
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardRecord {
    pub metadata: ShardMetadata,
    pub health: ShardHealth,
    pub stats: ShardRuntimeStats,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardRegistryMetrics {
    pub shard_count: usize,
    pub healthy_shard_count: usize,
    pub total_load: u64,
    pub total_access_count: u64,
    pub total_cache_hits: u64,
    pub total_cache_misses: u64,
    pub average_latency_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ShardRegistry {
    shards: BTreeMap<String, ShardRecord>,
    owners: BTreeMap<String, BTreeSet<String>>,
}

impl ShardRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_shard(&mut self, shard: ShardRecord) {
        self.owners
            .entry(shard.metadata.owner_node_id.clone())
            .or_default()
            .insert(shard.metadata.shard_id.clone());
        self.shards.insert(shard.metadata.shard_id.clone(), shard);
    }

    #[must_use]
    pub fn get(&self, shard_id: &str) -> Option<&ShardRecord> {
        self.shards.get(shard_id)
    }

    #[must_use]
    pub fn all(&self) -> Vec<&ShardRecord> {
        self.shards.values().collect()
    }

    #[must_use]
    pub fn shards_for_owner(&self, node_id: &str) -> Vec<&ShardRecord> {
        let ids = self.owners.get(node_id).cloned().unwrap_or_default();
        ids.into_iter().filter_map(|id| self.shards.get(&id)).collect()
    }

    #[must_use]
    pub fn route_by_hash(&self, key: &str) -> Option<&ShardRecord> {
        if self.shards.is_empty() {
            return None;
        }
        let mut ids = self.shards.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        let idx = stable_hash_u64(key.as_bytes()) as usize % ids.len();
        self.shards.get(&ids[idx])
    }

    #[must_use]
    pub fn route_static_fallback(&self) -> Option<&ShardRecord> {
        self.shards.values().next()
    }

    #[must_use]
    pub fn lookup_by_dataset(&self, dataset_id: &str) -> Vec<&ShardRecord> {
        self.shards
            .values()
            .filter(|record| record.metadata.dataset_id == dataset_id)
            .collect()
    }

    pub fn assign_round_robin(
        &mut self,
        dataset_id: &str,
        shard_count: usize,
        node_ids: &[String],
    ) -> Vec<String> {
        let mut assigned = Vec::new();
        if node_ids.is_empty() || shard_count == 0 {
            return assigned;
        }
        for idx in 0..shard_count {
            let owner = node_ids[idx % node_ids.len()].clone();
            let shard_id = format!("{dataset_id}-s{:03}", idx + 1);
            let record = ShardRecord {
                metadata: ShardMetadata {
                    shard_id: shard_id.clone(),
                    dataset_id: dataset_id.to_string(),
                    key_range_start: format!("k{:03}", idx * 100),
                    key_range_end: format!("k{:03}", idx * 100 + 99),
                    owner_node_id: owner,
                    replica_node_ids: Vec::new(),
                },
                health: ShardHealth {
                    healthy: true,
                    open_errors: 0,
                },
                stats: ShardRuntimeStats {
                    load: 0,
                    access_count: 0,
                    cache_hits: 0,
                    cache_misses: 0,
                    latency_sum_ms: 0,
                    latency_samples: 0,
                },
            };
            self.upsert_shard(record);
            assigned.push(shard_id);
        }
        assigned
    }

    pub fn transfer_ownership(&mut self, shard_id: &str, new_owner_node_id: &str) -> bool {
        let Some(record) = self.shards.get_mut(shard_id) else {
            return false;
        };
        let old_owner = record.metadata.owner_node_id.clone();
        record.metadata.owner_node_id = new_owner_node_id.to_string();
        if let Some(set) = self.owners.get_mut(&old_owner) {
            set.remove(shard_id);
        }
        self.owners
            .entry(new_owner_node_id.to_string())
            .or_default()
            .insert(shard_id.to_string());
        true
    }

    pub fn relocate_shard(&mut self, shard_id: &str, target_owner_node_id: &str) -> bool {
        self.transfer_ownership(shard_id, target_owner_node_id)
    }

    pub fn rebalance(&mut self, node_ids: &[String]) {
        if node_ids.is_empty() || self.shards.is_empty() {
            return;
        }
        let mut ids = self.shards.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        for (idx, shard_id) in ids.iter().enumerate() {
            let owner = &node_ids[idx % node_ids.len()];
            let _ = self.transfer_ownership(shard_id, owner);
        }
    }

    pub fn record_access(&mut self, shard_id: &str, latency_ms: u64, cache_hit: bool) {
        if let Some(record) = self.shards.get_mut(shard_id) {
            record.stats.access_count = record.stats.access_count.saturating_add(1);
            record.stats.load = record.stats.load.saturating_add(1);
            record.stats.latency_sum_ms = record.stats.latency_sum_ms.saturating_add(latency_ms);
            record.stats.latency_samples = record.stats.latency_samples.saturating_add(1);
            if cache_hit {
                record.stats.cache_hits = record.stats.cache_hits.saturating_add(1);
            } else {
                record.stats.cache_misses = record.stats.cache_misses.saturating_add(1);
            }
        }
    }

    pub fn mark_unhealthy(&mut self, shard_id: &str) {
        if let Some(record) = self.shards.get_mut(shard_id) {
            record.health.healthy = false;
            record.health.open_errors = record.health.open_errors.saturating_add(1);
        }
    }

    #[must_use]
    pub fn metrics(&self) -> ShardRegistryMetrics {
        let shard_count = self.shards.len();
        let mut healthy_shard_count = 0usize;
        let mut total_load = 0u64;
        let mut total_access_count = 0u64;
        let mut total_cache_hits = 0u64;
        let mut total_cache_misses = 0u64;
        let mut latency_sum_ms = 0u64;
        let mut latency_samples = 0u64;

        for shard in self.shards.values() {
            if shard.health.healthy {
                healthy_shard_count += 1;
            }
            total_load = total_load.saturating_add(shard.stats.load);
            total_access_count = total_access_count.saturating_add(shard.stats.access_count);
            total_cache_hits = total_cache_hits.saturating_add(shard.stats.cache_hits);
            total_cache_misses = total_cache_misses.saturating_add(shard.stats.cache_misses);
            latency_sum_ms = latency_sum_ms.saturating_add(shard.stats.latency_sum_ms);
            latency_samples = latency_samples.saturating_add(shard.stats.latency_samples);
        }
        let average_latency_ms = if latency_samples == 0 {
            0
        } else {
            latency_sum_ms / latency_samples
        };

        ShardRegistryMetrics {
            shard_count,
            healthy_shard_count,
            total_load,
            total_access_count,
            total_cache_hits,
            total_cache_misses,
            average_latency_ms,
        }
    }
}

#[must_use]
pub fn stable_hash_u64(input: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in input {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::ShardRegistry;

    #[test]
    fn assignment_and_lookup_cover_dataset_shards() {
        let mut registry = ShardRegistry::new();
        let owners = vec!["node-a".to_string(), "node-b".to_string()];
        let assigned = registry.assign_round_robin("ds1", 4, &owners);
        assert_eq!(assigned.len(), 4);
        assert_eq!(registry.lookup_by_dataset("ds1").len(), 4);
        assert!(registry.route_by_hash("chr1:100-200").is_some());
    }

    #[test]
    fn rebalance_and_relocation_update_ownership() {
        let mut registry = ShardRegistry::new();
        let owners = vec!["node-a".to_string(), "node-b".to_string()];
        let assigned = registry.assign_round_robin("ds2", 2, &owners);
        let shard = assigned[0].clone();
        assert!(registry.transfer_ownership(&shard, "node-z"));
        assert_eq!(registry.get(&shard).expect("shard").metadata.owner_node_id, "node-z");
        assert!(registry.relocate_shard(&shard, "node-y"));
        assert_eq!(registry.get(&shard).expect("shard").metadata.owner_node_id, "node-y");
        registry.rebalance(&owners);
        assert!(owners
            .iter()
            .any(|owner| registry.get(&shard).expect("shard").metadata.owner_node_id == *owner));
    }

    #[test]
    fn shard_metrics_include_health_load_access_cache_and_latency() {
        let mut registry = ShardRegistry::new();
        let owners = vec!["node-a".to_string()];
        let assigned = registry.assign_round_robin("ds3", 1, &owners);
        let shard = assigned[0].clone();
        registry.record_access(&shard, 25, true);
        registry.record_access(&shard, 35, false);
        registry.mark_unhealthy(&shard);

        let metrics = registry.metrics();
        assert_eq!(metrics.shard_count, 1);
        assert_eq!(metrics.healthy_shard_count, 0);
        assert_eq!(metrics.total_load, 2);
        assert_eq!(metrics.total_access_count, 2);
        assert_eq!(metrics.total_cache_hits, 1);
        assert_eq!(metrics.total_cache_misses, 1);
        assert_eq!(metrics.average_latency_ms, 30);
    }
}
