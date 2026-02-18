impl AppState {
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
        let redis_policy = telemetry::redis_backend::RedisPolicy {
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
            metrics: Arc::new(RequestMetrics::default()),
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
    Router::new()
        .route("/", get(http::handlers::landing_handler))
        .route("/healthz", get(http::handlers::healthz_handler))
        .route(
            "/healthz/overload",
            get(http::handlers::overload_health_handler),
        )
        .route("/readyz", get(http::handlers::readyz_handler))
        .route("/metrics", get(http::handlers::metrics_handler))
        .route("/v1/openapi.json", get(http::handlers::openapi_handler))
        .route("/v1/version", get(http::handlers::version_handler))
        .route("/v1/datasets", get(http::handlers::datasets_handler))
        .route(
            "/v1/releases/:release/species/:species/assemblies/:assembly",
            get(http::handlers::release_dataset_handler),
        )
        .route("/v1/genes", get(http::handlers::genes_handler))
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
        )
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
        .layer(from_fn_with_state(state.clone(), cors_middleware))
        .layer(from_fn_with_state(state.clone(), security_middleware))
        .layer(from_fn_with_state(state.clone(), resilience_middleware))
        .layer(from_fn(provenance_headers_middleware))
        .layer(DefaultBodyLimit::max(state.api.max_body_bytes))
        .with_state(state)
}

pub use store::fake::FakeStore;

#[cfg(test)]
mod bulkhead_tests {
    use super::*;

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
}
