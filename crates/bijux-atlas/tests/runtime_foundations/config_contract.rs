// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::contracts::errors::{ConfigPathScope, MachineError};
use bijux_atlas::runtime::config::{resolve_bijux_cache_dir, resolve_bijux_config_path};

#[test]
fn cache_dir_resolution_never_returns_empty_path() {
    let resolved = resolve_bijux_cache_dir();
    assert!(!resolved.as_os_str().is_empty());
}

#[test]
fn workspace_config_path_is_stable() {
    let path = resolve_bijux_config_path(ConfigPathScope::Workspace);
    assert_eq!(path.to_string_lossy(), ".bijux/config.toml");
}

#[test]
fn machine_error_rejects_unknown_fields() {
    let raw = r#"{"code":"x","message":"y","details":{},"extra":1}"#;
    let parsed = serde_json::from_str::<MachineError>(raw);
    assert!(parsed.is_err(), "unknown fields must be rejected");
}
