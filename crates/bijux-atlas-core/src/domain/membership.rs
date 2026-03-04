// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::distributed::{NodeDescriptor, NodeIdentity};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipState {
    Joining,
    Active,
    Quarantined,
    Maintenance,
    Draining,
    Recovering,
    TimedOut,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipPolicy {
    pub heartbeat_interval_ms: u64,
    pub node_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub identity: NodeIdentity,
    pub sent_at_unix_ms: u64,
    pub load_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMembershipRecord {
    pub descriptor: NodeDescriptor,
    pub state: MembershipState,
    pub registered_at_unix_ms: u64,
    pub last_heartbeat_unix_ms: u64,
    pub load_percent: u8,
    pub restart_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipMetrics {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub timed_out_nodes: usize,
    pub quarantined_nodes: usize,
    pub maintenance_nodes: usize,
    pub draining_nodes: usize,
    pub average_load_percent: u8,
}

#[derive(Debug, Clone)]
pub struct MembershipRegistry {
    policy: MembershipPolicy,
    nodes: BTreeMap<String, NodeMembershipRecord>,
}

impl MembershipRegistry {
    #[must_use]
    pub fn new(policy: MembershipPolicy) -> Self {
        Self {
            policy,
            nodes: BTreeMap::new(),
        }
    }

    pub fn join_node(&mut self, descriptor: NodeDescriptor, now_unix_ms: u64) {
        let node_id = descriptor.identity.node_id.clone();
        self.nodes.insert(
            node_id,
            NodeMembershipRecord {
                descriptor,
                state: MembershipState::Joining,
                registered_at_unix_ms: now_unix_ms,
                last_heartbeat_unix_ms: now_unix_ms,
                load_percent: 0,
                restart_count: 0,
            },
        );
    }

    pub fn activate_node(&mut self, node_id: &str) {
        if let Some(record) = self.nodes.get_mut(node_id) {
            record.state = MembershipState::Active;
        }
    }

    pub fn apply_heartbeat(&mut self, message: HeartbeatMessage) {
        if let Some(record) = self.nodes.get_mut(&message.identity.node_id) {
            if record.descriptor.identity.generation == message.identity.generation {
                record.last_heartbeat_unix_ms = message.sent_at_unix_ms;
                record.load_percent = message.load_percent.min(100);
                if matches!(
                    record.state,
                    MembershipState::Joining | MembershipState::Recovering
                ) {
                    record.state = MembershipState::Active;
                }
            }
        }
    }

    #[must_use]
    pub fn node_is_live(&self, node_id: &str, now_unix_ms: u64) -> bool {
        let Some(record) = self.nodes.get(node_id) else {
            return false;
        };
        if !matches!(
            record.state,
            MembershipState::Joining
                | MembershipState::Active
                | MembershipState::Maintenance
                | MembershipState::Draining
                | MembershipState::Recovering
        ) {
            return false;
        }
        now_unix_ms.saturating_sub(record.last_heartbeat_unix_ms) <= self.policy.node_timeout_ms
    }

    pub fn detect_timeouts(&mut self, now_unix_ms: u64) -> Vec<String> {
        let mut timed_out = Vec::new();
        for (node_id, record) in &mut self.nodes {
            if matches!(
                record.state,
                MembershipState::Removed | MembershipState::Quarantined
            ) {
                continue;
            }
            if now_unix_ms.saturating_sub(record.last_heartbeat_unix_ms)
                > self.policy.node_timeout_ms
            {
                record.state = MembershipState::TimedOut;
                timed_out.push(node_id.clone());
            }
        }
        timed_out
    }

    pub fn remove_node(&mut self, node_id: &str) {
        if let Some(record) = self.nodes.get_mut(node_id) {
            record.state = MembershipState::Removed;
        }
    }

    pub fn set_quarantine(&mut self, node_id: &str) {
        if let Some(record) = self.nodes.get_mut(node_id) {
            record.state = MembershipState::Quarantined;
        }
    }

    pub fn set_maintenance(&mut self, node_id: &str) {
        if let Some(record) = self.nodes.get_mut(node_id) {
            record.state = MembershipState::Maintenance;
        }
    }

    pub fn set_draining(&mut self, node_id: &str) {
        if let Some(record) = self.nodes.get_mut(node_id) {
            record.state = MembershipState::Draining;
        }
    }

    pub fn handle_restart(&mut self, node_id: &str, new_generation: u64, now_unix_ms: u64) {
        if let Some(record) = self.nodes.get_mut(node_id) {
            record.descriptor.identity.generation = new_generation.max(1);
            record.state = MembershipState::Joining;
            record.last_heartbeat_unix_ms = now_unix_ms;
            record.restart_count = record.restart_count.saturating_add(1);
        }
    }

    pub fn recover_node(&mut self, node_id: &str, now_unix_ms: u64) {
        if let Some(record) = self.nodes.get_mut(node_id) {
            record.state = MembershipState::Recovering;
            record.last_heartbeat_unix_ms = now_unix_ms;
        }
    }

    #[must_use]
    pub fn nodes(&self) -> Vec<&NodeMembershipRecord> {
        self.nodes.values().collect()
    }

    #[must_use]
    pub fn metrics(&self) -> MembershipMetrics {
        let total_nodes = self.nodes.len();
        let mut active_nodes = 0usize;
        let mut timed_out_nodes = 0usize;
        let mut quarantined_nodes = 0usize;
        let mut maintenance_nodes = 0usize;
        let mut draining_nodes = 0usize;
        let mut load_sum = 0usize;

        for node in self.nodes.values() {
            load_sum += usize::from(node.load_percent);
            match node.state {
                MembershipState::Active => active_nodes += 1,
                MembershipState::TimedOut => timed_out_nodes += 1,
                MembershipState::Quarantined => quarantined_nodes += 1,
                MembershipState::Maintenance => maintenance_nodes += 1,
                MembershipState::Draining => draining_nodes += 1,
                _ => {}
            }
        }

        let average_load_percent = if total_nodes == 0 {
            0
        } else {
            (load_sum / total_nodes) as u8
        };

        MembershipMetrics {
            total_nodes,
            active_nodes,
            timed_out_nodes,
            quarantined_nodes,
            maintenance_nodes,
            draining_nodes,
            average_load_percent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HeartbeatMessage, MembershipPolicy, MembershipRegistry, MembershipState};
    use crate::domain::distributed::{
        NodeDescriptor, NodeIdentity, NodeRole, ReadinessPolicy, ShutdownPolicy,
    };

    fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            identity: NodeIdentity {
                cluster_id: "atlas-test".to_string(),
                node_id: "node-1".to_string(),
                generation: 1,
            },
            role: NodeRole::Hybrid,
            advertise_addr: "http://node-1:8080".to_string(),
            capabilities: vec!["query.execute".to_string(), "ingest.pipeline".to_string()],
            readiness: ReadinessPolicy {
                require_membership: true,
                require_dataset_registry: true,
                require_health_probes: true,
            },
            shutdown: ShutdownPolicy {
                drain_timeout_ms: 1_000,
                publish_exit_state: true,
            },
        }
    }

    #[test]
    fn membership_timeout_detection_marks_timed_out_nodes() {
        let mut registry = MembershipRegistry::new(MembershipPolicy {
            heartbeat_interval_ms: 1_000,
            node_timeout_ms: 5_000,
        });
        registry.join_node(descriptor(), 1_000);
        registry.activate_node("node-1");
        let timed_out = registry.detect_timeouts(7_000);
        assert_eq!(timed_out, vec!["node-1".to_string()]);
        assert_eq!(registry.nodes()[0].state, MembershipState::TimedOut);
    }

    #[test]
    fn membership_rejoin_via_recover_and_heartbeat_restores_active_state() {
        let mut registry = MembershipRegistry::new(MembershipPolicy {
            heartbeat_interval_ms: 1_000,
            node_timeout_ms: 5_000,
        });
        registry.join_node(descriptor(), 1_000);
        registry.activate_node("node-1");
        registry.detect_timeouts(8_000);
        registry.recover_node("node-1", 8_500);
        registry.apply_heartbeat(HeartbeatMessage {
            identity: NodeIdentity {
                cluster_id: "atlas-test".to_string(),
                node_id: "node-1".to_string(),
                generation: 1,
            },
            sent_at_unix_ms: 9_000,
            load_percent: 42,
        });
        assert_eq!(registry.nodes()[0].state, MembershipState::Active);
    }

    #[test]
    fn membership_restart_updates_generation_and_counts_restart() {
        let mut registry = MembershipRegistry::new(MembershipPolicy {
            heartbeat_interval_ms: 1_000,
            node_timeout_ms: 5_000,
        });
        registry.join_node(descriptor(), 1_000);
        registry.activate_node("node-1");
        registry.handle_restart("node-1", 2, 2_000);
        let node = registry.nodes()[0];
        assert_eq!(node.descriptor.identity.generation, 2);
        assert_eq!(node.restart_count, 1);
        assert_eq!(node.state, MembershipState::Joining);
    }

    #[test]
    fn membership_modes_support_quarantine_maintenance_and_draining() {
        let mut registry = MembershipRegistry::new(MembershipPolicy {
            heartbeat_interval_ms: 1_000,
            node_timeout_ms: 5_000,
        });
        registry.join_node(descriptor(), 1_000);
        registry.set_quarantine("node-1");
        assert_eq!(registry.nodes()[0].state, MembershipState::Quarantined);
        registry.set_maintenance("node-1");
        assert_eq!(registry.nodes()[0].state, MembershipState::Maintenance);
        registry.set_draining("node-1");
        assert_eq!(registry.nodes()[0].state, MembershipState::Draining);
    }

    #[test]
    fn membership_metrics_include_load_and_capability_carrier_node() {
        let mut registry = MembershipRegistry::new(MembershipPolicy {
            heartbeat_interval_ms: 1_000,
            node_timeout_ms: 5_000,
        });
        registry.join_node(descriptor(), 1_000);
        registry.activate_node("node-1");
        registry.apply_heartbeat(HeartbeatMessage {
            identity: NodeIdentity {
                cluster_id: "atlas-test".to_string(),
                node_id: "node-1".to_string(),
                generation: 1,
            },
            sent_at_unix_ms: 1_500,
            load_percent: 70,
        });

        let metrics = registry.metrics();
        assert_eq!(metrics.total_nodes, 1);
        assert_eq!(metrics.active_nodes, 1);
        assert_eq!(metrics.average_load_percent, 70);
        assert!(registry.nodes()[0]
            .descriptor
            .capabilities
            .iter()
            .any(|capability| capability == "query.execute"));
    }
}
