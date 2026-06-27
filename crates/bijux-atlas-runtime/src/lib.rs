// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

extern crate self as bijux_atlas_runtime;

pub mod adapters;
pub mod app;
pub mod contracts;
pub mod domain;
pub mod packaged;
pub mod runtime;
pub mod version;
#[allow(dead_code)]
pub(crate) mod version_support;

pub(crate) use crate::app::cache::{CacheError, RegistrySourceHealth};
pub(crate) use crate::app::ports::{CatalogFetch, DatasetStoreBackend};
pub(crate) use crate::runtime::config::DatasetCacheConfig;
#[allow(unused_imports)]
pub(crate) use crate::runtime::config::{runtime_build_hash, RateLimitConfig};

pub const CRATE_NAME: &str = "bijux-atlas-runtime";
pub const ENV_BIJUX_LOG_LEVEL: &str = "BIJUX_LOG_LEVEL";
pub const ENV_BIJUX_CACHE_DIR: &str = "BIJUX_CACHE_DIR";
pub const NO_RANDOMNESS_POLICY: &str = "Randomness is forbidden in bijux-atlas-runtime";

#[must_use]
pub const fn no_randomness_policy() -> &'static str {
    NO_RANDOMNESS_POLICY
}
