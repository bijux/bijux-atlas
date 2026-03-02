// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bijux_atlas_core::sha256_hex;
use bijux_atlas_server::{
    build_router, effective_runtime_config_payload, load_runtime_config, AppState,
    DatasetCacheManager, FederatedBackend, LocalFsBackend, RegistrySource, S3LikeBackend,
};
use clap::Parser;
use opentelemetry::trace::TracerProvider as _;
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

fn pod_jitter_ms(seed_source: &str, max_ms: u64) -> u64 {
    if max_ms == 0 {
        return 0;
    }
    let seed = seed_source
        .bytes()
        .fold(0_u64, |acc, b| acc.wrapping_mul(131).wrapping_add(b as u64));
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
        match try_claim_startup_warmup_lock(&mut conn, &key, &lock_val, lock_ttl_secs).await {
            Ok(true) => {
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

fn chrono_like_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

#[cfg(test)]
mod tests {
    use super::try_claim_startup_warmup_lock;

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

fn init_tracing(log_json: bool, otel_enabled: bool) -> Result<(), String> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if otel_enabled {
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
    init_tracing(runtime.log_json, runtime.otel_enabled)?;

    let bind_addr = runtime.startup.bind_addr.clone();
    let effective_config_payload = effective_runtime_config_payload(&runtime)?;
    let effective_config_log = serde_json::to_string(&effective_config_payload)
        .map_err(|err| format!("render effective config log: {err}"))?;
    info!(
        effective_config = %effective_config_log,
        "effective runtime config"
    );

    if cli.validate_config {
        info!("configuration validated");
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
    let startup_warmup = coordinated_startup_warmup_datasets(
        runtime.cache.startup_warmup.clone(),
        runtime.api.redis_url.as_deref(),
        runtime.warm_coordination_enabled,
        runtime.warm_coordination_lock_ttl_secs,
        &runtime.pod_id,
    )
    .await;
    let cache_cfg = bijux_atlas_server::DatasetCacheConfig {
        startup_warmup,
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
        event = "policy_mode_selected",
        policy_mode = %policy_mode,
        "policy mode selected"
    );
    info!(
        runtime_policy_hash = %runtime_policy_hash,
        runtime_policy = %runtime_policy_payload,
        "canonical runtime policy"
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
            tokio::time::sleep(Duration::from_millis(shutdown_drain_ms)).await;
        })
        .await
        .map_err(|e| format!("server failed: {e}"))
}
