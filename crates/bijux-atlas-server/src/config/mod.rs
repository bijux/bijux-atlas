// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const CONFIG_SCHEMA_VERSION: &str = "1";

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
    pub otel_enabled: bool,
    pub warm_coordination_enabled: bool,
    pub warm_coordination_lock_ttl_secs: u64,
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

fn invalid_format(name: &str, value: String, message: String) -> RuntimeConfigError {
    RuntimeConfigError::InvalidFormat {
        name: name.to_string(),
        value,
        message,
    }
}

fn env_bool(name: &str, default: bool) -> Result<bool, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    match value.as_str() {
        "1" | "true" | "TRUE" | "yes" | "YES" => Ok(true),
        "0" | "false" | "FALSE" | "no" | "NO" => Ok(false),
        _ => Err(invalid_format(
            name,
            value.clone(),
            format!(
                "invalid boolean value for {name}: {value} (expected one of 1/0/true/false/yes/no)"
            ),
        )),
    }
}

fn env_u64(name: &str, default: u64) -> Result<u64, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    value.parse::<u64>().map_err(|err| {
        invalid_format(
            name,
            value.clone(),
            format!("invalid u64 value for {name}: {value} ({err})"),
        )
    })
}

fn env_usize(name: &str, default: usize) -> Result<usize, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    value.parse::<usize>().map_err(|err| {
        invalid_format(
            name,
            value.clone(),
            format!("invalid usize value for {name}: {value} ({err})"),
        )
    })
}

fn env_f64(name: &str, default: f64) -> Result<f64, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    value.parse::<f64>().map_err(|err| {
        invalid_format(
            name,
            value.clone(),
            format!("invalid f64 value for {name}: {value} ({err})"),
        )
    })
}

fn env_duration_ms(name: &str, default_ms: u64) -> Result<Duration, RuntimeConfigError> {
    Ok(Duration::from_millis(env_u64(name, default_ms)?))
}

fn env_list(name: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn env_dataset_list(name: &str) -> Result<Vec<bijux_atlas_model::DatasetId>, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(Vec::new());
    };
    let mut datasets = Vec::new();
    for item in value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let parts: Vec<_> = item.split('/').collect();
        if parts.len() != 3 {
            return Err(invalid_format(
                name,
                value.clone(),
                format!(
                    "invalid dataset list entry for {name}: {item} (expected release/species/assembly)"
                ),
            ));
        }
        let dataset =
            bijux_atlas_model::DatasetId::new(parts[0], parts[1], parts[2]).map_err(|err| {
                invalid_format(
                    name,
                    value.clone(),
                    format!("invalid dataset list entry for {name}: {item} ({err})"),
                )
            })?;
        datasets.push(dataset);
    }
    Ok(datasets)
}

fn env_map(name: &str) -> Result<HashMap<String, String>, RuntimeConfigError> {
    let Some(raw) = std::env::var(name).ok() else {
        return Ok(HashMap::new());
    };
    let mut entries = HashMap::new();
    for item in raw
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let (key, value) = item.split_once('=').ok_or_else(|| {
            invalid_format(
                name,
                raw.clone(),
                format!("invalid key=value entry for {name}: {item}"),
            )
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(invalid_format(
                name,
                raw.clone(),
                format!("invalid key=value entry for {name}: {item}"),
            ));
        }
        entries.insert(key.to_string(), value.to_string());
    }
    Ok(entries)
}

fn validate_url(name: &str, value: &str, required: bool) -> Result<(), RuntimeConfigError> {
    if value.trim().is_empty() {
        if required {
            return Err(RuntimeConfigError::MissingRequiredEnv {
                name: name.to_string(),
                message: format!("{name} must not be empty"),
            });
        }
        return Ok(());
    }
    reqwest::Url::parse(value).map_err(|err| {
        invalid_format(
            name,
            value.to_string(),
            format!("invalid url value for {name}: {value} ({err})"),
        )
    })?;
    Ok(())
}

fn parse_registry_source_specs(
    retry: &StoreRetryConfig,
) -> Result<Vec<RegistrySourceSpec>, RuntimeConfigError> {
    let raw = std::env::var("ATLAS_REGISTRY_SOURCES").unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    let signatures = env_map("ATLAS_REGISTRY_SIGNATURES")?;
    let ttl = env_u64("ATLAS_REGISTRY_TTL_MS", 15_000)?;
    let max_sources = env_usize("ATLAS_REGISTRY_MAX_SOURCES", 8)?;
    let mut sources = Vec::new();
    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let (name, spec) = piece.split_once('=').ok_or_else(|| {
            invalid_format(
                "ATLAS_REGISTRY_SOURCES",
                raw.clone(),
                format!("invalid ATLAS_REGISTRY_SOURCES entry: {piece}"),
            )
        })?;
        let name = name.trim();
        let spec = spec.trim();
        let (scheme, endpoint) = if let Some(path) = spec.strip_prefix("local:") {
            ("local".to_string(), path.to_string())
        } else if let Some(url) = spec.strip_prefix("s3:") {
            validate_url("ATLAS_REGISTRY_SOURCES", url, true)?;
            ("s3".to_string(), url.to_string())
        } else if let Some(url) = spec.strip_prefix("http:") {
            validate_url("ATLAS_REGISTRY_SOURCES", url, true)?;
            ("http".to_string(), url.to_string())
        } else {
            return Err(RuntimeConfigError::InvalidValue {
                message: format!(
                    "unsupported registry source scheme in {piece}; use local:/path, s3:https://..., or http:https://..."
                ),
            });
        };
        sources.push(RegistrySourceSpec {
            name: name.to_string(),
            scheme,
            endpoint,
            signature: signatures.get(name).cloned(),
            ttl_ms: ttl,
        });
    }
    if sources.len() > max_sources {
        return Err(RuntimeConfigError::InvalidValue {
            message: format!(
                "ATLAS_REGISTRY_SOURCES exceeds max allowed sources: {} > {}",
                sources.len(),
                max_sources
            ),
        });
    }
    let priority = std::env::var("ATLAS_REGISTRY_PRIORITY").unwrap_or_default();
    if !priority.trim().is_empty() {
        let mut by_name: HashMap<String, RegistrySourceSpec> = sources
            .into_iter()
            .map(|row| (row.name.clone(), row))
            .collect();
        let mut ordered = Vec::new();
        for name in priority.split(',').map(str::trim).filter(|x| !x.is_empty()) {
            if let Some(row) = by_name.remove(name) {
                ordered.push(row);
            }
        }
        let mut rest: Vec<RegistrySourceSpec> = by_name.into_values().collect();
        rest.sort_by(|a, b| a.name.cmp(&b.name));
        ordered.extend(rest);
        sources = ordered;
    }
    for row in &sources {
        if row.scheme != "local" {
            validate_url("ATLAS_REGISTRY_SOURCES", &row.endpoint, true)?;
        }
    }
    let _ = retry;
    Ok(sources)
}

pub fn validate_runtime_env_contract() -> Result<(), RuntimeConfigError> {
    let raw = include_str!("../../../../configs/contracts/env.schema.json");
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
    pub enable_audit_log: bool,
    pub require_api_key: bool,
    pub allowed_api_keys: Vec<String>,
    pub hmac_secret: Option<String>,
    pub hmac_required: bool,
    pub hmac_max_skew_secs: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
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
            enable_audit_log: false,
            require_api_key: false,
            allowed_api_keys: Vec::new(),
            hmac_secret: None,
            hmac_required: false,
            hmac_max_skew_secs: 300,
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

pub fn runtime_startup_config_schema_json() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://bijux.dev/schemas/runtime-startup-config.schema.json",
        "title": "RuntimeStartupConfig",
        "description": "Runtime startup configuration resolved from CLI, env, config file, then defaults.",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "bind_addr": {
                "type": "string",
                "description": "Socket bind address in host:port form.",
                "default": DEFAULT_BIND_ADDR
            },
            "store_root": {
                "type": "string",
                "description": "Directory for local runtime store artifacts.",
                "default": DEFAULT_STORE_ROOT
            },
            "cache_root": {
                "type": "string",
                "description": "Directory for local runtime cache artifacts.",
                "default": DEFAULT_CACHE_ROOT
            }
        },
        "required": ["bind_addr", "store_root", "cache_root"],
        "x-resolution-order": ["cli", "env", "config_file", "default"]
    })
}

pub fn runtime_startup_config_docs_markdown() -> String {
    format!(
        "# Runtime Startup Config\n\n\
Source of truth for startup config resolution used by `atlas-server`.\n\n\
Resolution precedence: `CLI > ENV > config file > defaults`.\n\n\
| Field | CLI Flag | ENV | Config Key | Default |\n\
|---|---|---|---|---|\n\
| `bind_addr` | `--bind` | `ATLAS_BIND` | `bind_addr` | `{}` |\n\
| `store_root` | `--store-root` | `ATLAS_STORE_ROOT` | `store_root` | `{}` |\n\
| `cache_root` | `--cache-root` | `ATLAS_CACHE_ROOT` | `cache_root` | `{}` |\n\n\
File formats: `.json`, `.yaml`/`.yml`, `.toml`.\n\
Validation: all resolved fields are required and must be non-empty.\n",
        DEFAULT_BIND_ADDR, DEFAULT_STORE_ROOT, DEFAULT_CACHE_ROOT
    )
}

pub fn effective_config_payload(
    startup: &RuntimeStartupConfig,
    api: &ApiConfig,
    cache: &crate::DatasetCacheConfig,
) -> Result<serde_json::Value, String> {
    let mut api_json =
        serde_json::to_value(api).map_err(|err| format!("serialize api config: {err}"))?;
    if let Some(obj) = api_json.as_object_mut() {
        if obj.contains_key("redis_url") {
            obj.insert("redis_url".to_string(), serde_json::json!("<redacted>"));
        }
        if obj.contains_key("allowed_api_keys") {
            obj.insert(
                "allowed_api_keys".to_string(),
                serde_json::json!(["<redacted>"]),
            );
        }
        if obj.contains_key("hmac_secret") {
            obj.insert("hmac_secret".to_string(), serde_json::json!("<redacted>"));
        }
    }
    let startup_json =
        serde_json::to_value(startup).map_err(|err| format!("serialize startup config: {err}"))?;
    let cache_json =
        serde_json::to_value(cache).map_err(|err| format!("serialize cache config: {err}"))?;
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "atlas_server_effective_config_v1",
        "startup": startup_json,
        "api": api_json,
        "cache": cache_json
    }))
}

fn redact_known_secrets(config_json: &mut serde_json::Value) {
    let Some(obj) = config_json.as_object_mut() else {
        return;
    };
    const SECRET_FIELD_DENYLIST: &[&str] = &[
        "redis_url",
        "allowed_api_keys",
        "hmac_secret",
        "s3_bearer",
        "http_bearer",
    ];
    for &key in SECRET_FIELD_DENYLIST {
        if obj.contains_key(key) {
            let value = if key == "allowed_api_keys" {
                serde_json::json!(["<redacted>"])
            } else {
                serde_json::json!("<redacted>")
            };
            obj.insert(key.to_string(), value);
        }
    }
}

pub fn effective_runtime_config_payload(
    runtime: &RuntimeConfig,
) -> Result<serde_json::Value, String> {
    let mut payload = serde_json::json!({
        "schema_version": 1,
        "kind": "atlas_server_effective_config_v2",
        "env_name": runtime.env_name,
        "catalog_mode": runtime.catalog_mode,
        "startup": runtime.startup,
        "api": runtime.api,
        "cache": runtime.cache,
        "store": runtime.store,
        "runtime": {
            "log_json": runtime.log_json,
            "otel_enabled": runtime.otel_enabled,
            "warm_coordination_enabled": runtime.warm_coordination_enabled,
            "warm_coordination_lock_ttl_secs": runtime.warm_coordination_lock_ttl_secs,
            "pod_id": runtime.pod_id,
            "policy_mode": runtime.policy_mode,
            "shutdown_drain_ms": runtime.shutdown_drain_ms,
            "tcp_keepalive_enabled": runtime.tcp_keepalive_enabled
        }
    });
    if let Some(api_json) = payload.get_mut("api") {
        redact_known_secrets(api_json);
    }
    if let Some(store_json) = payload.get_mut("store") {
        redact_known_secrets(store_json);
    }
    Ok(payload)
}

pub fn runtime_config_contract_snapshot() -> Result<serde_json::Value, String> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let env_schema_path = repo_root.join("configs/contracts/env.schema.json");
    let env_schema_text = std::fs::read_to_string(&env_schema_path)
        .map_err(|err| format!("read {}: {err}", env_schema_path.display()))?;
    let env_schema_json: serde_json::Value = serde_json::from_str(&env_schema_text)
        .map_err(|err| format!("parse {}: {err}", env_schema_path.display()))?;
    let mut allowlisted_env = env_schema_json
        .get("allowed_env")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            format!(
                "{} must contain an allowed_env array",
                env_schema_path.display()
            )
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "allowed_env entries must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    allowlisted_env.sort();
    allowlisted_env.dedup();
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "atlas_runtime_config_contract_snapshot_v1",
        "env_schema_path": "configs/contracts/env.schema.json",
        "docs_path": "docs/reference/runtime/config.md",
        "allowlisted_env": allowlisted_env
    }))
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

        let api = ApiConfig {
            max_body_bytes: env_usize("ATLAS_MAX_BODY_BYTES", 16 * 1024)?,
            max_uri_bytes: env_usize("ATLAS_MAX_URI_BYTES", 2048)?,
            max_header_bytes: env_usize("ATLAS_MAX_HEADER_BYTES", 16 * 1024)?,
            request_timeout: env_duration_ms("ATLAS_REQUEST_TIMEOUT_MS", 5000)?,
            sql_timeout: env_duration_ms("ATLAS_SQL_TIMEOUT_MS", 800)?,
            response_max_bytes: env_usize("ATLAS_RESPONSE_MAX_BYTES", 512 * 1024)?,
            slow_query_threshold: env_duration_ms("ATLAS_SLOW_QUERY_THRESHOLD_MS", 200)?,
            enable_debug_datasets: env_bool("ATLAS_ENABLE_DEBUG_DATASETS", false)?,
            enable_exemplars: env_bool("ATLAS_ENABLE_EXEMPLARS", false)?,
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
            enable_audit_log: env_bool("ATLAS_ENABLE_AUDIT_LOG", false)?,
            require_api_key: env_bool("ATLAS_REQUIRE_API_KEY", false)?,
            allowed_api_keys: env_list("ATLAS_ALLOWED_API_KEYS"),
            hmac_secret: std::env::var("ATLAS_HMAC_SECRET")
                .ok()
                .filter(|x| !x.is_empty()),
            hmac_required: env_bool("ATLAS_HMAC_REQUIRED", false)?,
            hmac_max_skew_secs: env_u64("ATLAS_HMAC_MAX_SKEW_SECS", 300)?,
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
            otel_enabled: env_bool("ATLAS_OTEL_ENABLED", false)?,
            warm_coordination_enabled: env_bool("ATLAS_WARM_COORDINATION_ENABLED", false)?,
            warm_coordination_lock_ttl_secs: env_u64("ATLAS_WARM_COORDINATION_LOCK_TTL_SECS", 300)?,
            pod_id: std::env::var("HOSTNAME").unwrap_or_else(|_| "atlas-pod".to_string()),
            policy_mode: std::env::var("ATLAS_POLICY_MODE")
                .unwrap_or_else(|_| "strict".to_string()),
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
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_runtime_env() {
        let names: Vec<String> = std::env::vars()
            .map(|(name, _)| name)
            .filter(|name| name.starts_with("ATLAS_") || name.starts_with("BIJUX_"))
            .collect();
        for name in names {
            std::env::remove_var(name);
        }
    }

    fn with_runtime_env<F>(pairs: &[(&str, &str)], test: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().expect("env lock");
        clear_runtime_env();
        for (name, value) in pairs {
            std::env::set_var(name, value);
        }
        test();
        clear_runtime_env();
    }

    #[test]
    fn startup_config_validation_rejects_invalid_watermark_order() {
        let api = ApiConfig::default();
        let cache = crate::DatasetCacheConfig {
            disk_high_watermark_pct: 70,
            disk_low_watermark_pct: 75,
            ..crate::DatasetCacheConfig::default()
        };
        let err = validate_startup_config_contract(&api, &cache).expect_err("invalid watermarks");
        assert!(err.contains("high > low"));
    }

    #[test]
    fn startup_config_validation_enforces_auth_contracts() {
        let mut api = ApiConfig {
            require_api_key: true,
            ..ApiConfig::default()
        };
        let cache = crate::DatasetCacheConfig::default();
        let err = validate_startup_config_contract(&api, &cache).expect_err("missing keys");
        assert!(err.contains("allowed api key"));

        api.allowed_api_keys = vec!["k".to_string()];
        api.hmac_required = true;
        api.hmac_secret = None;
        let err = validate_startup_config_contract(&api, &cache).expect_err("missing hmac");
        assert!(err.contains("hmac_secret"));
    }

    #[test]
    fn runtime_startup_config_cli_overrides_env_and_file() {
        let resolved = resolve_runtime_startup_config(
            RuntimeStartupConfigFile {
                bind_addr: Some("127.0.0.1:9000".to_string()),
                store_root: Some(PathBuf::from("from-file-store")),
                cache_root: Some(PathBuf::from("from-file-cache")),
            },
            Some("127.0.0.1:9200"),
            Some(Path::new("from-cli-store")),
            Some(Path::new("from-cli-cache")),
            Some("127.0.0.1:9100".to_string()),
            Some(PathBuf::from("from-env-store")),
            Some(PathBuf::from("from-env-cache")),
        )
        .expect("load");
        assert_eq!(resolved.bind_addr, "127.0.0.1:9200");
        assert_eq!(resolved.store_root, PathBuf::from("from-cli-store"));
        assert_eq!(resolved.cache_root, PathBuf::from("from-cli-cache"));
    }

    #[test]
    fn runtime_startup_config_env_overrides_file() {
        let resolved = resolve_runtime_startup_config(
            RuntimeStartupConfigFile {
                bind_addr: Some("127.0.0.1:9000".to_string()),
                store_root: Some(PathBuf::from("from-file-store")),
                cache_root: Some(PathBuf::from("from-file-cache")),
            },
            None,
            None,
            None,
            Some("127.0.0.1:9100".to_string()),
            Some(PathBuf::from("from-env-store")),
            Some(PathBuf::from("from-env-cache")),
        )
        .expect("load");
        assert_eq!(resolved.bind_addr, "127.0.0.1:9100");
        assert_eq!(resolved.store_root, PathBuf::from("from-env-store"));
        assert_eq!(resolved.cache_root, PathBuf::from("from-env-cache"));
    }

    #[test]
    fn runtime_startup_config_uses_defaults_without_sources() {
        let resolved = resolve_runtime_startup_config(
            RuntimeStartupConfigFile::default(),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("load");
        assert_eq!(resolved.bind_addr, DEFAULT_BIND_ADDR);
        assert_eq!(resolved.store_root, PathBuf::from(DEFAULT_STORE_ROOT));
        assert_eq!(resolved.cache_root, PathBuf::from(DEFAULT_CACHE_ROOT));
    }

    #[test]
    fn runtime_startup_config_contract_artifacts_match_generated() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let schema_path = root.join("docs/generated/runtime-startup-config.schema.json");
        let docs_path = root.join("docs/generated/runtime-startup-config.md");
        let expected_schema = std::fs::read_to_string(schema_path).expect("schema file");
        let expected_docs = std::fs::read_to_string(docs_path).expect("docs file");

        let generated_schema = runtime_startup_config_schema_json();
        let expected_schema: serde_json::Value =
            serde_json::from_str(&expected_schema).expect("parse schema file");
        let generated_docs = runtime_startup_config_docs_markdown();

        assert_eq!(
            generated_schema, expected_schema,
            "runtime startup config schema drift; regenerate docs/generated/runtime-startup-config.schema.json"
        );
        assert_eq!(
            generated_docs, expected_docs,
            "runtime startup config docs drift; regenerate docs/generated/runtime-startup-config.md"
        );
    }

    #[test]
    fn effective_config_snapshot_matches_generated() {
        let startup = RuntimeStartupConfig {
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            store_root: PathBuf::from(DEFAULT_STORE_ROOT),
            cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
        };
        let payload = effective_config_payload(
            &startup,
            &ApiConfig::default(),
            &crate::DatasetCacheConfig::default(),
        )
        .expect("effective config payload");

        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let snapshot_path = root.join("docs/generated/effective-config.snapshot.json");
        let expected: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&snapshot_path).expect("read effective config snapshot"),
        )
        .expect("parse effective config snapshot");
        assert_eq!(
            payload, expected,
            "effective config snapshot drift; regenerate docs/generated/effective-config.snapshot.json"
        );
    }

    #[test]
    fn runtime_config_rejects_cached_only_with_catalog_required() {
        with_runtime_env(
            &[
                ("ATLAS_CACHED_ONLY_MODE", "true"),
                ("ATLAS_READINESS_REQUIRES_CATALOG", "true"),
            ],
            || {
                let startup = RuntimeStartupConfig {
                    bind_addr: DEFAULT_BIND_ADDR.to_string(),
                    store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                    cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
                };
                let err = RuntimeConfig::from_env(startup).expect_err("invalid invariant");
                assert_eq!(
                    err.to_string(),
                    "ATLAS_CACHED_ONLY_MODE=true requires ATLAS_READINESS_REQUIRES_CATALOG=false"
                );
            },
        );
    }

    #[test]
    fn runtime_config_accepts_valid_cached_only_mode() {
        with_runtime_env(
            &[
                ("ATLAS_CACHED_ONLY_MODE", "true"),
                ("ATLAS_READINESS_REQUIRES_CATALOG", "false"),
            ],
            || {
                let startup = RuntimeStartupConfig {
                    bind_addr: DEFAULT_BIND_ADDR.to_string(),
                    store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                    cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
                };
                let runtime = RuntimeConfig::from_env(startup).expect("cached-only config");
                assert!(runtime.cache.cached_only_mode);
                assert!(matches!(runtime.catalog_mode, CatalogMode::Optional));
            },
        );
    }

    #[test]
    fn runtime_config_accepts_catalog_required_mode() {
        with_runtime_env(&[("ATLAS_READINESS_REQUIRES_CATALOG", "true")], || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let runtime = RuntimeConfig::from_env(startup).expect("catalog-required config");
            assert!(!runtime.cache.cached_only_mode);
            assert!(matches!(runtime.catalog_mode, CatalogMode::Required));
        });
    }

    #[test]
    fn effective_runtime_config_redacts_secrets() {
        with_runtime_env(
            &[
                ("ATLAS_HMAC_SECRET", "secret-material"),
                ("ATLAS_ALLOWED_API_KEYS", "alpha,beta"),
                ("ATLAS_STORE_S3_ENABLED", "true"),
                ("ATLAS_STORE_S3_BASE_URL", "https://example.invalid/store"),
                ("ATLAS_STORE_S3_BEARER", "token"),
            ],
            || {
                let startup = RuntimeStartupConfig {
                    bind_addr: DEFAULT_BIND_ADDR.to_string(),
                    store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                    cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
                };
                let runtime = RuntimeConfig::from_env(startup).expect("runtime");
                let payload = effective_runtime_config_payload(&runtime).expect("payload");
                assert_eq!(
                    payload["api"]["hmac_secret"],
                    serde_json::json!("<redacted>")
                );
                assert_eq!(
                    payload["api"]["allowed_api_keys"],
                    serde_json::json!(["<redacted>"])
                );
                assert_eq!(
                    payload["store"]["s3_bearer"],
                    serde_json::json!("<redacted>")
                );
            },
        );
    }

    #[test]
    fn runtime_config_contract_snapshot_points_to_the_allowlist_source() {
        let snapshot = runtime_config_contract_snapshot().expect("contract snapshot");
        assert_eq!(
            snapshot["env_schema_path"],
            serde_json::json!("configs/contracts/env.schema.json")
        );
        assert_eq!(
            snapshot["docs_path"],
            serde_json::json!("docs/reference/runtime/config.md")
        );
        let allowlisted_env = snapshot["allowlisted_env"]
            .as_array()
            .expect("allowlisted_env array");
        assert!(
            allowlisted_env
                .iter()
                .any(|value| value == "ATLAS_STORE_S3_BASE_URL"),
            "snapshot must include canonical runtime env keys"
        );
    }
}
