use std::path::PathBuf;

use crate::errors::ConfigPathScope;

#[must_use]
pub fn resolve_bijux_cache_dir() -> PathBuf {
    if let Ok(explicit) = std::env::var(crate::ENV_BIJUX_CACHE_DIR) {
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
