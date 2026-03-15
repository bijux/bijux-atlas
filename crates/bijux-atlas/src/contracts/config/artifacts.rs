// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use crate::runtime::config::{
    ApiConfig, RuntimeConfig, RuntimeStartupConfig, DEFAULT_BIND_ADDR, DEFAULT_CACHE_ROOT,
    DEFAULT_STORE_ROOT,
};

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
Source of truth for startup config resolution used by `bijux-atlas-server`.\n\n\
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
        if obj.contains_key("token_signing_secret") {
            obj.insert(
                "token_signing_secret".to_string(),
                serde_json::json!("<redacted>"),
            );
        }
    }
    let startup_json =
        serde_json::to_value(startup).map_err(|err| format!("serialize startup config: {err}"))?;
    let mut cache_json =
        serde_json::to_value(cache).map_err(|err| format!("serialize cache config: {err}"))?;
    if let Some(obj) = cache_json.as_object_mut() {
        obj.insert(
            "disk_root".to_string(),
            serde_json::Value::String(startup.cache_root.display().to_string()),
        );
    }
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
        "token_signing_secret",
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
            "warm_coordination_retry_budget": runtime.warm_coordination_retry_budget,
            "warm_coordination_retry_base_ms": runtime.warm_coordination_retry_base_ms,
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
    let env_schema_path = repo_root.join("configs/schemas/contracts/env.schema.json");
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
        "env_schema_path": "configs/schemas/contracts/env.schema.json",
        "docs_path": "docs/reference/runtime/config.md",
        "allowlisted_env": allowlisted_env
    }))
}
