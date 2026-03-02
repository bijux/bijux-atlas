// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn sanitized_server_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas-server"));
    command.current_dir(repo_root());
    for (name, _) in std::env::vars() {
        if name.starts_with("ATLAS_") || name.starts_with("BIJUX_") {
            command.env_remove(name);
        }
    }
    command
}

#[test]
fn startup_rejects_unknown_prefixed_env_before_binding_a_port() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_UNKNOWN_CONTRACT_BREAKER", "1")
        .output()
        .expect("run atlas-server");

    assert!(!output.status.success(), "unknown env must fail startup");
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        stderr.contains(
            "unknown env vars rejected by contract; set ATLAS_DEV_ALLOW_UNKNOWN_ENV=1 only for local dev override: ATLAS_UNKNOWN_CONTRACT_BREAKER"
        ),
        "stderr must contain the explicit contract failure:\n{stderr}"
    );
    assert!(
        !stderr.contains("atlas-server listening on"),
        "unknown env validation must fail before binding a port:\n{stderr}"
    );
    assert!(
        !stderr.contains("bind failed:"),
        "unknown env validation must fail before any bind attempt:\n{stderr}"
    );
}

#[test]
fn startup_accepts_an_allowlisted_env_surface() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_ENV", "dev")
        .env("ATLAS_LOG_JSON", "false")
        .env("ATLAS_REQUEST_TIMEOUT_MS", "5000")
        .output()
        .expect("run atlas-server");

    assert!(
        output.status.success(),
        "allowlisted env keys must pass startup validation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        !stderr.contains("unknown env vars rejected by contract"),
        "allowlisted startup env must not trip unknown-env validation:\n{stderr}"
    );
}

#[test]
fn startup_accepts_the_runtime_env_surface_emitted_by_the_helm_chart() {
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_STORE_S3_ENABLED", "true")
        .env("ATLAS_STORE_S3_BASE_URL", "https://example.invalid/store")
        .env(
            "ATLAS_STORE_S3_PRESIGNED_BASE_URL",
            "https://example.invalid/presigned",
        )
        .env("ATLAS_MAX_DATASET_COUNT", "64")
        .env("ATLAS_MAX_DISK_BYTES", "1073741824")
        .env("ATLAS_PINNED_DATASETS", "")
        .env("ATLAS_MAX_CONCURRENT_DOWNLOADS", "4")
        .env("ATLAS_STARTUP_WARMUP_JITTER_MAX_MS", "1000")
        .env("ATLAS_REQUEST_TIMEOUT_MS", "5000")
        .env("ATLAS_SQL_TIMEOUT_MS", "2500")
        .env("ATLAS_RESPONSE_MAX_BYTES", "1048576")
        .env("ATLAS_MAX_BODY_BYTES", "1048576")
        .env("ATLAS_SLOW_QUERY_THRESHOLD_MS", "500")
        .env("ATLAS_ENABLE_DEBUG_DATASETS", "false")
        .env("ATLAS_ENABLE_EXEMPLARS", "false")
        .env("ATLAS_CACHED_ONLY_MODE", "false")
        .env("ATLAS_READ_ONLY_FS_MODE", "false")
        .env("ATLAS_SHUTDOWN_DRAIN_MS", "10000")
        .env("ATLAS_READINESS_REQUIRES_CATALOG", "true")
        .env("ATLAS_MAX_SEQUENCE_BASES", "20000")
        .env("ATLAS_SEQUENCE_API_KEY_REQUIRED_BASES", "5000")
        .env("ATLAS_SEQUENCE_TTL_MS", "300000")
        .env("ATLAS_SEQUENCE_RATE_LIMIT_CAPACITY", "15")
        .env("ATLAS_SEQUENCE_RATE_LIMIT_REFILL_PER_SEC", "5")
        .env("ATLAS_CATALOG_BACKOFF_BASE_MS", "250")
        .env("ATLAS_CATALOG_BREAKER_FAILURE_THRESHOLD", "5")
        .env("ATLAS_CATALOG_BREAKER_OPEN_MS", "10000")
        .output()
        .expect("run atlas-server");

    assert!(
        output.status.success(),
        "the Helm-emitted runtime env surface must pass startup validation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn startup_only_allows_unknown_prefixed_env_when_the_dev_override_is_explicitly_enabled() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_DEV_ALLOW_UNKNOWN_ENV", "1")
        .env("ATLAS_UNKNOWN_CONTRACT_BREAKER", "1")
        .output()
        .expect("run atlas-server");

    assert!(
        output.status.success(),
        "explicit dev override must permit unknown env keys in local validation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn startup_rejects_invalid_config_values_instead_of_silently_using_defaults() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_ENABLE_AUDIT_LOG", "definitely")
        .output()
        .expect("run atlas-server");

    assert!(!output.status.success(), "invalid config must fail startup");
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        stderr.contains("invalid boolean value for ATLAS_ENABLE_AUDIT_LOG: definitely"),
        "invalid config value must be reported explicitly:\n{stderr}"
    );
}

#[test]
fn startup_rejects_missing_required_store_s3_configuration() {
    let output = sanitized_server_command()
        .env("ATLAS_STORE_S3_ENABLED", "1")
        .output()
        .expect("run atlas-server");

    assert!(
        !output.status.success(),
        "missing required S3 configuration must fail startup"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        stderr.contains("ATLAS_STORE_S3_BASE_URL is required when S3 enabled"),
        "missing required env must be reported explicitly:\n{stderr}"
    );
    assert!(
        !stderr.contains("atlas-server listening on"),
        "missing required env must fail before binding a port:\n{stderr}"
    );
}

#[test]
fn startup_rejects_loopback_bind_addresses_when_atlas_env_is_prod() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("ATLAS_ENV", "prod")
        .env("ATLAS_BIND", "127.0.0.1:8080")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .output()
        .expect("run atlas-server");

    assert!(
        !output.status.success(),
        "prod startup must reject loopback bind addresses"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        stderr.contains("ATLAS_ENV=prod forbids localhost/loopback bind addresses"),
        "prod bind guard must fail explicitly:\n{stderr}"
    );
    assert!(
        !stderr.contains("atlas-server listening on"),
        "prod bind guard must fail before binding a port:\n{stderr}"
    );
}

#[test]
fn startup_logs_the_redacted_effective_config_during_validation() {
    let store_root = tempdir().expect("store root tempdir");
    let cache_root = tempdir().expect("cache root tempdir");
    let output = sanitized_server_command()
        .arg("--validate-config")
        .env("RUST_LOG", "info")
        .env("ATLAS_LOG_JSON", "false")
        .env("ATLAS_STORE_ROOT", store_root.path())
        .env("ATLAS_CACHE_ROOT", cache_root.path())
        .env("ATLAS_HMAC_SECRET", "secret-material")
        .env("ATLAS_ALLOWED_API_KEYS", "alpha,beta")
        .output()
        .expect("run atlas-server");

    assert!(
        output.status.success(),
        "validation with explicit config must succeed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("effective runtime config"),
        "startup must log the effective config:\n{stdout}"
    );
    assert!(
        stdout.contains("<redacted>"),
        "startup logs must redact sensitive config values:\n{stdout}"
    );
}
