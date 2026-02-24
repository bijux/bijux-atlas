#![forbid(unsafe_code)]

mod adapters;
mod cache;
mod config;
mod dataset_shards;
mod effect_adapters;
mod http;
mod middleware;
mod routing_hash;
mod services;
mod store;
mod store_resilience;
mod telemetry;

include!("runtime/state/mod.rs");
include!("runtime/effects/mod.rs");
include!("runtime/orchestrator/mod.rs");

pub use crate::telemetry::generated::metrics_contract::CONTRACT_METRIC_NAMES;
pub use crate::telemetry::generated::trace_spans_contract::CONTRACT_TRACE_SPAN_NAMES;

#[cfg(test)]
mod cache_manager_tests;
