// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use bijux_atlas::{core as bijux_atlas_core, model as bijux_atlas_model};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

mod contract_artifacts;
mod env_parsing;

pub const CONFIG_SCHEMA_VERSION: &str = "1";
pub use contract_artifacts::{
    effective_config_payload, effective_runtime_config_payload, runtime_config_contract_snapshot,
    runtime_startup_config_docs_markdown, runtime_startup_config_schema_json,
};
use env_parsing::{
    env_bool, env_dataset_list, env_duration_ms, env_f64, env_list, env_u64, env_usize,
    parse_registry_source_specs, validate_url,
};

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitConfig {
    pub capacity: f64,
    pub refill_per_sec: f64,
}

#[derive(Debug, Clone, Serialize)]
pub enum CatalogMode {
    Required,
    Optional,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMode {
    #[default]
    Disabled,
    ApiKey,
    Token,
    Oidc,
    Mtls,
}

impl AuthMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::ApiKey => "api-key",
            Self::Token => "token",
            Self::Oidc => "oidc",
            Self::Mtls => "mtls",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AuditSink {
    #[default]
    Stdout,
    File,
    Otel,
}

impl AuditSink {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::File => "file",
            Self::Otel => "otel",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub sink: AuditSink,
    pub file_path: String,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub enum StoreMode {
    Local,
    S3,
    Federated,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreRetryConfig {
    pub max_attempts: usize,
    pub base_backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrySourceSpec {
    pub name: String,
    pub scheme: String,
    pub endpoint: String,
    pub signature: Option<String>,
    pub ttl_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreConfig {
    pub mode: StoreMode,
    pub local_root: PathBuf,
    pub s3_base_url: Option<String>,
    pub s3_presigned_base_url: Option<String>,
    pub s3_bearer: Option<String>,
    pub http_bearer: Option<String>,
    pub allow_private_hosts: bool,
    pub retry: StoreRetryConfig,
    pub registry_sources: Vec<RegistrySourceSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeConfig {
    pub startup: RuntimeStartupConfig,
    pub api: ApiConfig,
    pub cache: crate::DatasetCacheConfig,
    pub store: StoreConfig,
    pub env_name: String,
    pub catalog_mode: CatalogMode,
    pub log_json: bool,
    pub log_level: String,
    pub log_filter_targets: Option<String>,
    pub log_sampling_rate: f64,
    pub log_redaction_enabled: bool,
    pub log_rotation_max_bytes: u64,
    pub log_rotation_max_files: u32,
    pub otel_enabled: bool,
    pub trace_sampling_ratio: f64,
    pub trace_exporter: String,
    pub trace_otlp_endpoint: Option<String>,
    pub trace_jaeger_endpoint: Option<String>,
    pub trace_file_path: Option<String>,
    pub trace_service_name: String,
    pub trace_context_propagation_enabled: bool,
    pub warm_coordination_enabled: bool,
    pub warm_coordination_lock_ttl_secs: u64,
    pub warm_coordination_retry_budget: usize,
    pub warm_coordination_retry_base_ms: u64,
    pub pod_id: String,
    pub policy_mode: String,
    pub shutdown_drain_ms: u64,
    pub tcp_keepalive_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeConfigError {
    MissingRequiredEnv {
        name: String,
        message: String,
    },
    UnknownEnv {
        names: Vec<String>,
        dev_flag: String,
    },
    InvalidFormat {
        name: String,
        value: String,
        message: String,
    },
    InvalidValue {
        message: String,
    },
}

impl std::fmt::Display for RuntimeConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequiredEnv { message, .. } => write!(f, "{message}"),
            Self::UnknownEnv { names, dev_flag } => write!(
                f,
                "unknown env vars rejected by contract; set {dev_flag}=1 only for local dev override: {}",
                names.join(",")
            ),
            Self::InvalidFormat { message, .. } => write!(f, "{message}"),
            Self::InvalidValue { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for RuntimeConfigError {}

pub fn validate_runtime_env_contract() -> Result<(), RuntimeConfigError> {
    let raw = include_str!("../../../../../../configs/contracts/env.schema.json");
    let parsed: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| RuntimeConfigError::InvalidValue {
            message: format!("invalid env contract json: {e}"),
        })?;
    let allowed: HashSet<String> = parsed["allowed_env"]
        .as_array()
        .ok_or_else(|| RuntimeConfigError::InvalidValue {
            message: "env contract missing allowed_env array".to_string(),
        })?
        .iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect();
    let dev_flag = parsed["dev_mode_allow_unknown_env"]
        .as_str()
        .unwrap_or("ATLAS_DEV_ALLOW_UNKNOWN_ENV")
        .to_string();
    let allow_unknown = env_bool(&dev_flag, false)?;

    let mut unknown: Vec<String> = std::env::vars()
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
        return Ok(());
    }
    Err(RuntimeConfigError::UnknownEnv {
        names: unknown,
        dev_flag,
    })
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            capacity: 30.0,
            refill_per_sec: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiConfig {
    pub auth_mode: AuthMode,
    pub enable_admin_endpoints: bool,
    pub max_body_bytes: usize,
    pub max_uri_bytes: usize,
    pub max_header_bytes: usize,
    pub request_timeout: Duration,
    pub sql_timeout: Duration,
    pub response_max_bytes: usize,
    pub discovery_ttl: Duration,
    pub immutable_gene_ttl: Duration,
    pub enable_debug_datasets: bool,
    pub enable_api_key_rate_limit: bool,
    pub rate_limit_per_ip: RateLimitConfig,
    pub rate_limit_per_api_key: RateLimitConfig,
    pub concurrency_cheap: usize,
    pub concurrency_medium: usize,
    pub concurrency_heavy: usize,
    pub slow_query_threshold: Duration,
    pub enable_exemplars: bool,
    pub enable_metrics_endpoint: bool,
    pub readiness_requires_catalog: bool,
    pub heavy_worker_pool_size: usize,
    pub shed_load_enabled: bool,
    pub shed_latency_p95_threshold_ms: u64,
    pub shed_latency_min_samples: usize,
    pub enable_response_compression: bool,
    pub compression_min_bytes: usize,
    pub query_coalesce_ttl: Duration,
    pub redis_url: Option<String>,
    pub redis_prefix: String,
    pub enable_redis_response_cache: bool,
    pub redis_response_cache_ttl_secs: usize,
    pub enable_redis_rate_limit: bool,
    pub redis_timeout_ms: u64,
    pub redis_retry_attempts: usize,
    pub redis_breaker_failure_threshold: u32,
    pub redis_breaker_open_ms: u64,
    pub redis_cache_max_key_bytes: usize,
    pub redis_cache_max_cardinality: usize,
    pub redis_cache_ttl_max_secs: usize,
    pub enable_cheap_only_survival: bool,
    pub allow_min_viable_response: bool,
    pub continue_download_on_request_timeout_for_warmup: bool,
    pub max_sequence_bases: usize,
    pub sequence_api_key_required_bases: usize,
    pub sequence_rate_limit_per_ip: RateLimitConfig,
    pub sequence_ttl: Duration,
    pub adaptive_rate_limit_factor: f64,
    pub adaptive_heavy_limit_factor: f64,
    pub emergency_global_breaker: bool,
    pub disable_heavy_endpoints: bool,
    pub memory_pressure_shed_enabled: bool,
    pub memory_pressure_rss_bytes: u64,
    pub max_request_queue_depth: usize,
    pub cors_allowed_origins: Vec<String>,
    pub audit: AuditConfig,
    pub require_api_key: bool,
    pub allowed_api_keys: Vec<String>,
    pub api_key_expiration_days: u64,
    pub api_key_rotation_overlap_secs: u64,
    pub hmac_secret: Option<String>,
    pub hmac_required: bool,
    pub hmac_max_skew_secs: u64,
    pub require_https: bool,
    pub token_signing_secret: Option<String>,
    pub token_required_issuer: Option<String>,
    pub token_required_audience: Option<String>,
    pub token_required_scopes: Vec<String>,
    pub token_revoked_ids: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            auth_mode: AuthMode::Disabled,
            enable_admin_endpoints: false,
            max_body_bytes: 16 * 1024,
            max_uri_bytes: 2048,
            max_header_bytes: 16 * 1024,
            request_timeout: Duration::from_secs(5),
            sql_timeout: Duration::from_millis(800),
            response_max_bytes: 512 * 1024,
            discovery_ttl: Duration::from_secs(30),
            immutable_gene_ttl: Duration::from_secs(900),
            enable_debug_datasets: false,
            enable_api_key_rate_limit: false,
            rate_limit_per_ip: RateLimitConfig::default(),
            rate_limit_per_api_key: RateLimitConfig {
                capacity: 100.0,
                refill_per_sec: 30.0,
            },
            concurrency_cheap: 128,
            concurrency_medium: 64,
            concurrency_heavy: 16,
            slow_query_threshold: Duration::from_millis(200),
            enable_exemplars: false,
            enable_metrics_endpoint: true,
            readiness_requires_catalog: true,
            heavy_worker_pool_size: 8,
            shed_load_enabled: false,
            shed_latency_p95_threshold_ms: 900,
            shed_latency_min_samples: 50,
            enable_response_compression: true,
            compression_min_bytes: 4096,
            query_coalesce_ttl: Duration::from_millis(500),
            redis_url: None,
            redis_prefix: "atlas".to_string(),
            enable_redis_response_cache: false,
            redis_response_cache_ttl_secs: 30,
            enable_redis_rate_limit: false,
            redis_timeout_ms: 50,
            redis_retry_attempts: 2,
            redis_breaker_failure_threshold: 8,
            redis_breaker_open_ms: 3000,
            redis_cache_max_key_bytes: 256,
            redis_cache_max_cardinality: 100_000,
            redis_cache_ttl_max_secs: 60,
            enable_cheap_only_survival: false,
            allow_min_viable_response: true,
            continue_download_on_request_timeout_for_warmup: true,
            max_sequence_bases: 20_000,
            sequence_api_key_required_bases: 5_000,
            sequence_rate_limit_per_ip: RateLimitConfig {
                capacity: 15.0,
                refill_per_sec: 5.0,
            },
            sequence_ttl: Duration::from_secs(300),
            adaptive_rate_limit_factor: 0.5,
            adaptive_heavy_limit_factor: 0.5,
            emergency_global_breaker: false,
            disable_heavy_endpoints: false,
            memory_pressure_shed_enabled: false,
            memory_pressure_rss_bytes: 3 * 1024 * 1024 * 1024,
            max_request_queue_depth: 256,
            cors_allowed_origins: Vec::new(),
            audit: AuditConfig {
                enabled: false,
                sink: AuditSink::Stdout,
                file_path: "artifacts/server-audit/audit.log".to_string(),
                max_bytes: 1_048_576,
            },
            require_api_key: false,
            allowed_api_keys: Vec::new(),
            api_key_expiration_days: 90,
            api_key_rotation_overlap_secs: 86_400,
            hmac_secret: None,
            hmac_required: false,
            hmac_max_skew_secs: 300,
            require_https: false,
            token_signing_secret: None,
            token_required_issuer: None,
            token_required_audience: None,
            token_required_scopes: Vec::new(),
            token_revoked_ids: Vec::new(),
        }
    }
}

pub fn validate_startup_config_contract(
    api: &ApiConfig,
    cache: &crate::DatasetCacheConfig,
) -> Result<(), String> {
    if api.max_body_bytes == 0 || api.max_uri_bytes == 0 || api.max_header_bytes == 0 {
        return Err("api size limits must be > 0".to_string());
    }
    if api.request_timeout.is_zero() || api.sql_timeout.is_zero() {
        return Err("timeouts must be > 0".to_string());
    }
    if cache.disk_high_watermark_pct <= cache.disk_low_watermark_pct {
        return Err("cache watermark contract requires high > low".to_string());
    }
    if cache.max_dataset_count == 0 || cache.max_total_connections == 0 {
        return Err("cache capacity limits must be > 0".to_string());
    }
    if cache.max_concurrent_downloads == 0 {
        return Err("max concurrent downloads must be > 0".to_string());
    }
    if api.require_api_key && api.allowed_api_keys.is_empty() {
        return Err("require_api_key=true requires at least one allowed api key".to_string());
    }
    if api.api_key_expiration_days == 0 {
        return Err("api_key_expiration_days must be greater than 0".to_string());
    }
    if api.api_key_rotation_overlap_secs == 0 {
        return Err("api_key_rotation_overlap_secs must be greater than 0".to_string());
    }
    if api.require_api_key && api.hmac_required {
        return Err("api key auth and hmac auth cannot both be enabled".to_string());
    }
    if api.hmac_required && api.hmac_secret.as_deref().is_none_or(str::is_empty) {
        return Err("hmac_required=true requires a non-empty hmac_secret".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RuntimeStartupConfigFile {
    bind_addr: Option<String>,
    store_root: Option<PathBuf>,
    cache_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStartupConfig {
    pub bind_addr: String,
    pub store_root: PathBuf,
    pub cache_root: PathBuf,
}

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_STORE_ROOT: &str = "artifacts/server-store";
const DEFAULT_CACHE_ROOT: &str = "artifacts/server-cache";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("bijux-atlas workspace root")
        .to_path_buf()
}

fn resolve_runtime_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        repo_root().join(path)
    }
}

#[must_use]
pub fn default_runtime_store_root() -> PathBuf {
    resolve_runtime_path(PathBuf::from(DEFAULT_STORE_ROOT))
}

#[must_use]
pub fn default_runtime_cache_root() -> PathBuf {
    resolve_runtime_path(PathBuf::from(DEFAULT_CACHE_ROOT))
}

#[must_use]
pub fn default_runtime_policy_mode() -> String {
    std::env::var("ATLAS_POLICY_MODE").unwrap_or_else(|_| "strict".to_string())
}

fn resolve_runtime_startup_config(
    file_cfg: RuntimeStartupConfigFile,
    cli_bind_addr: Option<&str>,
    cli_store_root: Option<&Path>,
    cli_cache_root: Option<&Path>,
    env_bind_addr: Option<String>,
    env_store_root: Option<PathBuf>,
    env_cache_root: Option<PathBuf>,
) -> Result<RuntimeStartupConfig, String> {
    let bind_addr = cli_bind_addr
        .map(ToString::to_string)
        .or(env_bind_addr)
        .or(file_cfg.bind_addr)
        .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string());

    let store_root = cli_store_root
        .map(Path::to_path_buf)
        .or(env_store_root)
        .or(file_cfg.store_root)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_STORE_ROOT));

    let cache_root = cli_cache_root
        .map(Path::to_path_buf)
        .or(env_cache_root)
        .or(file_cfg.cache_root)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CACHE_ROOT));

    if bind_addr.trim().is_empty() {
        return Err("runtime config bind_addr must not be empty".to_string());
    }
    if store_root.as_os_str().is_empty() || cache_root.as_os_str().is_empty() {
        return Err("runtime config store_root/cache_root must not be empty".to_string());
    }
    let store_root = resolve_runtime_path(store_root);
    let cache_root = resolve_runtime_path(cache_root);

    Ok(RuntimeStartupConfig {
        bind_addr,
        store_root,
        cache_root,
    })
}

fn parse_runtime_startup_config_file(path: &Path) -> Result<RuntimeStartupConfigFile, String> {
    let text = std::fs::read_to_string(path).map_err(|err| {
        format!(
            "failed reading runtime config file {}: {err}",
            path.display()
        )
    })?;
    match path.extension().and_then(|v| v.to_str()) {
        Some("json") => serde_json::from_str(&text)
            .map_err(|err| format!("invalid runtime config json {}: {err}", path.display())),
        Some("yaml") | Some("yml") => serde_yaml::from_str(&text)
            .map_err(|err| format!("invalid runtime config yaml {}: {err}", path.display())),
        Some("toml") => toml::from_str(&text)
            .map_err(|err| format!("invalid runtime config toml {}: {err}", path.display())),
        _ => Err(format!(
            "unsupported runtime config extension for {} (expected .json/.yaml/.yml/.toml)",
            path.display()
        )),
    }
}

pub fn load_runtime_startup_config(
    config_path: Option<&Path>,
    cli_bind_addr: Option<&str>,
    cli_store_root: Option<&Path>,
    cli_cache_root: Option<&Path>,
) -> Result<RuntimeStartupConfig, String> {
    let file_cfg = if let Some(path) = config_path {
        parse_runtime_startup_config_file(path)?
    } else {
        RuntimeStartupConfigFile::default()
    };
    resolve_runtime_startup_config(
        file_cfg,
        cli_bind_addr,
        cli_store_root,
        cli_cache_root,
        std::env::var("ATLAS_BIND").ok(),
        std::env::var("ATLAS_STORE_ROOT").ok().map(PathBuf::from),
        std::env::var("ATLAS_CACHE_ROOT").ok().map(PathBuf::from),
    )
}

fn validate_runtime_config_contract(runtime: &RuntimeConfig) -> Result<(), RuntimeConfigError> {
    validate_startup_config_contract(&runtime.api, &runtime.cache)
        .map_err(|message| RuntimeConfigError::InvalidValue { message })?;
    if runtime.cache.cached_only_mode && runtime.api.readiness_requires_catalog {
        return Err(RuntimeConfigError::InvalidValue {
            message: "ATLAS_CACHED_ONLY_MODE=true requires ATLAS_READINESS_REQUIRES_CATALOG=false"
                .to_string(),
        });
    }
    if runtime.warm_coordination_enabled {
        if runtime.warm_coordination_lock_ttl_secs == 0 {
            return Err(RuntimeConfigError::InvalidValue {
                message: "ATLAS_WARM_COORDINATION_ENABLED=true requires ATLAS_WARM_COORDINATION_LOCK_TTL_SECS>0"
                    .to_string(),
            });
        }
        if runtime.warm_coordination_retry_budget == 0 {
            return Err(RuntimeConfigError::InvalidValue {
                message: "ATLAS_WARM_COORDINATION_ENABLED=true requires ATLAS_WARM_COORDINATION_RETRY_BUDGET>0"
                    .to_string(),
            });
        }
    }
    if !(0.0..=1.0).contains(&runtime.log_sampling_rate) {
        return Err(RuntimeConfigError::InvalidValue {
            message: "ATLAS_LOG_SAMPLING_RATE must be in [0.0, 1.0]".to_string(),
        });
    }
    if runtime.log_rotation_max_bytes == 0 {
        return Err(RuntimeConfigError::InvalidValue {
            message: "ATLAS_LOG_ROTATION_MAX_BYTES must be > 0".to_string(),
        });
    }
    if runtime.log_rotation_max_files == 0 {
        return Err(RuntimeConfigError::InvalidValue {
            message: "ATLAS_LOG_ROTATION_MAX_FILES must be > 0".to_string(),
        });
    }
    if !matches!(
        runtime.log_level.to_ascii_lowercase().as_str(),
        "trace" | "debug" | "info" | "warn" | "error"
    ) {
        return Err(RuntimeConfigError::InvalidValue {
            message: "ATLAS_LOG_LEVEL must be one of: trace, debug, info, warn, error".to_string(),
        });
    }
    if !(0.0..=1.0).contains(&runtime.trace_sampling_ratio) {
        return Err(RuntimeConfigError::InvalidValue {
            message: "ATLAS_TRACE_SAMPLING_RATIO must be in [0.0, 1.0]".to_string(),
        });
    }
    if !matches!(
        runtime.trace_exporter.as_str(),
        "otlp" | "jaeger" | "file" | "none"
    ) {
        return Err(RuntimeConfigError::InvalidValue {
            message: "ATLAS_TRACE_EXPORTER must be one of: otlp, jaeger, file, none".to_string(),
        });
    }
    if runtime.env_name.eq_ignore_ascii_case("prod") {
        if runtime.startup.bind_addr.contains("127.0.0.1")
            || runtime.startup.bind_addr.contains("localhost")
        {
            return Err(RuntimeConfigError::InvalidValue {
                message: "ATLAS_ENV=prod forbids localhost/loopback bind addresses".to_string(),
            });
        }
        if runtime.cache.cached_only_mode {
            return Err(RuntimeConfigError::InvalidValue {
                message: "ATLAS_ENV=prod forbids ATLAS_CACHED_ONLY_MODE=true".to_string(),
            });
        }
        if runtime.api.redis_url.as_deref().is_none_or(str::is_empty) {
            return Err(RuntimeConfigError::InvalidValue {
                message: "ATLAS_ENV=prod requires ATLAS_REDIS_URL".to_string(),
            });
        }
        if runtime.api.require_api_key && runtime.api.allowed_api_keys.is_empty() {
            return Err(RuntimeConfigError::InvalidValue {
                message:
                    "ATLAS_ENV=prod requires non-empty ATLAS_ALLOWED_API_KEYS when api key auth is enabled"
                        .to_string(),
            });
        }
        if runtime.api.token_signing_secret.is_some()
            && (runtime
                .api
                .token_required_issuer
                .as_deref()
                .is_none_or(str::is_empty)
                || runtime
                    .api
                    .token_required_audience
                    .as_deref()
                    .is_none_or(str::is_empty))
        {
            return Err(RuntimeConfigError::InvalidValue {
                message:
                    "token auth requires ATLAS_TOKEN_REQUIRED_ISSUER and ATLAS_TOKEN_REQUIRED_AUDIENCE"
                        .to_string(),
            });
        }
    }
    Ok(())
}

impl RuntimeConfig {
    pub fn from_env(startup: RuntimeStartupConfig) -> Result<Self, RuntimeConfigError> {
        validate_runtime_env_contract()?;

        let env_name = std::env::var("ATLAS_ENV").unwrap_or_else(|_| "dev".to_string());
        let pinned = std::env::var("ATLAS_PINNED_DATASETS")
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
            .collect::<HashSet<_>>();
        let startup_warmup = env_dataset_list("ATLAS_STARTUP_WARMUP")?;

        let cache = crate::DatasetCacheConfig {
            disk_root: startup.cache_root.clone(),
            max_disk_bytes: env_u64("ATLAS_MAX_DISK_BYTES", 8 * 1024 * 1024 * 1024)?,
            disk_high_watermark_pct: env_u64("ATLAS_CACHE_HIGH_WATERMARK_PCT", 90)? as u8,
            disk_low_watermark_pct: env_u64("ATLAS_CACHE_LOW_WATERMARK_PCT", 75)? as u8,
            max_dataset_count: env_usize("ATLAS_MAX_DATASET_COUNT", 8)?,
            pinned_datasets: pinned,
            startup_warmup,
            startup_warmup_limit: env_usize("ATLAS_STARTUP_WARMUP_LIMIT", 8)?,
            fail_readiness_on_missing_warmup: env_bool("ATLAS_FAIL_ON_WARMUP_ERROR", false)?,
            read_only_fs: env_bool("ATLAS_READ_ONLY_FS_MODE", false)?,
            cached_only_mode: env_bool("ATLAS_CACHED_ONLY_MODE", false)?,
            dataset_open_timeout: env_duration_ms("ATLAS_DATASET_OPEN_TIMEOUT_MS", 3000)?,
            store_breaker_failure_threshold: env_u64("ATLAS_STORE_BREAKER_FAILURE_THRESHOLD", 5)?
                as u32,
            store_breaker_open_duration: env_duration_ms("ATLAS_STORE_BREAKER_OPEN_MS", 20_000)?,
            store_retry_budget: env_u64("ATLAS_STORE_RETRY_BUDGET", 20)? as u32,
            max_concurrent_downloads: env_usize("ATLAS_MAX_CONCURRENT_DOWNLOADS", 3)?,
            max_concurrent_downloads_node: {
                let value = env_usize("ATLAS_MAX_CONCURRENT_DOWNLOADS_NODE", 0)?;
                (value > 0).then_some(value)
            },
            integrity_reverify_interval: env_duration_ms("ATLAS_INTEGRITY_REVERIFY_MS", 300_000)?,
            sqlite_pragma_cache_kib: env_u64("ATLAS_SQLITE_CACHE_KIB", 32 * 1024)? as i64,
            sqlite_pragma_mmap_bytes: env_u64("ATLAS_SQLITE_MMAP_BYTES", 256 * 1024 * 1024)? as i64,
            max_open_shards_per_pod: env_usize("ATLAS_MAX_OPEN_SHARDS_PER_POD", 16)?,
            startup_warmup_jitter_max_ms: env_u64("ATLAS_STARTUP_WARMUP_JITTER_MAX_MS", 0)?,
            catalog_backoff_base_ms: env_u64("ATLAS_CATALOG_BACKOFF_BASE_MS", 250)?,
            catalog_breaker_failure_threshold: env_u64(
                "ATLAS_CATALOG_BREAKER_FAILURE_THRESHOLD",
                5,
            )? as u32,
            catalog_breaker_open_ms: env_u64("ATLAS_CATALOG_BREAKER_OPEN_MS", 5000)?,
            quarantine_after_corruption_failures: env_u64(
                "ATLAS_QUARANTINE_CORRUPTION_FAILURES",
                3,
            )? as u32,
            quarantine_retry_ttl: env_duration_ms("ATLAS_QUARANTINE_RETRY_TTL_MS", 300_000)?,
            registry_ttl: env_duration_ms("ATLAS_REGISTRY_TTL_MS", 15_000)?,
            registry_freeze_mode: env_bool("ATLAS_REGISTRY_FREEZE_MODE", false)?,
            ..crate::DatasetCacheConfig::default()
        };

        let require_api_key_env = env_bool("ATLAS_REQUIRE_API_KEY", false)?;
        let hmac_required_env = env_bool("ATLAS_HMAC_REQUIRED", false)?;
        let allowed_api_keys = env_list("ATLAS_ALLOWED_API_KEYS");
        let hmac_secret = std::env::var("ATLAS_HMAC_SECRET")
            .ok()
            .filter(|x| !x.is_empty());
        let token_signing_secret = std::env::var("ATLAS_TOKEN_SIGNING_SECRET")
            .ok()
            .filter(|value| !value.is_empty());
        let token_required_issuer = std::env::var("ATLAS_TOKEN_REQUIRED_ISSUER")
            .ok()
            .filter(|value| !value.is_empty());
        let token_required_audience = std::env::var("ATLAS_TOKEN_REQUIRED_AUDIENCE")
            .ok()
            .filter(|value| !value.is_empty());
        let token_required_scopes = env_list("ATLAS_TOKEN_REQUIRED_SCOPES");
        let token_revoked_ids = env_list("ATLAS_TOKEN_REVOKED_IDS");
        let audit_sink = match std::env::var("ATLAS_AUDIT_SINK") {
            Ok(value) => match value.as_str() {
                "stdout" => AuditSink::Stdout,
                "file" => AuditSink::File,
                "otel" => AuditSink::Otel,
                _ => {
                    return Err(RuntimeConfigError::InvalidFormat {
                        name: "ATLAS_AUDIT_SINK".to_string(),
                        value,
                        message: "ATLAS_AUDIT_SINK must be one of: stdout, file, otel".to_string(),
                    });
                }
            },
            Err(std::env::VarError::NotPresent) => AuditSink::Stdout,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(RuntimeConfigError::InvalidFormat {
                    name: "ATLAS_AUDIT_SINK".to_string(),
                    value: "<non-unicode>".to_string(),
                    message: "ATLAS_AUDIT_SINK must be valid unicode".to_string(),
                });
            }
        };
        let audit_enabled = env_bool(
            "ATLAS_AUDIT_ENABLED",
            env_bool("ATLAS_ENABLE_AUDIT_LOG", false)?,
        )?;
        let audit_file_path = std::env::var("ATLAS_AUDIT_FILE_PATH")
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "artifacts/server-audit/audit.log".to_string());
        let audit_max_bytes = env_u64("ATLAS_AUDIT_MAX_BYTES", 1_048_576)?;
        let auth_mode_env = match std::env::var("ATLAS_AUTH_MODE") {
            Ok(value) => Some(match value.as_str() {
                "disabled" => AuthMode::Disabled,
                "api-key" => AuthMode::ApiKey,
                "token" => AuthMode::Token,
                "oidc" => AuthMode::Oidc,
                "mtls" => AuthMode::Mtls,
                _ => {
                    return Err(RuntimeConfigError::InvalidFormat {
                        name: "ATLAS_AUTH_MODE".to_string(),
                        value,
                        message:
                            "ATLAS_AUTH_MODE must be one of: disabled, api-key, token, oidc, mtls"
                                .to_string(),
                    });
                }
            }),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(RuntimeConfigError::InvalidFormat {
                    name: "ATLAS_AUTH_MODE".to_string(),
                    value: "<non-unicode>".to_string(),
                    message: "ATLAS_AUTH_MODE must be valid unicode".to_string(),
                });
            }
        };
        if require_api_key_env && hmac_required_env {
            return Err(RuntimeConfigError::InvalidValue {
                message:
                    "ATLAS_REQUIRE_API_KEY=true and ATLAS_HMAC_REQUIRED=true cannot be enabled together"
                        .to_string(),
            });
        }
        let auth_mode = match auth_mode_env {
            Some(AuthMode::Disabled) => {
                if require_api_key_env || hmac_required_env {
                    return Err(RuntimeConfigError::InvalidValue {
                        message: "ATLAS_AUTH_MODE=disabled conflicts with legacy auth enable flags"
                            .to_string(),
                    });
                }
                AuthMode::Disabled
            }
            Some(AuthMode::ApiKey) => {
                if hmac_required_env {
                    return Err(RuntimeConfigError::InvalidValue {
                        message: "ATLAS_AUTH_MODE=api-key conflicts with ATLAS_HMAC_REQUIRED=true"
                            .to_string(),
                    });
                }
                AuthMode::ApiKey
            }
            Some(AuthMode::Token) => AuthMode::Token,
            Some(AuthMode::Oidc) => AuthMode::Oidc,
            Some(AuthMode::Mtls) => AuthMode::Mtls,
            None => {
                if require_api_key_env {
                    AuthMode::ApiKey
                } else {
                    AuthMode::Disabled
                }
            }
        };

        let api = ApiConfig {
            auth_mode,
            enable_admin_endpoints: env_bool("ATLAS_ENABLE_ADMIN_ENDPOINTS", false)?,
            max_body_bytes: env_usize("ATLAS_MAX_BODY_BYTES", 16 * 1024)?,
            max_uri_bytes: env_usize("ATLAS_MAX_URI_BYTES", 2048)?,
            max_header_bytes: env_usize("ATLAS_MAX_HEADER_BYTES", 16 * 1024)?,
            request_timeout: env_duration_ms("ATLAS_REQUEST_TIMEOUT_MS", 5000)?,
            sql_timeout: env_duration_ms("ATLAS_SQL_TIMEOUT_MS", 800)?,
            response_max_bytes: env_usize("ATLAS_RESPONSE_MAX_BYTES", 512 * 1024)?,
            slow_query_threshold: env_duration_ms("ATLAS_SLOW_QUERY_THRESHOLD_MS", 200)?,
            enable_debug_datasets: env_bool("ATLAS_ENABLE_DEBUG_DATASETS", false)?,
            enable_exemplars: env_bool("ATLAS_ENABLE_EXEMPLARS", false)?,
            enable_metrics_endpoint: env_bool("ATLAS_ENABLE_METRICS_ENDPOINT", true)?,
            readiness_requires_catalog: env_bool("ATLAS_READINESS_REQUIRES_CATALOG", true)?,
            heavy_worker_pool_size: env_usize("ATLAS_HEAVY_WORKER_POOL_SIZE", 8)?,
            shed_load_enabled: env_bool("ATLAS_SHED_LOAD_ENABLED", false)?,
            shed_latency_p95_threshold_ms: env_u64("ATLAS_SHED_LATENCY_P95_MS", 900)?,
            shed_latency_min_samples: env_usize("ATLAS_SHED_MIN_SAMPLES", 50)?,
            enable_response_compression: env_bool("ATLAS_ENABLE_RESPONSE_COMPRESSION", true)?,
            compression_min_bytes: env_usize("ATLAS_COMPRESSION_MIN_BYTES", 4096)?,
            query_coalesce_ttl: env_duration_ms("ATLAS_QUERY_COALESCE_TTL_MS", 500)?,
            redis_url: std::env::var("ATLAS_REDIS_URL").ok(),
            redis_prefix: std::env::var("ATLAS_REDIS_PREFIX")
                .unwrap_or_else(|_| "atlas".to_string()),
            enable_redis_response_cache: env_bool("ATLAS_ENABLE_REDIS_RESPONSE_CACHE", false)?,
            redis_response_cache_ttl_secs: env_usize("ATLAS_REDIS_RESPONSE_CACHE_TTL_SECS", 30)?,
            enable_redis_rate_limit: env_bool("ATLAS_ENABLE_REDIS_RATE_LIMIT", false)?,
            redis_timeout_ms: env_u64("ATLAS_REDIS_TIMEOUT_MS", 50)?,
            redis_retry_attempts: env_usize("ATLAS_REDIS_RETRY_ATTEMPTS", 2)?,
            redis_breaker_failure_threshold: env_u64("ATLAS_REDIS_BREAKER_FAILURE_THRESHOLD", 8)?
                as u32,
            redis_breaker_open_ms: env_u64("ATLAS_REDIS_BREAKER_OPEN_MS", 3000)?,
            redis_cache_max_key_bytes: env_usize("ATLAS_REDIS_CACHE_MAX_KEY_BYTES", 256)?,
            redis_cache_max_cardinality: env_usize("ATLAS_REDIS_CACHE_MAX_CARDINALITY", 100_000)?,
            redis_cache_ttl_max_secs: env_usize("ATLAS_REDIS_CACHE_TTL_MAX_SECS", 60)?,
            enable_cheap_only_survival: env_bool("ATLAS_ENABLE_CHEAP_ONLY_SURVIVAL", false)?,
            allow_min_viable_response: env_bool("ATLAS_ALLOW_MIN_VIABLE_RESPONSE", true)?,
            continue_download_on_request_timeout_for_warmup: env_bool(
                "ATLAS_CONTINUE_DOWNLOAD_ON_TIMEOUT_FOR_WARMUP",
                true,
            )?,
            max_sequence_bases: env_usize("ATLAS_MAX_SEQUENCE_BASES", 20_000)?,
            sequence_api_key_required_bases: env_usize(
                "ATLAS_SEQUENCE_API_KEY_REQUIRED_BASES",
                5_000,
            )?,
            sequence_rate_limit_per_ip: RateLimitConfig {
                capacity: env_f64("ATLAS_SEQUENCE_RATE_LIMIT_CAPACITY", 15.0)?,
                refill_per_sec: env_f64("ATLAS_SEQUENCE_RATE_LIMIT_REFILL_PER_SEC", 5.0)?,
            },
            sequence_ttl: env_duration_ms("ATLAS_SEQUENCE_TTL_MS", 300_000)?,
            adaptive_rate_limit_factor: env_f64("ATLAS_ADAPTIVE_RATE_LIMIT_FACTOR", 0.5)?,
            adaptive_heavy_limit_factor: env_f64("ATLAS_ADAPTIVE_HEAVY_LIMIT_FACTOR", 0.5)?,
            emergency_global_breaker: env_bool("ATLAS_EMERGENCY_GLOBAL_BREAKER", false)?,
            disable_heavy_endpoints: env_bool("ATLAS_DISABLE_HEAVY_ENDPOINTS", false)?,
            memory_pressure_shed_enabled: env_bool("ATLAS_MEMORY_PRESSURE_SHED_ENABLED", false)?,
            memory_pressure_rss_bytes: env_u64(
                "ATLAS_MEMORY_PRESSURE_RSS_BYTES",
                3 * 1024 * 1024 * 1024,
            )?,
            max_request_queue_depth: env_usize("ATLAS_MAX_REQUEST_QUEUE_DEPTH", 256)?,
            cors_allowed_origins: env_list("ATLAS_CORS_ALLOWED_ORIGINS"),
            audit: AuditConfig {
                enabled: audit_enabled,
                sink: audit_sink,
                file_path: audit_file_path,
                max_bytes: audit_max_bytes,
            },
            require_api_key: matches!(auth_mode, AuthMode::ApiKey),
            allowed_api_keys,
            api_key_expiration_days: env_u64("ATLAS_API_KEY_EXPIRATION_DAYS", 90)?,
            api_key_rotation_overlap_secs: env_u64("ATLAS_API_KEY_ROTATION_OVERLAP_SECS", 86_400)?,
            hmac_secret,
            hmac_required: hmac_required_env,
            hmac_max_skew_secs: env_u64("ATLAS_HMAC_MAX_SKEW_SECS", 300)?,
            require_https: env_bool("ATLAS_REQUIRE_HTTPS", false)?,
            token_signing_secret,
            token_required_issuer,
            token_required_audience,
            token_required_scopes,
            token_revoked_ids,
            ..ApiConfig::default()
        };

        let retry = StoreRetryConfig {
            max_attempts: env_usize("ATLAS_STORE_RETRY_ATTEMPTS", 4)?,
            base_backoff_ms: env_u64("ATLAS_STORE_RETRY_BASE_MS", 120)?,
        };
        let registry_sources = parse_registry_source_specs(&retry)?;
        let s3_enabled = env_bool("ATLAS_STORE_S3_ENABLED", false)?;
        let s3_base_url = std::env::var("ATLAS_STORE_S3_BASE_URL").ok();
        let s3_presigned_base_url = std::env::var("ATLAS_STORE_S3_PRESIGNED_BASE_URL").ok();
        let s3_bearer = std::env::var("ATLAS_STORE_S3_BEARER").ok();
        let http_bearer = std::env::var("ATLAS_STORE_HTTP_BEARER").ok();
        let allow_private_hosts = env_bool("ATLAS_ALLOW_PRIVATE_STORE_HOSTS", false)?;
        if let Some(value) = s3_base_url.as_deref() {
            validate_url("ATLAS_STORE_S3_BASE_URL", value, true)?;
        }
        if let Some(value) = s3_presigned_base_url.as_deref() {
            validate_url("ATLAS_STORE_S3_PRESIGNED_BASE_URL", value, false)?;
        }
        if s3_enabled && s3_base_url.as_deref().is_none_or(str::is_empty) {
            return Err(RuntimeConfigError::MissingRequiredEnv {
                name: "ATLAS_STORE_S3_BASE_URL".to_string(),
                message: "ATLAS_STORE_S3_BASE_URL is required when S3 enabled".to_string(),
            });
        }
        let store = StoreConfig {
            mode: if !registry_sources.is_empty() {
                StoreMode::Federated
            } else if s3_enabled {
                StoreMode::S3
            } else {
                StoreMode::Local
            },
            local_root: startup.store_root.clone(),
            s3_base_url,
            s3_presigned_base_url,
            s3_bearer,
            http_bearer,
            allow_private_hosts,
            retry,
            registry_sources,
        };

        let runtime = Self {
            catalog_mode: if cache.cached_only_mode || !api.readiness_requires_catalog {
                CatalogMode::Optional
            } else {
                CatalogMode::Required
            },
            log_json: env_bool("ATLAS_LOG_JSON", true)?,
            log_level: std::env::var("ATLAS_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            log_filter_targets: std::env::var("ATLAS_LOG_FILTER_TARGETS").ok(),
            log_sampling_rate: env_f64("ATLAS_LOG_SAMPLING_RATE", 1.0)?,
            log_redaction_enabled: env_bool("ATLAS_LOG_REDACTION_ENABLED", true)?,
            log_rotation_max_bytes: env_u64("ATLAS_LOG_ROTATION_MAX_BYTES", 10_485_760)?,
            log_rotation_max_files: env_u64("ATLAS_LOG_ROTATION_MAX_FILES", 5)? as u32,
            otel_enabled: env_bool("ATLAS_OTEL_ENABLED", false)?,
            trace_sampling_ratio: env_f64("ATLAS_TRACE_SAMPLING_RATIO", 1.0)?,
            trace_exporter: std::env::var("ATLAS_TRACE_EXPORTER")
                .unwrap_or_else(|_| "otlp".to_string()),
            trace_otlp_endpoint: std::env::var("ATLAS_TRACE_OTLP_ENDPOINT").ok(),
            trace_jaeger_endpoint: std::env::var("ATLAS_TRACE_JAEGER_ENDPOINT").ok(),
            trace_file_path: std::env::var("ATLAS_TRACE_FILE_PATH").ok(),
            trace_service_name: std::env::var("ATLAS_TRACE_SERVICE_NAME")
                .unwrap_or_else(|_| "bijux-atlas-server".to_string()),
            trace_context_propagation_enabled: env_bool(
                "ATLAS_TRACE_CONTEXT_PROPAGATION_ENABLED",
                true,
            )?,
            warm_coordination_enabled: env_bool("ATLAS_WARM_COORDINATION_ENABLED", false)?,
            warm_coordination_lock_ttl_secs: env_u64("ATLAS_WARM_COORDINATION_LOCK_TTL_SECS", 300)?,
            warm_coordination_retry_budget: env_usize("ATLAS_WARM_COORDINATION_RETRY_BUDGET", 3)?,
            warm_coordination_retry_base_ms: env_u64("ATLAS_WARM_COORDINATION_RETRY_BASE_MS", 250)?,
            pod_id: std::env::var("HOSTNAME").unwrap_or_else(|_| "atlas-pod".to_string()),
            policy_mode: default_runtime_policy_mode(),
            shutdown_drain_ms: env_u64("ATLAS_SHUTDOWN_DRAIN_MS", 5000)?,
            tcp_keepalive_enabled: env_bool("ATLAS_TCP_KEEPALIVE_ENABLED", true)?,
            startup,
            api,
            cache,
            store,
            env_name,
        };
        validate_runtime_config_contract(&runtime)?;
        Ok(runtime)
    }
}

pub fn load_runtime_config(
    config_path: Option<&Path>,
    cli_bind_addr: Option<&str>,
    cli_store_root: Option<&Path>,
    cli_cache_root: Option<&Path>,
) -> Result<RuntimeConfig, RuntimeConfigError> {
    let startup =
        load_runtime_startup_config(config_path, cli_bind_addr, cli_store_root, cli_cache_root)
            .map_err(|message| RuntimeConfigError::InvalidValue { message })?;
    RuntimeConfig::from_env(startup)
}

#[cfg(test)]
mod tests;
