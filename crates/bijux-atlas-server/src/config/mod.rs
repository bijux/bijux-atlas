// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const CONFIG_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitConfig {
    pub capacity: f64,
    pub refill_per_sec: f64,
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
            obj.insert("allowed_api_keys".to_string(), serde_json::json!(["<redacted>"]));
        }
        if obj.contains_key("hmac_secret") {
            obj.insert("hmac_secret".to_string(), serde_json::json!("<redacted>"));
        }
    }
    let startup_json = serde_json::to_value(startup)
        .map_err(|err| format!("serialize startup config: {err}"))?;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
