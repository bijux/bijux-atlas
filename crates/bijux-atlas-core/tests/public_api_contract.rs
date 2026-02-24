use std::fs;
use std::path::PathBuf;

#[test]
fn public_api_doc_lists_only_exported_symbols() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_doc = fs::read_to_string(manifest_dir.join("docs/public-api.md"))
        .expect("read docs/public-api.md");
    let lib_rs = fs::read_to_string(manifest_dir.join("src/lib.rs")).expect("read src/lib.rs");

    let expected_items = [
        "CRATE_NAME",
        "ENV_BIJUX_LOG_LEVEL",
        "ENV_BIJUX_CACHE_DIR",
        "NO_RANDOMNESS_POLICY",
        "ExitCode",
        "ConfigPathScope",
        "MachineError",
        "Error",
        "Result<T>",
        "ErrorCode",
        "ERROR_CODES",
        "Hash256",
        "DatasetId",
        "ShardId",
        "RunId",
        "FsPort",
        "ClockPort",
        "NetPort",
        "ProcessPort",
        "ProcessResult",
        "ErrorContext",
        "ResultExt",
        "canonical` module",
        "time` module",
        "sha256_hex",
        "sha256",
        "no_randomness_policy",
        "resolve_bijux_cache_dir",
        "resolve_bijux_config_path",
    ];

    for item in expected_items {
        assert!(api_doc.contains(item), "public api doc missing: {item}");
    }

    for token in [
        "pub mod domain;",
        "pub mod effects;",
        "pub mod errors;",
        "pub mod ports;",
        "pub mod types;",
        "pub const CRATE_NAME",
        "pub const ENV_BIJUX_LOG_LEVEL",
        "pub const ENV_BIJUX_CACHE_DIR",
        "pub const NO_RANDOMNESS_POLICY",
        "pub const fn no_randomness_policy",
        "pub use crate::domain::canonical",
        "pub use crate::domain::time",
        "pub use crate::domain::{resolve_bijux_cache_dir, resolve_bijux_config_path, sha256, sha256_hex, Hash256}",
        "pub use crate::errors::{",
    ] {
        assert!(
            lib_rs.contains(token),
            "lib.rs export contract missing: {token}"
        );
    }

    let exported_identifiers = [
        "CRATE_NAME",
        "ENV_BIJUX_LOG_LEVEL",
        "ENV_BIJUX_CACHE_DIR",
        "NO_RANDOMNESS_POLICY",
        "ConfigPathScope",
        "ErrorCode",
        "ERROR_CODES",
        "ExitCode",
        "MachineError",
        "Error",
        "Result",
        "Hash256",
        "DatasetId",
        "ShardId",
        "RunId",
        "FsPort",
        "ClockPort",
        "NetPort",
        "ProcessPort",
        "ProcessResult",
        "ErrorContext",
        "ResultExt",
        "sha256_hex",
        "sha256",
        "no_randomness_policy",
        "resolve_bijux_cache_dir",
        "resolve_bijux_config_path",
        "canonical",
        "time",
    ];

    for ident in exported_identifiers {
        assert!(
            api_doc.contains(ident),
            "docs/public-api.md missing exported identifier: {ident}"
        );
    }
}
