#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const CRATE_NAME: &str = "bijux-atlas-core";

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    Usage = 2,
    Validation = 3,
    DependencyFailure = 4,
    Internal = 10,
}

pub const ENV_BIJUX_LOG_LEVEL: &str = "BIJUX_LOG_LEVEL";
pub const ENV_BIJUX_CACHE_DIR: &str = "BIJUX_CACHE_DIR";

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPathScope {
    User,
    Workspace,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachineError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: BTreeMap<String, String>,
}

impl MachineError {
    #[must_use]
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }
}

pub mod canonical {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use serde::Serialize;
    use serde_json::{Map, Value};
    use sha2::{Digest, Sha256};

    #[must_use]
    pub fn stable_sort_by_key<T, K: Ord, F: FnMut(&T) -> K>(
        mut values: Vec<T>,
        mut key: F,
    ) -> Vec<T> {
        values.sort_by_key(|v| key(v));
        values
    }

    pub fn stable_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
        let raw = serde_json::to_value(value)?;
        let normalized = normalize_json_value(raw);
        serde_json::to_vec(&normalized)
    }

    #[must_use]
    pub fn stable_hash_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    pub fn stable_json_hash_hex<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
        let bytes = stable_json_bytes(value)?;
        Ok(stable_hash_hex(&bytes))
    }

    pub fn encode_cursor_payload<T: Serialize>(payload: &T) -> Result<String, serde_json::Error> {
        let bytes = stable_json_bytes(payload)?;
        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn decode_cursor_payload(token: &str) -> Result<Value, String> {
        let bytes = URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|e| format!("cursor base64 decode failed: {e}"))?;
        serde_json::from_slice::<Value>(&bytes)
            .map_err(|e| format!("cursor JSON decode failed: {e}"))
    }

    fn normalize_json_value(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut sorted = Map::new();
                let mut entries: Vec<(String, Value)> = map
                    .into_iter()
                    .map(|(k, v)| (k, normalize_json_value(v)))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                for (k, v) in entries {
                    sorted.insert(k, v);
                }
                Value::Object(sorted)
            }
            Value::Array(items) => {
                Value::Array(items.into_iter().map(normalize_json_value).collect())
            }
            other => other,
        }
    }
}
