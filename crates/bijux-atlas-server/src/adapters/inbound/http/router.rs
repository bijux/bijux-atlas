// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http;
use crate::adapters::inbound::http::request_policies::{
    cors_middleware, debug_route_hardening_middleware, provenance_headers_middleware,
    resilience_middleware, security_middleware,
};
use crate::adapters::inbound::http::{
    catalog, diagnostic_routes, gene_routes, service_routes, transcript_routes,
};
use crate::app::server::AppState;
use axum::extract::DefaultBodyLimit;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;

pub fn build_router(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/", get(service_routes::landing_handler))
        .route("/health", get(service_routes::health_handler))
        .route("/healthz", get(service_routes::healthz_handler))
        .route(
            "/healthz/overload",
            get(service_routes::overload_health_handler),
        )
        .route("/ready", get(service_routes::ready_handler))
        .route("/readyz", get(service_routes::readyz_handler))
        .route("/live", get(service_routes::live_handler))
        .route("/metrics", get(catalog::metrics_handler))
        .route("/v1/openapi.json", get(service_routes::openapi_handler))
        .route("/v1/version", get(service_routes::version_handler))
        .route("/v1/datasets", get(catalog::datasets_handler))
        .route(
            "/v1/datasets/{release}/{species}/{assembly}",
            get(catalog::dataset_identity_handler),
        )
        .route(
            "/v1/releases/{release}/species/{species}/assemblies/{assembly}",
            get(catalog::release_dataset_handler),
        )
        .route("/v1/genes", get(gene_routes::genes_handler))
        .route(
            "/v1/query/validate",
            post(diagnostic_routes::query_validate_handler),
        )
        .route("/v1/genes/count", get(gene_routes::genes_count_handler))
        .route("/v1/diff/genes", get(http::diff::diff_genes_handler))
        .route("/v1/diff/region", get(http::diff::diff_region_handler))
        .route(
            "/v1/sequence/region",
            get(http::sequence::sequence_region_handler),
        )
        .route(
            "/v1/genes/{gene_id}/sequence",
            get(http::sequence::gene_sequence_handler),
        )
        .route(
            "/v1/genes/{gene_id}/transcripts",
            get(transcript_routes::gene_transcripts_handler),
        )
        .route(
            "/v1/transcripts/{tx_id}",
            get(transcript_routes::transcript_summary_handler),
        );
    if state.api.enable_admin_endpoints {
        router = router
            .route(
                "/debug/datasets",
                get(diagnostic_routes::debug_datasets_handler),
            )
            .route(
                "/debug/dataset-health",
                get(diagnostic_routes::dataset_health_handler),
            )
            .route(
                "/debug/registry-health",
                get(diagnostic_routes::registry_health_handler),
            )
            .route(
                "/debug/diagnostics",
                get(diagnostic_routes::diagnostics_handler),
            )
            .route(
                "/debug/runtime-stats",
                get(diagnostic_routes::runtime_stats_handler),
            )
            .route(
                "/debug/system-info",
                get(diagnostic_routes::system_info_handler),
            )
            .route(
                "/debug/build-metadata",
                get(diagnostic_routes::build_metadata_handler),
            )
            .route(
                "/debug/runtime-config",
                get(diagnostic_routes::runtime_config_dump_handler),
            )
            .route(
                "/debug/dataset-registry",
                get(diagnostic_routes::dataset_registry_dump_handler),
            )
            .route(
                "/debug/shard-map",
                get(diagnostic_routes::shard_map_dump_handler),
            )
            .route(
                "/debug/query-planner-stats",
                get(diagnostic_routes::query_planner_stats_dump_handler),
            )
            .route(
                "/debug/cache-stats",
                get(diagnostic_routes::cache_stats_dump_handler),
            )
            .route(
                "/debug/cluster/nodes",
                get(service_routes::cluster_nodes_handler),
            )
            .route(
                "/debug/cluster-status",
                get(service_routes::cluster_status_handler),
            )
            .route(
                "/debug/cluster/register",
                post(service_routes::cluster_register_handler),
            )
            .route(
                "/debug/cluster/heartbeat",
                post(service_routes::cluster_heartbeat_handler),
            )
            .route(
                "/debug/cluster/mode",
                post(service_routes::cluster_mode_handler),
            )
            .route(
                "/debug/cluster/replicas",
                get(service_routes::cluster_replica_list_handler),
            )
            .route(
                "/debug/cluster/replicas/health",
                get(service_routes::cluster_replica_health_handler),
            )
            .route(
                "/debug/cluster/replicas/failover",
                post(service_routes::cluster_replica_failover_handler),
            )
            .route(
                "/debug/cluster/replicas/diagnostics",
                get(service_routes::cluster_replica_diagnostics_handler),
            )
            .route(
                "/debug/recovery/run",
                post(service_routes::cluster_recovery_run_handler),
            )
            .route(
                "/debug/recovery/diagnostics",
                get(service_routes::recovery_diagnostics_handler),
            )
            .route(
                "/debug/failure-injection",
                post(service_routes::failure_injection_handler),
            )
            .route("/debug/chaos/run", post(service_routes::chaos_run_handler))
            .route(
                "/v1/_debug/echo",
                get(diagnostic_routes::debug_echo_handler),
            );
    }
    router
        .layer(from_fn_with_state(
            state.clone(),
            crate::adapters::inbound::http::middleware::request_tracing::request_tracing_middleware,
        ))
        .layer(from_fn_with_state(state.clone(), cors_middleware))
        .layer(from_fn_with_state(state.clone(), security_middleware))
        .layer(from_fn_with_state(state.clone(), resilience_middleware))
        .layer(from_fn_with_state(
            state.clone(),
            provenance_headers_middleware,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            debug_route_hardening_middleware,
        ))
        .layer(DefaultBodyLimit::max(state.api.max_body_bytes))
        .layer(from_fn_with_state(
            state.clone(),
            crate::adapters::inbound::http::middleware::error_envelope::error_envelope_middleware,
        ))
        .with_state(state)
}

#[cfg(test)]
mod bulkhead_tests {
    use super::*;
    use crate::app::server::{DatasetCacheConfig, DatasetCacheManager};
    use bijux_atlas_query::{QueryClass, QueryLimits};
    use bijux_atlas_runtime::adapters::outbound::store::FakeStore;
    use bijux_atlas_runtime::runtime::config::ApiConfig;
    use std::sync::Arc;

    #[tokio::test]
    async fn heavy_bulkhead_saturation_does_not_block_cheap_permits() {
        let store = Arc::new(FakeStore::default());
        let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
        let api = ApiConfig {
            concurrency_cheap: 2,
            concurrency_heavy: 1,
            ..ApiConfig::default()
        };
        let state = AppState::with_config(cache, api, QueryLimits::default());

        let heavy = state
            .class_heavy
            .clone()
            .try_acquire_owned()
            .expect("heavy permit");
        let cheap = state
            .class_cheap
            .clone()
            .try_acquire_owned()
            .expect("cheap should remain available");

        drop((heavy, cheap));
    }

    #[tokio::test]
    async fn shutdown_drain_closes_heavy_bulkheads_and_keeps_cheap_open() {
        let store = Arc::new(FakeStore::default());
        let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
        let api = ApiConfig {
            concurrency_cheap: 1,
            concurrency_heavy: 1,
            heavy_worker_pool_size: 1,
            ..ApiConfig::default()
        };
        let state = AppState::with_config(cache, api, QueryLimits::default());

        state.begin_shutdown_drain_heavy();

        let heavy = state.try_acquire_query_class_permit(QueryClass::Heavy);
        assert!(heavy.is_err(), "heavy permits must be closed during drain");

        let worker = state.try_acquire_heavy_worker_permit();
        assert!(
            worker.is_err(),
            "heavy worker permits must be closed during drain"
        );

        let cheap = state
            .class_cheap
            .clone()
            .try_acquire_owned()
            .expect("cheap should remain available while draining heavy");
        drop(cheap);
    }

    #[tokio::test]
    async fn app_state_boots_without_demo_shards_or_replicas() {
        let store = Arc::new(FakeStore::default());
        let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
        let state = AppState::with_config(cache, ApiConfig::default(), QueryLimits::default());

        let shard_count = state.shard_registry().lock().await.metrics().shard_count;
        let replica_groups = state
            .replica_registry()
            .lock()
            .await
            .metrics()
            .replica_groups_total;

        assert_eq!(shard_count, 0, "runtime must not fabricate shard ownership");
        assert_eq!(
            replica_groups, 0,
            "runtime must not fabricate replica groups"
        );
    }

    #[tokio::test]
    async fn app_state_uses_runtime_policy_mode_default_owner() {
        let store = Arc::new(FakeStore::default());
        let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
        let state = AppState::with_config(cache, ApiConfig::default(), QueryLimits::default());

        assert_eq!(
            state.runtime_policy_mode.as_str(),
            bijux_atlas_runtime::runtime::config::default_runtime_policy_mode()
        );
    }
}
