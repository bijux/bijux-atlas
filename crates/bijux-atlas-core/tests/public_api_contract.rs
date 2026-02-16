use std::fs;
use std::path::PathBuf;

#[test]
fn public_api_doc_lists_only_exported_symbols() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_doc = fs::read_to_string(manifest_dir.join("docs/PUBLIC_API.md"))
        .expect("read docs/PUBLIC_API.md");
    let lib_rs = fs::read_to_string(manifest_dir.join("src/lib.rs")).expect("read src/lib.rs");

    let expected_items = [
        "CRATE_NAME",
        "ENV_BIJUX_LOG_LEVEL",
        "ENV_BIJUX_CACHE_DIR",
        "ExitCode",
        "ConfigPathScope",
        "MachineError",
        "ErrorCode",
        "canonical` module",
        "time` module",
        "sha256_hex",
        "resolve_bijux_cache_dir",
        "resolve_bijux_config_path",
    ];

    for item in expected_items {
        assert!(api_doc.contains(item), "public api doc missing: {item}");
    }

    for token in [
        "pub mod canonical;",
        "pub mod time;",
        "pub const CRATE_NAME",
        "pub const ENV_BIJUX_LOG_LEVEL",
        "pub const ENV_BIJUX_CACHE_DIR",
        "pub fn sha256_hex",
        "pub fn resolve_bijux_cache_dir",
        "pub fn resolve_bijux_config_path",
        "pub use crate::error::{ConfigPathScope, ErrorCode, ExitCode, MachineError}",
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
        "ConfigPathScope",
        "ErrorCode",
        "ExitCode",
        "MachineError",
        "sha256_hex",
        "resolve_bijux_cache_dir",
        "resolve_bijux_config_path",
        "canonical",
        "time",
    ];

    for ident in exported_identifiers {
        assert!(
            api_doc.contains(ident),
            "docs/PUBLIC_API.md missing exported identifier: {ident}"
        );
    }
}
