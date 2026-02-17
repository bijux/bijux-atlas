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

include!("runtime/server_runtime_core.rs");
include!("runtime/dataset_cache_manager_impl.rs");
include!("runtime/server_runtime_app.rs");
