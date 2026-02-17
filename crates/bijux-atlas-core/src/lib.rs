#![forbid(unsafe_code)]

pub mod canonical;
mod error;
mod generated;
pub mod result_ext;
pub mod time;

use std::path::PathBuf;

pub const CRATE_NAME: &str = "bijux-atlas-core";

pub const ENV_BIJUX_LOG_LEVEL: &str = "BIJUX_LOG_LEVEL";
pub const ENV_BIJUX_CACHE_DIR: &str = "BIJUX_CACHE_DIR";
pub const NO_RANDOMNESS_POLICY: &str = "Randomness is forbidden in bijux-atlas-core";

pub use crate::canonical::Hash256;
pub use crate::error::{ConfigPathScope, ErrorCode, ExitCode, MachineError};
pub use crate::result_ext::{ErrorContext, ResultExt};

#[must_use]
pub const fn no_randomness_policy() -> &'static str {
    NO_RANDOMNESS_POLICY
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    canonical::stable_hash_hex(bytes)
}

#[must_use]
pub fn sha256(bytes: &[u8]) -> Hash256 {
    canonical::stable_hash_bytes(bytes)
}

#[must_use]
pub fn resolve_bijux_cache_dir() -> PathBuf {
    if let Ok(explicit) = std::env::var(ENV_BIJUX_CACHE_DIR) {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
        let trimmed = xdg_cache_home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("bijux");
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join(".cache").join("bijux");
        }
    }

    PathBuf::from(".bijux").join("cache")
}

#[must_use]
pub fn resolve_bijux_config_path(scope: ConfigPathScope) -> PathBuf {
    match scope {
        ConfigPathScope::User => {
            if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
                let trimmed = xdg_config_home.trim();
                if !trimmed.is_empty() {
                    return PathBuf::from(trimmed).join("bijux").join("config.toml");
                }
            }
            if let Ok(home) = std::env::var("HOME") {
                let trimmed = home.trim();
                if !trimmed.is_empty() {
                    return PathBuf::from(trimmed)
                        .join(".config")
                        .join("bijux")
                        .join("config.toml");
                }
            }
            PathBuf::from(".bijux").join("config.toml")
        }
        ConfigPathScope::Workspace => PathBuf::from(".bijux").join("config.toml"),
    }
}
