// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

extern crate self as bijux_atlas;

mod adapters;
mod artifact_validation;
pub mod api;
mod cache;
pub mod cli;
pub mod client;
mod config;
pub mod core;
mod dataset_shards;
pub mod domain;
mod effect_adapters;
pub mod effects;
pub mod errors;
mod generated;
mod http;
pub mod ingest;
mod middleware;
pub mod model;
pub mod policies;
pub mod ports;
pub mod query;
mod routing_hash;
mod services;
mod server_store;
pub mod store;
mod store_resilience;
mod telemetry;
pub mod types;

include!("runtime/state/mod.rs");
include!("runtime/effects/mod.rs");
include!("runtime/orchestrator/mod.rs");

pub use crate::cli::main_entry;
pub use crate::telemetry::generated::metrics_contract::CONTRACT_METRIC_NAMES;
pub use crate::telemetry::generated::trace_spans_contract::CONTRACT_TRACE_SPAN_NAMES;
pub use crate::telemetry::logging::{redact_if_needed, LoggingConfig};
pub use crate::telemetry::tracing::{init_tracing, TraceConfig, TraceExporterKind};

#[cfg(test)]
mod cache_manager_tests;
#[cfg(test)]
mod registry_tests;

pub const CRATE_NAME: &str = "bijux-atlas";
pub const ENV_BIJUX_LOG_LEVEL: &str = "BIJUX_LOG_LEVEL";
pub const ENV_BIJUX_CACHE_DIR: &str = "BIJUX_CACHE_DIR";
pub const NO_RANDOMNESS_POLICY: &str = "Randomness is forbidden in bijux-atlas";

#[must_use]
pub const fn no_randomness_policy() -> &'static str {
    NO_RANDOMNESS_POLICY
}
