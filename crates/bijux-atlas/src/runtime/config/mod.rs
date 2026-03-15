// SPDX-License-Identifier: Apache-2.0

mod paths;
mod env;
mod settings;
#[cfg(test)]
mod tests;

pub use self::paths::{resolve_bijux_cache_dir, resolve_bijux_config_path};
pub use self::settings::{
    default_runtime_cache_root, default_runtime_policy_mode, default_runtime_pod_id,
    default_runtime_store_root,
    effective_config_payload, effective_runtime_config_payload, load_runtime_config,
    load_runtime_startup_config, runtime_build_hash, runtime_config_contract_snapshot,
    runtime_governance_version, runtime_release_id, runtime_startup_config_docs_markdown,
    runtime_startup_config_schema_json, validate_runtime_env_contract,
    validate_startup_config_contract, ApiConfig, AuditConfig, AuditSink, AuthMode, CatalogMode,
    CONFIG_SCHEMA_VERSION, RateLimitConfig, RuntimeConfig, RuntimeConfigError,
    RuntimeStartupConfig, StoreConfig, StoreMode,
};
pub(crate) use self::settings::resolve_runtime_path;
pub(crate) use self::settings::{
    DEFAULT_BIND_ADDR, DEFAULT_CACHE_ROOT, DEFAULT_STORE_ROOT,
};
