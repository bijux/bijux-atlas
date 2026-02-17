#![forbid(unsafe_code)]

mod cache;
mod config;
mod dataset_shards;
mod effect_adapters;
mod http;
mod middleware;
mod routing_hash;
mod store;
mod store_resilience;
mod telemetry;

include!("runtime/state/mod.rs");
include!("runtime/effects/mod.rs");
include!("runtime/orchestrator/mod.rs");
