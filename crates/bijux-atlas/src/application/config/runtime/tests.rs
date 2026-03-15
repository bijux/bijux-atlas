// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::*;

fn generated_docs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("docs")
        .join("bijux-atlas-crate")
        .join("server")
        .join("generated")
}

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
    api.auth_mode = AuthMode::ApiKey;
    api.hmac_required = true;
    api.hmac_secret = None;
    let err = validate_startup_config_contract(&api, &cache).expect_err("conflicting auth");
    assert!(err.contains("cannot both"));

    api.hmac_required = false;
    api.auth_mode = AuthMode::Mtls;
    api.require_api_key = false;
    api.hmac_required = true;
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
    let generated_docs = generated_docs_dir();
    let schema_path = generated_docs.join("runtime-startup-config.schema.json");
    let docs_path = generated_docs.join("runtime-startup-config.md");
    let expected_schema = std::fs::read_to_string(schema_path).expect("schema file");
    let expected_docs = std::fs::read_to_string(docs_path).expect("docs file");

    let generated_schema = runtime_startup_config_schema_json();
    let expected_schema: serde_json::Value =
        serde_json::from_str(&expected_schema).expect("parse schema file");
    let generated_docs = runtime_startup_config_docs_markdown();

    assert_eq!(
        generated_schema, expected_schema,
        "runtime startup config schema drift; regenerate docs/bijux-atlas-crate/server/generated/runtime-startup-config.schema.json"
    );
    assert_eq!(
        generated_docs, expected_docs,
        "runtime startup config docs drift; regenerate docs/bijux-atlas-crate/server/generated/runtime-startup-config.md"
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

    let snapshot_path = generated_docs_dir().join("effective-config.snapshot.json");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        let encoded =
            serde_json::to_string_pretty(&payload).expect("encode effective config snapshot");
        std::fs::write(&snapshot_path, format!("{encoded}\n"))
            .expect("write effective config snapshot");
        return;
    }
    let expected: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&snapshot_path).expect("read effective config snapshot"),
    )
    .expect("parse effective config snapshot");
    assert_eq!(
        payload, expected,
        "effective config snapshot drift; regenerate docs/bijux-atlas-crate/server/generated/effective-config.snapshot.json"
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
fn runtime_config_enforces_warm_coordination_retry_contract() {
    with_runtime_env(
        &[
            ("ATLAS_WARM_COORDINATION_ENABLED", "true"),
            ("ATLAS_WARM_COORDINATION_RETRY_BUDGET", "0"),
        ],
        || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let err = RuntimeConfig::from_env(startup).expect_err("invalid retry budget");
            assert!(err
                .to_string()
                .contains("ATLAS_WARM_COORDINATION_RETRY_BUDGET>0"));
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
fn runtime_config_accepts_trace_exporter_matrix() {
    for exporter in ["otlp", "jaeger", "file", "none"] {
        with_runtime_env(&[("ATLAS_TRACE_EXPORTER", exporter)], || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let runtime = RuntimeConfig::from_env(startup).expect("valid trace exporter");
            assert_eq!(runtime.trace_exporter, exporter);
        });
    }
}

#[test]
fn runtime_config_rejects_unknown_trace_exporter() {
    with_runtime_env(&[("ATLAS_TRACE_EXPORTER", "zipkin")], || {
        let startup = RuntimeStartupConfig {
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            store_root: PathBuf::from(DEFAULT_STORE_ROOT),
            cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
        };
        let err = RuntimeConfig::from_env(startup).expect_err("invalid trace exporter");
        assert!(err
            .to_string()
            .contains("ATLAS_TRACE_EXPORTER must be one of: otlp, jaeger, file, none"));
    });
}

#[test]
fn runtime_config_accepts_logging_configuration() {
    with_runtime_env(
        &[
            ("ATLAS_LOG_LEVEL", "debug"),
            ("ATLAS_LOG_FILTER_TARGETS", "atlas=debug,hyper=warn"),
            ("ATLAS_LOG_SAMPLING_RATE", "0.75"),
            ("ATLAS_LOG_REDACTION_ENABLED", "false"),
        ],
        || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let runtime = RuntimeConfig::from_env(startup).expect("valid logging config");
            assert_eq!(runtime.log_level, "debug");
            assert_eq!(
                runtime.log_filter_targets.as_deref(),
                Some("atlas=debug,hyper=warn")
            );
            assert_eq!(runtime.log_sampling_rate, 0.75);
            assert!(!runtime.log_redaction_enabled);
            assert_eq!(runtime.log_rotation_max_bytes, 10_485_760);
            assert_eq!(runtime.log_rotation_max_files, 5);
        },
    );
}

#[test]
fn runtime_config_rejects_invalid_log_level_and_sampling() {
    with_runtime_env(
        &[
            ("ATLAS_LOG_LEVEL", "verbose"),
            ("ATLAS_LOG_SAMPLING_RATE", "1.5"),
        ],
        || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let err = RuntimeConfig::from_env(startup).expect_err("invalid logging config");
            let msg = err.to_string();
            assert!(
                msg.contains("ATLAS_LOG_LEVEL must be one of")
                    || msg.contains("ATLAS_LOG_SAMPLING_RATE must be in [0.0, 1.0]")
            );
        },
    );
}

#[test]
fn runtime_config_rejects_invalid_log_rotation_limits() {
    with_runtime_env(
        &[
            ("ATLAS_LOG_ROTATION_MAX_BYTES", "0"),
            ("ATLAS_LOG_ROTATION_MAX_FILES", "0"),
        ],
        || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let err = RuntimeConfig::from_env(startup).expect_err("invalid log rotation config");
            let msg = err.to_string();
            assert!(
                msg.contains("ATLAS_LOG_ROTATION_MAX_BYTES must be > 0")
                    || msg.contains("ATLAS_LOG_ROTATION_MAX_FILES must be > 0")
            );
        },
    );
}

#[test]
fn runtime_config_rejects_invalid_auth_mode() {
    with_runtime_env(&[("ATLAS_AUTH_MODE", "hmac")], || {
        let startup = RuntimeStartupConfig {
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            store_root: PathBuf::from(DEFAULT_STORE_ROOT),
            cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
        };
        let err = RuntimeConfig::from_env(startup).expect_err("invalid auth mode");
        assert!(err
            .to_string()
            .contains("ATLAS_AUTH_MODE must be one of: disabled, api-key, token, oidc, mtls"));
    });
}

#[test]
fn runtime_config_accepts_explicit_api_key_auth_mode() {
    with_runtime_env(
        &[
            ("ATLAS_AUTH_MODE", "api-key"),
            ("ATLAS_ALLOWED_API_KEYS", "alpha,beta"),
        ],
        || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let runtime = RuntimeConfig::from_env(startup).expect("api key auth mode");
            assert_eq!(runtime.api.auth_mode, AuthMode::ApiKey);
            assert!(runtime.api.require_api_key);
            assert!(!runtime.api.hmac_required);
        },
    );
}

#[test]
fn runtime_config_accepts_proxy_verified_auth_modes() {
    for auth_mode in ["oidc", "mtls"] {
        with_runtime_env(&[("ATLAS_AUTH_MODE", auth_mode)], || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let runtime = RuntimeConfig::from_env(startup).expect("proxy auth mode");
            assert_eq!(runtime.api.auth_mode.as_str(), auth_mode);
        });
    }
}

#[test]
fn runtime_config_accepts_token_auth_mode() {
    with_runtime_env(
        &[
            ("ATLAS_AUTH_MODE", "token"),
            ("ATLAS_TOKEN_SIGNING_SECRET", "secret"),
            ("ATLAS_TOKEN_REQUIRED_ISSUER", "atlas-auth"),
            ("ATLAS_TOKEN_REQUIRED_AUDIENCE", "atlas-api"),
        ],
        || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let runtime = RuntimeConfig::from_env(startup).expect("token auth mode");
            assert_eq!(runtime.api.auth_mode, AuthMode::Token);
            assert!(!runtime.api.require_api_key);
        },
    );
}

#[test]
fn runtime_config_accepts_require_https_flag() {
    with_runtime_env(&[("ATLAS_REQUIRE_HTTPS", "true")], || {
        let startup = RuntimeStartupConfig {
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            store_root: PathBuf::from(DEFAULT_STORE_ROOT),
            cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
        };
        let runtime = RuntimeConfig::from_env(startup).expect("runtime");
        assert!(runtime.api.require_https);
    });
}

#[test]
fn runtime_config_admin_endpoints_are_disabled_by_default() {
    with_runtime_env(&[], || {
        let startup = RuntimeStartupConfig {
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            store_root: PathBuf::from(DEFAULT_STORE_ROOT),
            cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
        };
        let runtime = RuntimeConfig::from_env(startup).expect("default runtime");
        assert!(!runtime.api.enable_admin_endpoints);
    });
}

#[test]
fn runtime_config_accepts_explicit_audit_settings() {
    with_runtime_env(
        &[
            ("ATLAS_AUDIT_ENABLED", "true"),
            ("ATLAS_AUDIT_SINK", "otel"),
        ],
        || {
            let startup = RuntimeStartupConfig {
                bind_addr: DEFAULT_BIND_ADDR.to_string(),
                store_root: PathBuf::from(DEFAULT_STORE_ROOT),
                cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
            };
            let runtime = RuntimeConfig::from_env(startup).expect("audit runtime config");
            assert!(runtime.api.audit.enabled);
            assert_eq!(runtime.api.audit.sink.as_str(), "otel");
        },
    );
}

#[test]
fn runtime_config_rejects_invalid_audit_sink() {
    with_runtime_env(&[("ATLAS_AUDIT_SINK", "syslog")], || {
        let startup = RuntimeStartupConfig {
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            store_root: PathBuf::from(DEFAULT_STORE_ROOT),
            cache_root: PathBuf::from(DEFAULT_CACHE_ROOT),
        };
        let err = RuntimeConfig::from_env(startup).expect_err("invalid audit sink");
        assert!(err
            .to_string()
            .contains("ATLAS_AUDIT_SINK must be one of: stdout, file, otel"));
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
