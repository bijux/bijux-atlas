// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bijux_atlas_core::sha256_hex;
use bijux_atlas_server::{
    build_router, effective_runtime_config_payload, init_tracing, load_runtime_config, AppState,
    DatasetCacheManager, FederatedBackend, LocalFsBackend, RegistrySource, S3LikeBackend,
    TraceConfig, TraceExporterKind,
};
use clap::Parser;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tracing::{error, info, warn};

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

fn pod_jitter_ms(seed_source: &str, max_ms: u64) -> u64 {
    if max_ms == 0 {
        return 0;
    }
    let seed = seed_source
        .bytes()
        .fold(0_u64, |acc, b| acc.wrapping_mul(131).wrapping_add(b as u64));
    seed % max_ms
}

#[derive(Debug, Clone)]
struct WarmupLockLease {
    key: String,
    value: String,
}

#[derive(Debug, Default)]
struct WarmupCoordinationPlan {
    datasets: Vec<bijux_atlas_model::DatasetId>,
    leases: Vec<WarmupLockLease>,
    contention_total: u64,
    expired_total: u64,
    wait_samples_ns: Vec<u64>,
}

#[derive(Debug)]
struct WarmupLeaseAttempt {
    lease: Option<WarmupLockLease>,
    contention_total: u64,
    expired_total: u64,
}

struct WarmupLeaseRequest<'a> {
    key: &'a str,
    lock_val: &'a str,
    lock_ttl_secs: u64,
    retry_budget: usize,
    retry_base_ms: u64,
    pod_id: &'a str,
    dataset: &'a str,
}

fn warmup_lock_retry_delay_ms(
    pod_id: &str,
    dataset_key: &str,
    attempt: usize,
    base_ms: u64,
) -> u64 {
    let multiplier = (attempt as u64).saturating_add(1);
    let cap_ms = base_ms.saturating_mul(multiplier);
    if cap_ms == 0 {
        return 0;
    }
    let seed = format!("{pod_id}:{dataset_key}:{attempt}");
    1 + pod_jitter_ms(&seed, cap_ms)
}

async fn coordinated_startup_warmup_datasets(
    datasets: Vec<bijux_atlas_model::DatasetId>,
    redis_url: Option<&str>,
    enabled: bool,
    lock_ttl_secs: u64,
    retry_budget: usize,
    retry_base_ms: u64,
    pod_id: &str,
) -> WarmupCoordinationPlan {
    if !enabled {
        return WarmupCoordinationPlan {
            datasets,
            ..WarmupCoordinationPlan::default()
        };
    }
    let Some(url) = redis_url else {
        return WarmupCoordinationPlan {
            datasets,
            ..WarmupCoordinationPlan::default()
        };
    };
    let Ok(client) = redis::Client::open(url) else {
        return WarmupCoordinationPlan {
            datasets,
            ..WarmupCoordinationPlan::default()
        };
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return WarmupCoordinationPlan {
            datasets,
            ..WarmupCoordinationPlan::default()
        };
    };
    let mut plan = WarmupCoordinationPlan::default();
    for ds in datasets {
        let dataset = ds.canonical_string();
        let key = format!("atlas:warmup:{dataset}");
        let lock_val = unique_lock_value(pod_id);
        let started = std::time::Instant::now();
        match acquire_startup_warmup_lease(
            &mut conn,
            WarmupLeaseRequest {
                key: &key,
                lock_val: &lock_val,
                lock_ttl_secs,
                retry_budget,
                retry_base_ms,
                pod_id,
                dataset: &dataset,
            },
        )
        .await
        {
            Ok(attempt) => {
                plan.contention_total += attempt.contention_total;
                plan.expired_total += attempt.expired_total;
                let Some(lease) = attempt.lease else {
                    continue;
                };
                plan.wait_samples_ns
                    .push(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
                info!(
                    event_id = "warmup_lock_acquired",
                    dataset = %dataset,
                    lock_key = %lease.key,
                    wait_ms = started.elapsed().as_millis() as u64,
                    "startup warmup lock acquired"
                );
                plan.datasets.push(ds);
                plan.leases.push(lease);
            }
            Err(err) => {
                warn!(
                    event_id = "warmup_lock_expired",
                    dataset = %ds.canonical_string(),
                    error = %err,
                    "warmup lock fallback to local startup"
                );
                plan.expired_total += 1;
                plan.datasets.push(ds);
            }
        }
    }
    plan
}

async fn try_claim_startup_warmup_lock(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
    lock_val: &str,
    lock_ttl_secs: u64,
) -> Result<bool, redis::RedisError> {
    let response: Option<String> = redis::cmd("SET")
        .arg(key)
        .arg(lock_val)
        .arg("NX")
        .arg("EX")
        .arg(lock_ttl_secs)
        .query_async(conn)
        .await?;
    Ok(response.as_deref() == Some("OK"))
}

async fn try_release_startup_warmup_lock(
    conn: &mut redis::aio::MultiplexedConnection,
    lease: &WarmupLockLease,
) -> Result<bool, redis::RedisError> {
    let released: i32 = redis::Script::new(
        r#"if redis.call("GET", KEYS[1]) == ARGV[1] then
               return redis.call("DEL", KEYS[1])
           end
           return 0"#,
    )
    .key(&lease.key)
    .arg(&lease.value)
    .invoke_async(conn)
    .await?;
    Ok(released == 1)
}

async fn startup_warmup_lock_ttl_ms(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
) -> Result<i64, redis::RedisError> {
    redis::cmd("PTTL").arg(key).query_async(conn).await
}

fn is_stale_startup_warmup_lock(ttl_ms: i64) -> bool {
    ttl_ms == -2 || ttl_ms == 0
}

async fn acquire_startup_warmup_lease(
    conn: &mut redis::aio::MultiplexedConnection,
    request: WarmupLeaseRequest<'_>,
) -> Result<WarmupLeaseAttempt, redis::RedisError> {
    let WarmupLeaseRequest {
        key,
        lock_val,
        lock_ttl_secs,
        retry_budget,
        retry_base_ms,
        pod_id,
        dataset,
    } = request;
    let mut contention_total = 0_u64;
    for attempt in 0..=retry_budget {
        if try_claim_startup_warmup_lock(conn, key, lock_val, lock_ttl_secs).await? {
            return Ok(WarmupLeaseAttempt {
                lease: Some(WarmupLockLease {
                    key: key.to_string(),
                    value: lock_val.to_string(),
                }),
                contention_total,
                expired_total: 0,
            });
        }
        contention_total += 1;
        warn!(
            event_id = "warmup_lock_contention",
            dataset = %dataset,
            lock_key = %key,
            attempt,
            "startup warmup lock contention"
        );
        if attempt == retry_budget {
            let ttl_ms = startup_warmup_lock_ttl_ms(conn, key).await?;
            if is_stale_startup_warmup_lock(ttl_ms)
                && try_claim_startup_warmup_lock(conn, key, lock_val, lock_ttl_secs).await?
            {
                warn!(
                    event_id = "warmup_lock_expired",
                    dataset = %dataset,
                    lock_key = %key,
                    "startup warmup lock expired before retry window closed"
                );
                return Ok(WarmupLeaseAttempt {
                    lease: Some(WarmupLockLease {
                        key: key.to_string(),
                        value: lock_val.to_string(),
                    }),
                    contention_total,
                    expired_total: 1,
                });
            }
            return Ok(WarmupLeaseAttempt {
                lease: None,
                contention_total,
                expired_total: 0,
            });
        }
        let delay_ms = warmup_lock_retry_delay_ms(pod_id, key, attempt, retry_base_ms);
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }
    Ok(WarmupLeaseAttempt {
        lease: None,
        contention_total,
        expired_total: 0,
    })
}

async fn release_startup_warmup_leases(redis_url: Option<&str>, leases: &[WarmupLockLease]) {
    if leases.is_empty() {
        return;
    }
    let Some(url) = redis_url else {
        return;
    };
    let Ok(client) = redis::Client::open(url) else {
        return;
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return;
    };
    for lease in leases {
        match try_release_startup_warmup_lock(&mut conn, lease).await {
            Ok(true) => {}
            Ok(false) => {
                warn!(
                    event_id = "warmup_lock_expired",
                    lock_key = %lease.key,
                    "startup warmup lock already expired or transferred before release"
                );
            }
            Err(err) => {
                warn!(
                    event_id = "warmup_lock_expired",
                    lock_key = %lease.key,
                    error = %err,
                    "startup warmup lock release failed"
                );
            }
        }
    }
}

fn unique_lock_value(pod_id: &str) -> String {
    static NONCE: AtomicU64 = AtomicU64::new(0);
    let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{pod_id}:{}:{nonce}:{}",
        std::process::id(),
        chrono_like_millis()
    )
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

fn observability_release_id() -> String {
    std::env::var("ATLAS_RELEASE_ID").unwrap_or_else(|_| "dev".to_string())
}

fn observability_governance_version() -> String {
    std::env::var("ATLAS_GOVERNANCE_VERSION").unwrap_or_else(|_| "main@unknown".to_string())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = ServerCliArgs::parse();
    let runtime = load_runtime_config(
        cli.config.as_deref(),
        cli.bind.as_deref(),
        cli.store_root.as_deref(),
        cli.cache_root.as_deref(),
    )
    .map_err(|err| err.to_string())?;
    let trace_exporter = match runtime.trace_exporter.as_str() {
        "jaeger" => TraceExporterKind::Jaeger,
        "file" => TraceExporterKind::File,
        "none" => TraceExporterKind::None,
        _ => TraceExporterKind::Otlp,
    };
    let trace_cfg = TraceConfig {
        log_json: runtime.log_json,
        otel_enabled: runtime.otel_enabled,
        sampling_ratio: runtime.trace_sampling_ratio,
        exporter: trace_exporter,
        otlp_endpoint: runtime.trace_otlp_endpoint.clone(),
        jaeger_endpoint: runtime.trace_jaeger_endpoint.clone(),
        trace_file_path: runtime.trace_file_path.clone(),
        service_name: runtime.trace_service_name.clone(),
    };
    init_tracing(&trace_cfg)?;

    let bind_addr = runtime.startup.bind_addr.clone();
    let effective_config_payload = effective_runtime_config_payload(&runtime)?;
    let effective_config_log = serde_json::to_string(&effective_config_payload)
        .map_err(|err| format!("render effective config log: {err}"))?;
    let release_id = observability_release_id();
    let governance_version = observability_governance_version();
    info!(
        event_id = "config_loaded",
        release_id = %release_id,
        governance_version = %governance_version,
        effective_config = %effective_config_log,
        "effective runtime config"
    );
    if runtime.api.audit.enabled {
        info!(
            target: "atlas_audit",
            event_id = "audit_config_loaded",
            audit_payload = %serde_json::json!({
                "event_id": "audit_config_loaded",
                "event_name": "config_loaded",
                "timestamp_policy": "runtime-unix-seconds",
                "timestamp_unix_s": (SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())),
                "sink": runtime.api.audit.sink.as_str(),
                "action": "runtime.config.read",
                "resource_kind": "namespace",
                "resource_id": bind_addr
            }),
            "audit event"
        );
    }

    if cli.validate_config {
        info!(
            event_id = "config_validated",
            release_id = %release_id,
            governance_version = %governance_version,
            "configuration validated"
        );
        return Ok(());
    }
    if cli.print_effective_config {
        println!(
            "{}",
            serde_json::to_string_pretty(&effective_config_payload)
                .map_err(|err| format!("render effective config: {err}"))?
        );
        return Ok(());
    }

    let startup_warmup_jitter_max_ms = runtime.cache.startup_warmup_jitter_max_ms;
    let startup_warmup_plan = coordinated_startup_warmup_datasets(
        runtime.cache.startup_warmup.clone(),
        runtime.api.redis_url.as_deref(),
        runtime.warm_coordination_enabled,
        runtime.warm_coordination_lock_ttl_secs,
        runtime.warm_coordination_retry_budget,
        runtime.warm_coordination_retry_base_ms,
        &runtime.pod_id,
    )
    .await;
    let cache_cfg = bijux_atlas_server::DatasetCacheConfig {
        startup_warmup: startup_warmup_plan.datasets.clone(),
        ..runtime.cache.clone()
    };
    let retry = bijux_atlas_server::RetryPolicy {
        max_attempts: runtime.store.retry.max_attempts,
        base_backoff_ms: runtime.store.retry.base_backoff_ms,
    };
    let backend: Arc<dyn bijux_atlas_server::DatasetStoreBackend> =
        if !runtime.store.registry_sources.is_empty() {
            let ttl = cache_cfg.registry_ttl;
            let registries: Result<Vec<RegistrySource>, String> = runtime
                .store
                .registry_sources
                .iter()
                .map(|row| {
                    let backend: Arc<dyn bijux_atlas_server::DatasetStoreBackend> =
                        match row.scheme.as_str() {
                            "local" => Arc::new(LocalFsBackend::new(std::path::PathBuf::from(
                                &row.endpoint,
                            ))),
                            "s3" => Arc::new(S3LikeBackend::new(
                                row.endpoint.clone(),
                                runtime.store.s3_presigned_base_url.clone(),
                                runtime.store.s3_bearer.clone(),
                                retry.clone(),
                                runtime.store.allow_private_hosts,
                            )),
                            "http" => Arc::new(S3LikeBackend::new(
                                row.endpoint.clone(),
                                None,
                                runtime.store.http_bearer.clone(),
                                retry.clone(),
                                runtime.store.allow_private_hosts,
                            )),
                            other => {
                                return Err(format!("unsupported registry source scheme: {other}"))
                            }
                        };
                    Ok(RegistrySource::new(
                        &row.name,
                        backend,
                        ttl,
                        row.signature.clone(),
                    ))
                })
                .collect();
            if let Some(registries) = Some(registries?) {
                Arc::new(FederatedBackend::new(registries))
            } else {
                unreachable!()
            }
        } else if matches!(runtime.store.mode, bijux_atlas_server::StoreMode::S3) {
            let base_url =
                runtime.store.s3_base_url.clone().ok_or_else(|| {
                    "ATLAS_STORE_S3_BASE_URL is required when S3 enabled".to_string()
                })?;
            Arc::new(S3LikeBackend::new(
                base_url,
                runtime.store.s3_presigned_base_url.clone(),
                runtime.store.s3_bearer.clone(),
                retry,
                runtime.store.allow_private_hosts,
            ))
        } else {
            Arc::new(LocalFsBackend::new(runtime.store.local_root.clone()))
        };
    let cache = DatasetCacheManager::new(cache_cfg.clone(), backend);
    cache
        .metrics
        .warmup_lock_contention_total
        .fetch_add(startup_warmup_plan.contention_total, Ordering::Relaxed);
    cache
        .metrics
        .warmup_lock_expired_total
        .fetch_add(startup_warmup_plan.expired_total, Ordering::Relaxed);
    cache
        .metrics
        .warmup_lock_wait_ns
        .lock()
        .await
        .extend(startup_warmup_plan.wait_samples_ns.iter().copied());
    cache.spawn_background_tasks();
    if startup_warmup_jitter_max_ms > 0 {
        let delay = pod_jitter_ms(&runtime.pod_id, startup_warmup_jitter_max_ms);
        if delay > 0 {
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
    }
    if let Err(e) = cache.startup_warmup().await {
        error!("startup warmup failed: {e}");
    }
    release_startup_warmup_leases(
        runtime.api.redis_url.as_deref(),
        &startup_warmup_plan.leases,
    )
    .await;

    let query_limits = bijux_atlas_query::QueryLimits::default();
    let policy_mode = runtime.policy_mode.clone();
    let runtime_policy_payload = serde_json::json!({
        "policy_mode": policy_mode,
        "api": &runtime.api,
        "cache": &cache_cfg,
        "limits": &query_limits
    });
    let runtime_policy_hash =
        match bijux_atlas_core::canonical::stable_json_bytes(&runtime_policy_payload) {
            Ok(bytes) => sha256_hex(&bytes),
            Err(_) => sha256_hex(b"runtime-policy-hash-fallback"),
        };
    info!(
        event_id = "policy_mode_selected",
        release_id = %release_id,
        governance_version = %governance_version,
        event = "policy_mode_selected",
        policy_mode = %policy_mode,
        "policy mode selected"
    );
    info!(
        event_id = "runtime_policy_selected",
        release_id = %release_id,
        governance_version = %governance_version,
        runtime_policy_hash = %runtime_policy_hash,
        runtime_policy = %runtime_policy_payload,
        "canonical runtime policy"
    );
    info!(
        event_id = "auth_mode_selected",
        release_id = %release_id,
        governance_version = %governance_version,
        event = "auth_mode_selected",
        auth_mode = runtime.api.auth_mode.as_str(),
        auth_disabled = runtime.api.auth_mode.as_str() == "disabled",
        admin_endpoints_enabled = runtime.api.enable_admin_endpoints,
        audit_enabled = runtime.api.audit.enabled,
        audit_sink = runtime.api.audit.sink.as_str(),
        "runtime auth mode selected"
    );

    let mut state = AppState::with_config(cache.clone(), runtime.api.clone(), query_limits);
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
    let shutdown_drain_ms = runtime.shutdown_drain_ms;
    let socket = if addr.is_ipv4() {
        tokio::net::TcpSocket::new_v4().map_err(|e| format!("socket v4 failed: {e}"))?
    } else {
        tokio::net::TcpSocket::new_v6().map_err(|e| format!("socket v6 failed: {e}"))?
    };
    socket
        .set_reuseaddr(true)
        .map_err(|e| format!("set_reuseaddr failed: {e}"))?;
    socket
        .set_keepalive(runtime.tcp_keepalive_enabled)
        .map_err(|e| format!("set_keepalive failed: {e}"))?;
    socket.bind(addr).map_err(|e| format!("bind failed: {e}"))?;
    let listener: TcpListener = socket
        .listen(1024)
        .map_err(|e| format!("listen failed: {e}"))?;
    info!(
        event_id = "startup",
        release_id = %release_id,
        governance_version = %governance_version,
        bind_addr = %bind_addr,
        "atlas-server listening"
    );
    if runtime.api.audit.enabled {
        info!(
            target: "atlas_audit",
            event_id = "audit_startup",
            audit_payload = %serde_json::json!({
                "event_id": "audit_startup",
                "event_name": "startup",
                "timestamp_policy": "runtime-unix-seconds",
                "timestamp_unix_s": (SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())),
                "sink": runtime.api.audit.sink.as_str(),
                "principal": "operator",
                "action": "runtime.startup",
                "resource_kind": "namespace",
                "resource_id": bind_addr
            }),
            "audit event"
        );
    }
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
            tokio::time::sleep(Duration::from_millis(shutdown_drain_ms)).await;
        })
        .await
        .map_err(|e| format!("server failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::{
        is_stale_startup_warmup_lock, try_claim_startup_warmup_lock,
        try_release_startup_warmup_lock, unique_lock_value, warmup_lock_retry_delay_ms,
        WarmupLockLease,
    };

    #[test]
    fn startup_lock_uses_atomic_set_command_shape() {
        let _ = try_claim_startup_warmup_lock;
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
        )
        .expect("read main.rs");
        assert!(source.contains("redis::cmd(\"SET\")"));
        assert!(source.contains(".arg(\"NX\")"));
        assert!(source.contains(".arg(\"EX\")"));
    }

    #[test]
    fn startup_lock_uses_unique_nonce_values() {
        let first = unique_lock_value("atlas-a");
        let second = unique_lock_value("atlas-a");
        assert_ne!(first, second);
        assert!(first.starts_with("atlas-a:"));
    }

    #[test]
    fn startup_lock_retry_delay_is_bounded_and_nonzero() {
        let delay = warmup_lock_retry_delay_ms("pod-a", "atlas:warmup:one", 2, 25);
        assert!(delay >= 1);
        assert!(delay <= 75);
    }

    #[test]
    fn missing_startup_lock_is_treated_as_expired() {
        assert!(is_stale_startup_warmup_lock(-2));
        assert!(is_stale_startup_warmup_lock(0));
        assert!(!is_stale_startup_warmup_lock(100));
    }

    #[tokio::test]
    #[ignore = "requires REDIS_URL and local Redis; non-CI integration test"]
    async fn startup_lock_recovers_after_owner_crash_and_release_is_owner_safe() {
        let redis_url = match std::env::var("REDIS_URL") {
            Ok(value) => value,
            Err(_) => return,
        };
        let client = match redis::Client::open(redis_url) {
            Ok(client) => client,
            Err(_) => return,
        };
        let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
            return;
        };
        let key = format!("atlas:test:warmup-lock:{}", unique_lock_value("suite"));
        let owner_a = unique_lock_value("owner-a");
        let owner_b = unique_lock_value("owner-b");

        assert!(try_claim_startup_warmup_lock(&mut conn, &key, &owner_a, 1)
            .await
            .expect("claim first owner"));
        assert!(!try_claim_startup_warmup_lock(&mut conn, &key, &owner_b, 1)
            .await
            .expect("contention before ttl"));

        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

        assert!(try_claim_startup_warmup_lock(&mut conn, &key, &owner_b, 1)
            .await
            .expect("claim after ttl expiry"));

        let stale_release = WarmupLockLease {
            key: key.clone(),
            value: owner_a,
        };
        assert!(!try_release_startup_warmup_lock(&mut conn, &stale_release)
            .await
            .expect("stale owner release should not delete"));

        let current_release = WarmupLockLease {
            key,
            value: owner_b,
        };
        assert!(try_release_startup_warmup_lock(&mut conn, &current_release)
            .await
            .expect("current owner release should delete"));
    }
}
