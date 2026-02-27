// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bijux_atlas_core::sha256_hex;
use bijux_atlas_server::{
    build_router, load_runtime_startup_config, ApiConfig, AppState, DatasetCacheConfig,
    DatasetCacheManager, FederatedBackend, LocalFsBackend, RegistrySource, RetryPolicy,
    S3LikeBackend,
};
use clap::Parser;
use opentelemetry::trace::TracerProvider as _;
use redis::AsyncCommands;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Parser, Debug)]
#[command(name = "atlas-server", version, about = "Bijux Atlas runtime server")]
struct ServerCliArgs {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    bind: Option<String>,
    #[arg(long)]
    store_root: Option<PathBuf>,
    #[arg(long)]
    cache_root: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    print_effective_config: bool,
    #[arg(long, default_value_t = false)]
    validate_config: bool,
}

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

fn env_f64(name: &str, default: f64) -> f64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
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

fn env_map(name: &str) -> std::collections::HashMap<String, String> {
    env::var(name)
        .unwrap_or_default()
        .split(',')
        .filter_map(|item| {
            let (k, v) = item.split_once('=')?;
            let key = k.trim();
            let value = v.trim();
            if key.is_empty() || value.is_empty() {
                return None;
            }
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn env_list(name: &str) -> Vec<String> {
    env::var(name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_registry_sources(retry: RetryPolicy) -> Result<Option<Vec<RegistrySource>>, String> {
    let raw = env::var("ATLAS_REGISTRY_SOURCES").unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(None);
    }
    let signatures = env_map("ATLAS_REGISTRY_SIGNATURES");
    let ttl = env_duration_ms("ATLAS_REGISTRY_TTL_MS", 15_000);
    let max_sources = env_usize("ATLAS_REGISTRY_MAX_SOURCES", 8);
    let mut sources = Vec::new();
    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let (name, spec) = piece
            .split_once('=')
            .ok_or_else(|| format!("invalid ATLAS_REGISTRY_SOURCES entry: {piece}"))?;
        let name = name.trim();
        let spec = spec.trim();
        let backend: Arc<dyn bijux_atlas_server::DatasetStoreBackend> = if let Some(path) =
            spec.strip_prefix("local:")
        {
            Arc::new(LocalFsBackend::new(PathBuf::from(path)))
        } else if let Some(url) = spec.strip_prefix("s3:") {
            Arc::new(S3LikeBackend::new(
                url.to_string(),
                env::var("ATLAS_STORE_S3_PRESIGNED_BASE_URL").ok(),
                env::var("ATLAS_STORE_S3_BEARER").ok(),
                retry.clone(),
                env_bool("ATLAS_ALLOW_PRIVATE_STORE_HOSTS", false),
            ))
        } else if let Some(url) = spec.strip_prefix("http:") {
            Arc::new(S3LikeBackend::new(
                url.to_string(),
                None,
                env::var("ATLAS_STORE_HTTP_BEARER").ok(),
                retry.clone(),
                env_bool("ATLAS_ALLOW_PRIVATE_STORE_HOSTS", false),
            ))
        } else {
            return Err(format!(
                    "unsupported registry source scheme in {piece}; use local:/path, s3:https://..., or http:https://..."
                ));
        };
        sources.push(RegistrySource::new(
            name,
            backend,
            ttl,
            signatures.get(name).cloned(),
        ));
    }
    if sources.len() > max_sources {
        return Err(format!(
            "ATLAS_REGISTRY_SOURCES exceeds max allowed sources: {} > {}",
            sources.len(),
            max_sources
        ));
    }

    let priority = env::var("ATLAS_REGISTRY_PRIORITY").unwrap_or_default();
    if !priority.trim().is_empty() {
        let mut by_name: std::collections::HashMap<String, RegistrySource> =
            sources.into_iter().map(|s| (s.name.clone(), s)).collect();
        let mut ordered = Vec::new();
        for name in priority.split(',').map(str::trim).filter(|x| !x.is_empty()) {
            if let Some(src) = by_name.remove(name) {
                ordered.push(src);
            }
        }
        let mut rest: Vec<RegistrySource> = by_name.into_values().collect();
        rest.sort_by(|a, b| a.name.cmp(&b.name));
        ordered.extend(rest);
        sources = ordered;
    }

    Ok(Some(sources))
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

async fn coordinated_startup_warmup_datasets(
    datasets: Vec<bijux_atlas_model::DatasetId>,
    redis_url: Option<&str>,
    enabled: bool,
    lock_ttl_secs: u64,
    pod_id: &str,
) -> Vec<bijux_atlas_model::DatasetId> {
    if !enabled {
        return datasets;
    }
    let Some(url) = redis_url else {
        return datasets;
    };
    let Ok(client) = redis::Client::open(url) else {
        return datasets;
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return datasets;
    };
    let mut claimed = Vec::new();
    for ds in datasets {
        let key = format!("atlas:warmup:{}", ds.canonical_string());
        let lock_val = format!("{pod_id}:{}", chrono_like_millis());
        let set_res: Result<bool, redis::RedisError> = conn.set_nx(&key, lock_val).await;
        match set_res {
            Ok(true) => {
                let _: Result<(), redis::RedisError> =
                    conn.expire(&key, lock_ttl_secs as i64).await;
                claimed.push(ds);
            }
            Ok(false) => {}
            Err(_) => {
                claimed.push(ds);
            }
        }
    }
    claimed
}

fn chrono_like_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

async fn wait_for_shutdown_signal() -> Result<(), String> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())
            .map_err(|e| format!("failed to register SIGTERM handler: {e}"))?;
        let mut sigint = signal(SignalKind::interrupt())
            .map_err(|e| format!("failed to register SIGINT handler: {e}"))?;
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| format!("failed to register ctrl-c handler: {e}"))?;
    }
    Ok(())
}

fn init_tracing() -> Result<(), String> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let log_json = env_bool("ATLAS_LOG_JSON", true);
    if env_bool("ATLAS_OTEL_ENABLED", false) {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()
            .map_err(|e| format!("failed to build OTLP span exporter: {e}"))?;
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
    Ok(())
}

fn validate_runtime_env_contract() -> Result<(), String> {
    let raw = include_str!("../../../configs/contracts/env.schema.json");
    let parsed: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("invalid env contract json: {e}"))?;
    let allowed: HashSet<String> = parsed["allowed_env"]
        .as_array()
        .ok_or_else(|| "env contract missing allowed_env array".to_string())?
        .iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect();
    let dev_flag = parsed["dev_mode_allow_unknown_env"]
        .as_str()
        .unwrap_or("ATLAS_DEV_ALLOW_UNKNOWN_ENV");
    let allow_unknown = env_bool(dev_flag, false);

    let mut unknown: Vec<String> = env::vars()
        .map(|(k, _)| k)
        .filter(|k| k.starts_with("ATLAS_") || k.starts_with("BIJUX_"))
        .filter(|k| !allowed.contains(k))
        .collect();
    unknown.sort();
    unknown.dedup();
    if unknown.is_empty() {
        return Ok(());
    }
    if allow_unknown {
        info!(
            unknown = ?unknown,
            "unknown env vars permitted in dev mode"
        );
        return Ok(());
    }
    Err(format!(
        "unknown env vars rejected by contract; set {dev_flag}=1 only for local dev override: {}",
        unknown.join(",")
    ))
}

fn validate_prod_config_contract(
    bind_addr: &str,
    api: &ApiConfig,
    cache: &DatasetCacheConfig,
) -> Result<(), String> {
    let env_name = env::var("ATLAS_ENV").unwrap_or_else(|_| "dev".to_string());
    if !env_name.eq_ignore_ascii_case("prod") {
        return Ok(());
    }
    if bind_addr.contains("127.0.0.1") || bind_addr.contains("localhost") {
        return Err("ATLAS_ENV=prod forbids localhost/loopback bind addresses".to_string());
    }
    if cache.cached_only_mode {
        return Err("ATLAS_ENV=prod forbids ATLAS_CACHED_ONLY_MODE=true".to_string());
    }
    if api.redis_url.as_deref().is_none_or(str::is_empty) {
        return Err("ATLAS_ENV=prod requires ATLAS_REDIS_URL".to_string());
    }
    if api.require_api_key && api.allowed_api_keys.is_empty() {
        return Err(
            "ATLAS_ENV=prod requires non-empty ATLAS_ALLOWED_API_KEYS when api key auth is enabled"
                .to_string(),
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = ServerCliArgs::parse();
    init_tracing()?;
    validate_runtime_env_contract()?;

    let startup = load_runtime_startup_config(
        cli.config.as_deref(),
        cli.bind.as_deref(),
        cli.store_root.as_deref(),
        cli.cache_root.as_deref(),
    )?;
    let bind_addr = startup.bind_addr.clone();
    let store_root = startup.store_root.clone();
    let cache_root = startup.cache_root.clone();

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
    let startup_warmup = env_dataset_list("ATLAS_STARTUP_WARMUP");

    let cache_cfg = DatasetCacheConfig {
        disk_root: cache_root,
        max_disk_bytes: env_u64("ATLAS_MAX_DISK_BYTES", 8 * 1024 * 1024 * 1024),
        disk_high_watermark_pct: env_u64("ATLAS_CACHE_HIGH_WATERMARK_PCT", 90) as u8,
        disk_low_watermark_pct: env_u64("ATLAS_CACHE_LOW_WATERMARK_PCT", 75) as u8,
        max_dataset_count: env_usize("ATLAS_MAX_DATASET_COUNT", 8),
        pinned_datasets: pinned,
        startup_warmup,
        startup_warmup_limit: env_usize("ATLAS_STARTUP_WARMUP_LIMIT", 8),
        fail_readiness_on_missing_warmup: env_bool("ATLAS_FAIL_ON_WARMUP_ERROR", false),
        read_only_fs: env_bool("ATLAS_READ_ONLY_FS_MODE", false),
        cached_only_mode: env_bool("ATLAS_CACHED_ONLY_MODE", false),
        dataset_open_timeout: env_duration_ms("ATLAS_DATASET_OPEN_TIMEOUT_MS", 3000),
        store_breaker_failure_threshold: env_u64("ATLAS_STORE_BREAKER_FAILURE_THRESHOLD", 5) as u32,
        store_breaker_open_duration: env_duration_ms("ATLAS_STORE_BREAKER_OPEN_MS", 20_000),
        store_retry_budget: env_u64("ATLAS_STORE_RETRY_BUDGET", 20) as u32,
        max_concurrent_downloads: env_usize("ATLAS_MAX_CONCURRENT_DOWNLOADS", 3),
        max_concurrent_downloads_node: {
            let v = env_usize("ATLAS_MAX_CONCURRENT_DOWNLOADS_NODE", 0);
            (v > 0).then_some(v)
        },
        integrity_reverify_interval: env_duration_ms("ATLAS_INTEGRITY_REVERIFY_MS", 300_000),
        sqlite_pragma_cache_kib: env_u64("ATLAS_SQLITE_CACHE_KIB", 32 * 1024) as i64,
        sqlite_pragma_mmap_bytes: env_u64("ATLAS_SQLITE_MMAP_BYTES", 256 * 1024 * 1024) as i64,
        max_open_shards_per_pod: env_usize("ATLAS_MAX_OPEN_SHARDS_PER_POD", 16),
        startup_warmup_jitter_max_ms: env_u64("ATLAS_STARTUP_WARMUP_JITTER_MAX_MS", 0),
        catalog_backoff_base_ms: env_u64("ATLAS_CATALOG_BACKOFF_BASE_MS", 250),
        catalog_breaker_failure_threshold: env_u64("ATLAS_CATALOG_BREAKER_FAILURE_THRESHOLD", 5)
            as u32,
        catalog_breaker_open_ms: env_u64("ATLAS_CATALOG_BREAKER_OPEN_MS", 5000),
        quarantine_after_corruption_failures: env_u64("ATLAS_QUARANTINE_CORRUPTION_FAILURES", 3)
            as u32,
        quarantine_retry_ttl: env_duration_ms("ATLAS_QUARANTINE_RETRY_TTL_MS", 300_000),
        registry_ttl: env_duration_ms("ATLAS_REGISTRY_TTL_MS", 15_000),
        registry_freeze_mode: env_bool("ATLAS_REGISTRY_FREEZE_MODE", false),
        ..DatasetCacheConfig::default()
    };
    let api_cfg = ApiConfig {
        max_body_bytes: env_usize("ATLAS_MAX_BODY_BYTES", 16 * 1024),
        max_uri_bytes: env_usize("ATLAS_MAX_URI_BYTES", 2048),
        max_header_bytes: env_usize("ATLAS_MAX_HEADER_BYTES", 16 * 1024),
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
        redis_timeout_ms: env_u64("ATLAS_REDIS_TIMEOUT_MS", 50),
        redis_retry_attempts: env_usize("ATLAS_REDIS_RETRY_ATTEMPTS", 2),
        redis_breaker_failure_threshold: env_u64("ATLAS_REDIS_BREAKER_FAILURE_THRESHOLD", 8) as u32,
        redis_breaker_open_ms: env_u64("ATLAS_REDIS_BREAKER_OPEN_MS", 3000),
        redis_cache_max_key_bytes: env_usize("ATLAS_REDIS_CACHE_MAX_KEY_BYTES", 256),
        redis_cache_max_cardinality: env_usize("ATLAS_REDIS_CACHE_MAX_CARDINALITY", 100_000),
        redis_cache_ttl_max_secs: env_usize("ATLAS_REDIS_CACHE_TTL_MAX_SECS", 60),
        enable_cheap_only_survival: env_bool("ATLAS_ENABLE_CHEAP_ONLY_SURVIVAL", false),
        allow_min_viable_response: env_bool("ATLAS_ALLOW_MIN_VIABLE_RESPONSE", true),
        continue_download_on_request_timeout_for_warmup: env_bool(
            "ATLAS_CONTINUE_DOWNLOAD_ON_TIMEOUT_FOR_WARMUP",
            true,
        ),
        max_sequence_bases: env_usize("ATLAS_MAX_SEQUENCE_BASES", 20_000),
        sequence_api_key_required_bases: env_usize("ATLAS_SEQUENCE_API_KEY_REQUIRED_BASES", 5_000),
        sequence_rate_limit_per_ip: bijux_atlas_server::RateLimitConfig {
            capacity: env_f64("ATLAS_SEQUENCE_RATE_LIMIT_CAPACITY", 15.0),
            refill_per_sec: env_f64("ATLAS_SEQUENCE_RATE_LIMIT_REFILL_PER_SEC", 5.0),
        },
        sequence_ttl: env_duration_ms("ATLAS_SEQUENCE_TTL_MS", 300_000),
        adaptive_rate_limit_factor: env_f64("ATLAS_ADAPTIVE_RATE_LIMIT_FACTOR", 0.5),
        adaptive_heavy_limit_factor: env_f64("ATLAS_ADAPTIVE_HEAVY_LIMIT_FACTOR", 0.5),
        emergency_global_breaker: env_bool("ATLAS_EMERGENCY_GLOBAL_BREAKER", false),
        disable_heavy_endpoints: env_bool("ATLAS_DISABLE_HEAVY_ENDPOINTS", false),
        memory_pressure_shed_enabled: env_bool("ATLAS_MEMORY_PRESSURE_SHED_ENABLED", false),
        memory_pressure_rss_bytes: env_u64(
            "ATLAS_MEMORY_PRESSURE_RSS_BYTES",
            3 * 1024 * 1024 * 1024,
        ),
        max_request_queue_depth: env_usize("ATLAS_MAX_REQUEST_QUEUE_DEPTH", 256),
        cors_allowed_origins: env_list("ATLAS_CORS_ALLOWED_ORIGINS"),
        enable_audit_log: env_bool("ATLAS_ENABLE_AUDIT_LOG", false),
        require_api_key: env_bool("ATLAS_REQUIRE_API_KEY", false),
        allowed_api_keys: env_list("ATLAS_ALLOWED_API_KEYS"),
        hmac_secret: env::var("ATLAS_HMAC_SECRET").ok().filter(|x| !x.is_empty()),
        hmac_required: env_bool("ATLAS_HMAC_REQUIRED", false),
        hmac_max_skew_secs: env_u64("ATLAS_HMAC_MAX_SKEW_SECS", 300),
        ..ApiConfig::default()
    };
    bijux_atlas_server::validate_startup_config_contract(&api_cfg, &cache_cfg)?;
    validate_prod_config_contract(&bind_addr, &api_cfg, &cache_cfg)?;

    if cli.validate_config {
        info!("configuration validated");
        return Ok(());
    }
    if cli.print_effective_config {
        let payload = bijux_atlas_server::effective_config_payload(&startup, &api_cfg, &cache_cfg)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&payload)
                .map_err(|err| format!("render effective config: {err}"))?
        );
        return Ok(());
    }

    let startup_warmup_jitter_max_ms = cache_cfg.startup_warmup_jitter_max_ms;
    let startup_warmup = coordinated_startup_warmup_datasets(
        cache_cfg.startup_warmup.clone(),
        api_cfg.redis_url.as_deref(),
        env_bool("ATLAS_WARM_COORDINATION_ENABLED", false),
        env_u64("ATLAS_WARM_COORDINATION_LOCK_TTL_SECS", 300),
        &env::var("HOSTNAME").unwrap_or_else(|_| "atlas-pod".to_string()),
    )
    .await;
    let cache_cfg = DatasetCacheConfig {
        startup_warmup,
        ..cache_cfg
    };
    let retry = RetryPolicy {
        max_attempts: env_usize("ATLAS_STORE_RETRY_ATTEMPTS", 4),
        base_backoff_ms: env_u64("ATLAS_STORE_RETRY_BASE_MS", 120),
    };
    let backend: Arc<dyn bijux_atlas_server::DatasetStoreBackend> =
        if let Some(registries) = parse_registry_sources(retry.clone())? {
            Arc::new(FederatedBackend::new(registries))
        } else if env_bool("ATLAS_STORE_S3_ENABLED", false) {
            let base_url = env::var("ATLAS_STORE_S3_BASE_URL")
                .map_err(|_| "ATLAS_STORE_S3_BASE_URL is required when S3 enabled".to_string())?;
            Arc::new(S3LikeBackend::new(
                base_url,
                env::var("ATLAS_STORE_S3_PRESIGNED_BASE_URL").ok(),
                env::var("ATLAS_STORE_S3_BEARER").ok(),
                retry,
                env_bool("ATLAS_ALLOW_PRIVATE_STORE_HOSTS", false),
            ))
        } else {
            Arc::new(LocalFsBackend::new(store_root))
        };
    let cache = DatasetCacheManager::new(cache_cfg.clone(), backend);
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

    let query_limits = bijux_atlas_query::QueryLimits::default();
    let policy_mode = env::var("ATLAS_POLICY_MODE").unwrap_or_else(|_| "strict".to_string());
    let runtime_policy_payload = serde_json::json!({
        "policy_mode": policy_mode,
        "api": &api_cfg,
        "cache": &cache_cfg,
        "limits": &query_limits
    });
    let runtime_policy_hash =
        match bijux_atlas_core::canonical::stable_json_bytes(&runtime_policy_payload) {
            Ok(bytes) => sha256_hex(&bytes),
            Err(_) => sha256_hex(b"runtime-policy-hash-fallback"),
        };
    info!(
        event = "policy_mode_selected",
        policy_mode = %policy_mode,
        "policy mode selected"
    );
    info!(
        runtime_policy_hash = %runtime_policy_hash,
        runtime_policy = %runtime_policy_payload,
        "canonical runtime policy"
    );

    let mut state = AppState::with_config(cache.clone(), api_cfg, query_limits);
    state.runtime_policy_hash = Arc::new(runtime_policy_hash);
    state.runtime_policy_mode = Arc::new(policy_mode);
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
    let state_for_shutdown = state.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            if let Err(err) = wait_for_shutdown_signal().await {
                warn!("shutdown signal handler failed: {err}");
            }
            accepting.store(false, Ordering::Relaxed);
            // Stop admitting heavy work first, then drain remaining requests.
            state_for_shutdown.begin_shutdown_drain_heavy();
            let drain_ms = env_u64("ATLAS_SHUTDOWN_DRAIN_MS", 5000);
            tokio::time::sleep(Duration::from_millis(drain_ms)).await;
        })
        .await
        .map_err(|e| format!("server failed: {e}"))
}
