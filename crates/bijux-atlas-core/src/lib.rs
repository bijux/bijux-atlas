#![forbid(unsafe_code)]

mod generated;

pub mod domain;
pub mod effects;
pub mod errors;
pub mod ports;
pub mod types;

pub use crate::domain::canonical;
pub use crate::domain::time;
pub use crate::domain::{resolve_bijux_cache_dir, resolve_bijux_config_path, sha256, sha256_hex, Hash256};
pub use crate::errors::{
    ConfigPathScope, Error, ErrorCode, ErrorContext, ExitCode, MachineError, Result, ResultExt,
    ERROR_CODES,
};

pub const CRATE_NAME: &str = "bijux-atlas-core";
pub const ENV_BIJUX_LOG_LEVEL: &str = "BIJUX_LOG_LEVEL";
pub const ENV_BIJUX_CACHE_DIR: &str = "BIJUX_CACHE_DIR";
pub const NO_RANDOMNESS_POLICY: &str = "Randomness is forbidden in bijux-atlas-core";

#[must_use]
pub const fn no_randomness_policy() -> &'static str {
    NO_RANDOMNESS_POLICY
}
