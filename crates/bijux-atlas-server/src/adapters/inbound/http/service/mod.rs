// SPDX-License-Identifier: Apache-2.0

mod cluster_membership;
mod cluster_replicas;
mod lifecycle;
mod metadata;
mod recovery_controls;

pub(crate) use cluster_membership::{
    cluster_heartbeat_handler, cluster_mode_handler, cluster_nodes_handler,
    cluster_register_handler, cluster_status_handler,
};
pub(crate) use cluster_replicas::{
    cluster_replica_diagnostics_handler, cluster_replica_failover_handler,
    cluster_replica_health_handler, cluster_replica_list_handler,
};
pub(crate) use lifecycle::{
    health_handler, healthz_handler, live_handler, overload_health_handler, ready_handler,
    readyz_handler,
};
pub(crate) use metadata::{landing_handler, openapi_handler, version_handler};
pub(crate) use recovery_controls::{
    chaos_run_handler, cluster_recovery_run_handler, failure_injection_handler,
    recovery_diagnostics_handler,
};
