// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

extern crate self as bijux_atlas;

pub mod adapters;
pub mod api;
pub mod app;
#[path = "server/cache/mod.rs"]
mod cache;
pub mod cli;
pub mod client;
mod config;
pub mod contracts;
pub mod core;
#[path = "server/cache/shards.rs"]
mod dataset_shards;
pub mod domain;
#[path = "core/effect_adapters/mod.rs"]
mod effect_adapters;
pub mod effects;
pub mod errors;
#[path = "core/generated/mod.rs"]
mod generated;
#[path = "server/http/mod.rs"]
mod http;
pub mod ingest;
#[path = "server/middleware/mod.rs"]
mod middleware;
pub mod model;
#[path = "core/adapters/mod.rs"]
mod platform_adapters;
pub mod policies;
pub mod ports;
pub mod query;
#[path = "server/routing.rs"]
mod routing_hash;
pub mod server;
#[path = "server/registry/mod.rs"]
mod server_store;
#[path = "core/services/mod.rs"]
mod services;
pub mod store;
#[path = "server/cache/resilience.rs"]
mod store_resilience;
pub mod support;
mod telemetry;
pub mod types;

include!("server/runtime/state/mod.rs");
include!("server/runtime/effects/mod.rs");
include!("server/runtime/orchestrator/mod.rs");

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
