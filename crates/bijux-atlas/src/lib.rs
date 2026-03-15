// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

extern crate self as bijux_atlas;

pub(crate) use crate::contracts::api::{ApiError, ApiErrorCode};
pub(crate) use crate::domain::dataset::{ArtifactManifest, Catalog, DatasetId};
pub(crate) use crate::domain::sha256_hex;
pub(crate) use axum::body::Body;
pub(crate) use axum::extract::State;
pub(crate) use axum::http::{HeaderMap, HeaderValue, StatusCode};
pub(crate) use axum::response::{IntoResponse, Response};
pub(crate) use axum::Json;
pub(crate) use bijux_atlas::query::{
    classify_query, decode_cursor, encode_cursor, estimate_query_cost, query_genes, CursorPayload,
    GeneFields, GeneQueryRequest, OrderMode, QueryClass, RegionFilter, TranscriptFilter,
    TranscriptQueryRequest,
};
pub(crate) use rusqlite::Connection;
pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::atomic::{AtomicU64, Ordering};
pub(crate) use std::sync::Arc;
pub(crate) use std::time::{Duration, Instant};
pub(crate) use tokio::time::timeout;
pub(crate) use tracing::Instrument;

pub mod adapters;
pub mod app;
pub mod runtime;
pub mod bootstrap;
pub mod contracts;
pub mod core;
pub mod domain;
pub mod errors;
pub(crate) mod infrastructure;
pub(crate) mod interfaces;
pub mod model;
pub mod ports;
pub mod types;

pub use crate::contracts::api;
pub use crate::runtime::config::{
    effective_config_payload, effective_runtime_config_payload, load_runtime_config,
    load_runtime_startup_config, runtime_build_hash, runtime_config_contract_snapshot,
    runtime_governance_version, runtime_release_id,
    runtime_startup_config_docs_markdown, runtime_startup_config_schema_json,
    validate_runtime_env_contract, validate_startup_config_contract, ApiConfig, CatalogMode,
    RateLimitConfig, RuntimeConfig, RuntimeConfigError, RuntimeStartupConfig, StoreConfig,
    StoreMode,
};
pub use crate::app::server::{
    build_router, chrono_like_unix_millis, record_shed_reason, route_sli_class, AppState,
    DatasetCacheConfig, DatasetCacheManager, FederatedBackend, LocalFsBackend, RegistrySource,
    RetryPolicy, S3LikeBackend,
};
pub use crate::app::cache::{CacheError, RegistrySourceHealth};
pub use crate::app::ports::{CatalogFetch, DatasetStoreBackend};
pub use crate::domain::ingest;
pub use crate::domain::policy as policies;
pub use crate::domain::query;
pub use crate::domain::routing::consistent_route_dataset;
pub(crate) use crate::adapters::outbound::redis;
pub(crate) use crate::adapters::outbound::sqlite;
pub use crate::adapters::outbound::store;
pub(crate) use crate::adapters::outbound::telemetry;
pub use crate::adapters::inbound::cli;
pub use crate::adapters::inbound::cli::main_entry;
pub use crate::adapters::inbound::client;
pub(crate) use crate::adapters::inbound::http;
pub use crate::telemetry::generated::metrics_contract::CONTRACT_METRIC_NAMES;
pub use crate::telemetry::generated::trace_spans_contract::CONTRACT_TRACE_SPAN_NAMES;
pub use crate::telemetry::logging::{redact_if_needed, LoggingConfig};
pub use crate::telemetry::tracing::{init_tracing, TraceConfig, TraceExporterKind};

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
