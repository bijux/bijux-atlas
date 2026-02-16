#![forbid(unsafe_code)]

use bijux_atlas_server::{
    build_router, ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, LocalFsBackend,
    RetryPolicy, S3LikeBackend,
};
use opentelemetry::trace::TracerProvider as _;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .and_then(|v| match v.as_str() {
            "1" | "true" | "TRUE" | "yes" | "YES" => Some(true),
            "0" | "false" | "FALSE" | "no" | "NO" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_duration_ms(name: &str, default_ms: u64) -> Duration {
    Duration::from_millis(env_u64(name, default_ms))
}

fn env_dataset_list(name: &str) -> Vec<bijux_atlas_model::DatasetId> {
    env::var(name)
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| {
            let p: Vec<_> = s.trim().split('/').collect();
            if p.len() == 3 {
                bijux_atlas_model::DatasetId::new(p[0], p[1], p[2]).ok()
            } else {
                None
            }
        })
        .collect()
}

fn pod_jitter_ms(max_ms: u64) -> u64 {
    if max_ms == 0 {
        return 0;
    }
    let seed = env::var("HOSTNAME")
        .ok()
        .map(|s| {
            s.bytes()
                .fold(0_u64, |acc, b| acc.wrapping_mul(131).wrapping_add(b as u64))
        })
        .unwrap_or(1);
    seed % max_ms
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("register SIGTERM");
        let mut sigint = signal(SignalKind::interrupt()).expect("register SIGINT");
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let log_json = env_bool("ATLAS_LOG_JSON", true);
    if env_bool("ATLAS_OTEL_ENABLED", false) {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()
            .expect("otlp exporter");
        let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .build()
            .tracer("bijux-atlas-server");
        if log_json {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json())
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .init();
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .init();
        }
    } else if log_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    init_tracing();

    let bind_addr = env::var("ATLAS_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let store_root = PathBuf::from(
        env::var("ATLAS_STORE_ROOT").unwrap_or_else(|_| "artifacts/server-store".to_string()),
    );
    let cache_root = PathBuf::from(
        env::var("ATLAS_CACHE_ROOT").unwrap_or_else(|_| "artifacts/server-cache".to_string()),
    );

    let pinned: HashSet<_> = env::var("ATLAS_PINNED_DATASETS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| {
            let p: Vec<_> = s.split('/').collect();
            if p.len() == 3 {
                bijux_atlas_model::DatasetId::new(p[0], p[1], p[2]).ok()
            } else {
                None
            }
        })
        .collect();

    let cache_cfg = DatasetCacheConfig {
        disk_root: cache_root,
        max_disk_bytes: env_u64("ATLAS_MAX_DISK_BYTES", 8 * 1024 * 1024 * 1024),
        max_dataset_count: env_usize("ATLAS_MAX_DATASET_COUNT", 8),
        pinned_datasets: pinned,
        startup_warmup: env_dataset_list("ATLAS_STARTUP_WARMUP"),
        startup_warmup_limit: env_usize("ATLAS_STARTUP_WARMUP_LIMIT", 8),
        fail_readiness_on_missing_warmup: env_bool("ATLAS_FAIL_ON_WARMUP_ERROR", false),
        read_only_fs: env_bool("ATLAS_READ_ONLY_FS_MODE", false),
        cached_only_mode: env_bool("ATLAS_CACHED_ONLY_MODE", false),
        dataset_open_timeout: env_duration_ms("ATLAS_DATASET_OPEN_TIMEOUT_MS", 3000),
        store_breaker_failure_threshold: env_u64("ATLAS_STORE_BREAKER_FAILURE_THRESHOLD", 5) as u32,
        store_breaker_open_duration: env_duration_ms("ATLAS_STORE_BREAKER_OPEN_MS", 20_000),
        store_retry_budget: env_u64("ATLAS_STORE_RETRY_BUDGET", 20) as u32,
        max_concurrent_downloads: env_usize("ATLAS_MAX_CONCURRENT_DOWNLOADS", 3),
        integrity_reverify_interval: env_duration_ms("ATLAS_INTEGRITY_REVERIFY_MS", 300_000),
        sqlite_pragma_cache_kib: env_u64("ATLAS_SQLITE_CACHE_KIB", 32 * 1024) as i64,
        sqlite_pragma_mmap_bytes: env_u64("ATLAS_SQLITE_MMAP_BYTES", 256 * 1024 * 1024) as i64,
        max_open_shards_per_pod: env_usize("ATLAS_MAX_OPEN_SHARDS_PER_POD", 16),
        startup_warmup_jitter_max_ms: env_u64("ATLAS_STARTUP_WARMUP_JITTER_MAX_MS", 0),
        ..DatasetCacheConfig::default()
    };
    let api_cfg = ApiConfig {
        max_body_bytes: env_usize("ATLAS_MAX_BODY_BYTES", 16 * 1024),
        request_timeout: env_duration_ms("ATLAS_REQUEST_TIMEOUT_MS", 5000),
        sql_timeout: env_duration_ms("ATLAS_SQL_TIMEOUT_MS", 800),
        response_max_bytes: env_usize("ATLAS_RESPONSE_MAX_BYTES", 512 * 1024),
        slow_query_threshold: env_duration_ms("ATLAS_SLOW_QUERY_THRESHOLD_MS", 200),
        enable_debug_datasets: env_bool("ATLAS_ENABLE_DEBUG_DATASETS", false),
        enable_exemplars: env_bool("ATLAS_ENABLE_EXEMPLARS", false),
        readiness_requires_catalog: env_bool("ATLAS_READINESS_REQUIRES_CATALOG", true),
        heavy_worker_pool_size: env_usize("ATLAS_HEAVY_WORKER_POOL_SIZE", 8),
        shed_load_enabled: env_bool("ATLAS_SHED_LOAD_ENABLED", false),
        shed_latency_p95_threshold_ms: env_u64("ATLAS_SHED_LATENCY_P95_MS", 900),
        shed_latency_min_samples: env_usize("ATLAS_SHED_MIN_SAMPLES", 50),
        enable_response_compression: env_bool("ATLAS_ENABLE_RESPONSE_COMPRESSION", true),
        compression_min_bytes: env_usize("ATLAS_COMPRESSION_MIN_BYTES", 4096),
        query_coalesce_ttl: env_duration_ms("ATLAS_QUERY_COALESCE_TTL_MS", 500),
        redis_url: env::var("ATLAS_REDIS_URL").ok(),
        redis_prefix: env::var("ATLAS_REDIS_PREFIX").unwrap_or_else(|_| "atlas".to_string()),
        enable_redis_response_cache: env_bool("ATLAS_ENABLE_REDIS_RESPONSE_CACHE", false),
        redis_response_cache_ttl_secs: env_usize("ATLAS_REDIS_RESPONSE_CACHE_TTL_SECS", 30),
        enable_redis_rate_limit: env_bool("ATLAS_ENABLE_REDIS_RATE_LIMIT", false),
        enable_cheap_only_survival: env_bool("ATLAS_ENABLE_CHEAP_ONLY_SURVIVAL", false),
        allow_min_viable_response: env_bool("ATLAS_ALLOW_MIN_VIABLE_RESPONSE", true),
        ..ApiConfig::default()
    };

    let startup_warmup_jitter_max_ms = cache_cfg.startup_warmup_jitter_max_ms;
    let backend: Arc<dyn bijux_atlas_server::DatasetStoreBackend> =
        if env_bool("ATLAS_STORE_S3_ENABLED", false) {
            let base_url = env::var("ATLAS_STORE_S3_BASE_URL")
                .map_err(|_| "ATLAS_STORE_S3_BASE_URL is required when S3 enabled".to_string())?;
            Arc::new(S3LikeBackend::new(
                base_url,
                env::var("ATLAS_STORE_S3_BEARER").ok(),
                RetryPolicy {
                    max_attempts: env_usize("ATLAS_STORE_RETRY_ATTEMPTS", 4),
                    base_backoff_ms: env_u64("ATLAS_STORE_RETRY_BASE_MS", 120),
                },
            ))
        } else {
            Arc::new(LocalFsBackend::new(store_root))
        };
    let cache = DatasetCacheManager::new(cache_cfg, backend);
    cache.spawn_background_tasks();
    if startup_warmup_jitter_max_ms > 0 {
        let delay = pod_jitter_ms(startup_warmup_jitter_max_ms);
        if delay > 0 {
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
    }
    if let Err(e) = cache.startup_warmup().await {
        error!("startup warmup failed: {e}");
    }

    let state = AppState::with_config(
        cache.clone(),
        api_cfg,
        bijux_atlas_query::QueryLimits::default(),
    );
    let app = build_router(state.clone());

    // Ready only after first successful catalog refresh when required.
    state.ready.store(false, Ordering::Relaxed);
    if let Err(e) = cache.refresh_catalog().await {
        if cache.cached_only_mode() {
            state.ready.store(true, Ordering::Relaxed);
        } else {
            error!("initial catalog refresh failed: {e}");
        }
    } else {
        state.ready.store(true, Ordering::Relaxed);
    }

    let cache_bg = cache.clone();
    let ready_bg = state.ready.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            match cache_bg.refresh_catalog().await {
                Ok(_) => ready_bg.store(true, Ordering::Relaxed),
                Err(e) => {
                    if cache_bg.cached_only_mode() {
                        ready_bg.store(true, Ordering::Relaxed);
                    } else {
                        error!("catalog refresh failed: {e}");
                        ready_bg.store(false, Ordering::Relaxed);
                    }
                }
            }
        }
    });

    let addr: std::net::SocketAddr = bind_addr
        .parse()
        .map_err(|e| format!("invalid bind addr {bind_addr}: {e}"))?;
    let socket = if addr.is_ipv4() {
        tokio::net::TcpSocket::new_v4().map_err(|e| format!("socket v4 failed: {e}"))?
    } else {
        tokio::net::TcpSocket::new_v6().map_err(|e| format!("socket v6 failed: {e}"))?
    };
    socket
        .set_reuseaddr(true)
        .map_err(|e| format!("set_reuseaddr failed: {e}"))?;
    socket
        .set_keepalive(env_bool("ATLAS_TCP_KEEPALIVE_ENABLED", true))
        .map_err(|e| format!("set_keepalive failed: {e}"))?;
    socket.bind(addr).map_err(|e| format!("bind failed: {e}"))?;
    let listener: TcpListener = socket
        .listen(1024)
        .map_err(|e| format!("listen failed: {e}"))?;
    info!("atlas-server listening on {bind_addr}");
    let accepting = state.accepting_requests.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            wait_for_shutdown_signal().await;
            accepting.store(false, Ordering::Relaxed);
            let drain_ms = env_u64("ATLAS_SHUTDOWN_DRAIN_MS", 5000);
            tokio::time::sleep(Duration::from_millis(drain_ms)).await;
        })
        .await
        .map_err(|e| format!("server failed: {e}"))
}
