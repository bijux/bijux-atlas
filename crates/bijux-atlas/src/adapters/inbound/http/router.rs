// SPDX-License-Identifier: Apache-2.0

use crate::app::server::cache;
use crate::app::server::state::{
    AppState, DatasetCacheManager, RequestMetrics,
};
use crate::adapters::inbound::http::request_policies::{
    cors_middleware, provenance_headers_middleware, resilience_middleware, security_middleware,
};
use crate::adapters::outbound::redis::RedisBackend;
use crate::adapters::outbound::telemetry::rate_limiter::RateLimiter;
use crate::runtime::config::ApiConfig;
use crate::domain::cluster::config::load_cluster_config_from_path;
use crate::domain::canonical;
use crate::domain::cluster::membership::{MembershipPolicy, MembershipRegistry};
use crate::domain::cluster::replication::{
    ConsistencyGuarantee, ConsistencyLevel, ReplicaRegistry, ReplicationPolicy,
};
use crate::domain::cluster::resilience::{
    FailureDetectionPolicy, FailureRecoveryRegistry, RecoveryPolicy, ResilienceGuarantees,
};
use crate::domain::cluster::sharding::ShardRegistry;
use crate::domain::sha256_hex;
use crate::adapters::inbound::http;
use axum::extract::DefaultBodyLimit;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;
use bijux_atlas::domain::query::QueryLimits;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};

impl AppState {
    fn init_request_metrics() -> Arc<RequestMetrics> {
        Arc::new(RequestMetrics::default())
    }

    fn derive_runtime_policy_hash(api: &ApiConfig, limits: &QueryLimits) -> String {
        let payload = serde_json::json!({
            "api": api,
            "limits": limits
        });
        match canonical::stable_json_bytes(&payload) {
            Ok(bytes) => sha256_hex(&bytes),
            Err(_) => sha256_hex(b"runtime-policy-hash-fallback"),
        }
    }

    fn init_membership_registry() -> MembershipRegistry {
        let policy = std::env::var("ATLAS_CLUSTER_CONFIG_PATH")
            .ok()
            .and_then(|cluster_path| {
                load_cluster_config_from_path(std::path::Path::new(&cluster_path))
                    .ok()
                    .map(|cfg| MembershipPolicy {
                        heartbeat_interval_ms: cfg.health.heartbeat_interval_ms,
                        node_timeout_ms: cfg.health.node_timeout_ms,
                    })
            })
            .unwrap_or(MembershipPolicy {
                heartbeat_interval_ms: 1_000,
                node_timeout_ms: 5_000,
            });
        MembershipRegistry::new(policy)
    }

    fn init_shard_registry() -> ShardRegistry {
        ShardRegistry::new()
    }

    fn init_replica_registry() -> ReplicaRegistry {
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

    fn init_resilience_registry() -> FailureRecoveryRegistry {
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

    #[must_use]
    pub fn new(cache: Arc<DatasetCacheManager>) -> Self {
        Self::with_config(cache, ApiConfig::default(), QueryLimits::default())
    }

    #[must_use]
    pub fn with_config(
        cache: Arc<DatasetCacheManager>,
        api: ApiConfig,
        limits: QueryLimits,
    ) -> Self {
        let runtime_policy_hash = Arc::new(Self::derive_runtime_policy_hash(&api, &limits));
        let redis_policy = crate::adapters::outbound::redis::RedisPolicy {
            timeout: Duration::from_millis(api.redis_timeout_ms),
            retry_attempts: api.redis_retry_attempts.max(1),
            breaker_failure_threshold: api.redis_breaker_failure_threshold,
            breaker_open_duration: Duration::from_millis(api.redis_breaker_open_ms),
            max_key_bytes: api.redis_cache_max_key_bytes,
            max_cardinality: api.redis_cache_max_cardinality,
            max_ttl_secs: api.redis_cache_ttl_max_secs,
        };
        Self {
            cache,
            ready: Arc::new(AtomicBool::new(true)),
            class_cheap: Arc::new(Semaphore::new(api.concurrency_cheap)),
            class_medium: Arc::new(Semaphore::new(api.concurrency_medium)),
            class_heavy: Arc::new(Semaphore::new(api.concurrency_heavy)),
            heavy_workers: Arc::new(Semaphore::new(api.heavy_worker_pool_size)),
            ip_limiter: Arc::new(RateLimiter::new(
                if api.enable_redis_rate_limit {
                    api.redis_url.as_deref().and_then(|u| {
                        RedisBackend::new(u, &api.redis_prefix, redis_policy.clone()).ok()
                    })
                } else {
                    None
                },
                "ip",
            )),
            sequence_ip_limiter: Arc::new(RateLimiter::new(
                if api.enable_redis_rate_limit {
                    api.redis_url.as_deref().and_then(|u| {
                        RedisBackend::new(u, &api.redis_prefix, redis_policy.clone()).ok()
                    })
                } else {
                    None
                },
                "sequence_ip",
            )),
            api_key_limiter: Arc::new(RateLimiter::new(
                if api.enable_redis_rate_limit {
                    api.redis_url.as_deref().and_then(|u| {
                        RedisBackend::new(u, &api.redis_prefix, redis_policy.clone()).ok()
                    })
                } else {
                    None
                },
                "api_key",
            )),
            metrics: Self::init_request_metrics(),
            request_id_seed: Arc::new(AtomicU64::new(1)),
            accepting_requests: Arc::new(AtomicBool::new(true)),
            coalescer: Arc::new(cache::coalesce::QueryCoalescer::new()),
            hot_query_cache: Arc::new(Mutex::new(cache::hot::HotQueryCache::new(
                Duration::from_secs(2),
                512,
            ))),
            redis_backend: api
                .redis_url
                .as_deref()
                .and_then(|u| RedisBackend::new(u, &api.redis_prefix, redis_policy).ok())
                .map(Arc::new),
            queued_requests: Arc::new(AtomicU64::new(0)),
            membership: Arc::new(Mutex::new(Self::init_membership_registry())),
            shard_registry: Arc::new(Mutex::new(Self::init_shard_registry())),
            replica_registry: Arc::new(Mutex::new(Self::init_replica_registry())),
            resilience_registry: Arc::new(Mutex::new(Self::init_resilience_registry())),
            runtime_policy_hash,
            runtime_policy_mode: Arc::new(crate::runtime::config::default_runtime_policy_mode()),
            api,
            limits,
        }
    }

    pub fn begin_shutdown_drain_heavy(&self) {
        self.class_heavy.close();
        self.heavy_workers.close();
    }
}

pub fn build_router(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/", get(http::handlers::landing_handler))
        .route("/health", get(http::handlers::health_handler))
        .route("/healthz", get(http::handlers::healthz_handler))
        .route(
            "/healthz/overload",
            get(http::handlers::overload_health_handler),
        )
        .route("/ready", get(http::handlers::ready_handler))
        .route("/readyz", get(http::handlers::readyz_handler))
        .route("/live", get(http::handlers::live_handler))
        .route("/metrics", get(http::handlers::metrics_handler))
        .route("/v1/openapi.json", get(http::handlers::openapi_handler))
        .route("/v1/version", get(http::handlers::version_handler))
        .route("/v1/datasets", get(http::handlers::datasets_handler))
        .route(
            "/v1/datasets/:release/:species/:assembly",
            get(http::handlers::dataset_identity_handler),
        )
        .route(
            "/v1/releases/:release/species/:species/assemblies/:assembly",
            get(http::handlers::release_dataset_handler),
        )
        .route("/v1/genes", get(http::handlers::genes_handler))
        .route(
            "/v1/query/validate",
            post(http::handlers::query_validate_handler),
        )
        .route("/v1/genes/count", get(http::handlers::genes_count_handler))
        .route("/v1/diff/genes", get(http::diff::diff_genes_handler))
        .route("/v1/diff/region", get(http::diff::diff_region_handler))
        .route(
            "/v1/sequence/region",
            get(http::sequence::sequence_region_handler),
        )
        .route(
            "/v1/genes/:gene_id/sequence",
            get(http::sequence::gene_sequence_handler),
        )
        .route(
            "/v1/genes/:gene_id/transcripts",
            get(http::handlers::gene_transcripts_handler),
        )
        .route(
            "/v1/transcripts/:tx_id",
            get(http::handlers::transcript_summary_handler),
        );
    if state.api.enable_admin_endpoints {
        router = router
            .route(
                "/debug/datasets",
                get(http::handlers::debug_datasets_handler),
            )
            .route(
                "/debug/dataset-health",
                get(http::handlers::dataset_health_handler),
            )
            .route(
                "/debug/registry-health",
                get(http::handlers::registry_health_handler),
            )
            .route(
                "/debug/diagnostics",
                get(http::handlers::diagnostics_handler),
            )
            .route(
                "/debug/runtime-stats",
                get(http::handlers::runtime_stats_handler),
            )
            .route(
                "/debug/system-info",
                get(http::handlers::system_info_handler),
            )
            .route(
                "/debug/build-metadata",
                get(http::handlers::build_metadata_handler),
            )
            .route(
                "/debug/runtime-config",
                get(http::handlers::runtime_config_dump_handler),
            )
            .route(
                "/debug/dataset-registry",
                get(http::handlers::dataset_registry_dump_handler),
            )
            .route(
                "/debug/shard-map",
                get(http::handlers::shard_map_dump_handler),
            )
            .route(
                "/debug/query-planner-stats",
                get(http::handlers::query_planner_stats_dump_handler),
            )
            .route(
                "/debug/cache-stats",
                get(http::handlers::cache_stats_dump_handler),
            )
            .route(
                "/debug/cluster/nodes",
                get(http::handlers::cluster_nodes_handler),
            )
            .route(
                "/debug/cluster-status",
                get(http::handlers::cluster_status_handler),
            )
            .route(
                "/debug/cluster/register",
                post(http::handlers::cluster_register_handler),
            )
            .route(
                "/debug/cluster/heartbeat",
                post(http::handlers::cluster_heartbeat_handler),
            )
            .route(
                "/debug/cluster/mode",
                post(http::handlers::cluster_mode_handler),
            )
            .route(
                "/debug/cluster/replicas",
                get(http::handlers::cluster_replica_list_handler),
            )
            .route(
                "/debug/cluster/replicas/health",
                get(http::handlers::cluster_replica_health_handler),
            )
            .route(
                "/debug/cluster/replicas/failover",
                post(http::handlers::cluster_replica_failover_handler),
            )
            .route(
                "/debug/cluster/replicas/diagnostics",
                get(http::handlers::cluster_replica_diagnostics_handler),
            )
            .route(
                "/debug/recovery/run",
                post(http::handlers::cluster_recovery_run_handler),
            )
            .route(
                "/debug/recovery/diagnostics",
                get(http::handlers::recovery_diagnostics_handler),
            )
            .route(
                "/debug/failure-injection",
                post(http::handlers::failure_injection_handler),
            )
            .route("/debug/chaos/run", post(http::handlers::chaos_run_handler))
            .route("/v1/_debug/echo", get(http::handlers::debug_echo_handler));
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
        .layer(DefaultBodyLimit::max(state.api.max_body_bytes))
        .with_state(state)
}


#[cfg(test)]
mod bulkhead_tests {
    use super::*;
    use crate::adapters::outbound::store::testing::FakeStore;
    use crate::app::server::state::DatasetCacheConfig;

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

        let heavy = state.class_heavy.clone().try_acquire_owned();
        assert!(heavy.is_err(), "heavy permits must be closed during drain");

        let worker = state.heavy_workers.clone().try_acquire_owned();
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

        let shard_count = state.shard_registry.lock().await.metrics().shard_count;
        let replica_groups = state.replica_registry.lock().await.metrics().replica_groups_total;

        assert_eq!(shard_count, 0, "runtime must not fabricate shard ownership");
        assert_eq!(replica_groups, 0, "runtime must not fabricate replica groups");
    }

    #[tokio::test]
    async fn app_state_uses_runtime_policy_mode_default_owner() {
        let store = Arc::new(FakeStore::default());
        let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
        let state = AppState::with_config(cache, ApiConfig::default(), QueryLimits::default());

        assert_eq!(
            state.runtime_policy_mode.as_str(),
            crate::runtime::config::default_runtime_policy_mode()
        );
    }
}
