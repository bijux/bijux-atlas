// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

extern crate self as bijux_atlas;

#[allow(unused_imports)]
pub(crate) use bijux_atlas::{core as bijux_atlas_core, model as bijux_atlas_model};

pub(crate) use crate::api::{ApiError, ApiErrorCode};
pub(crate) use async_trait::async_trait;
pub(crate) use axum::body::Body;
pub(crate) use axum::extract::{DefaultBodyLimit, State};
pub(crate) use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, Uri};
pub(crate) use axum::middleware::{from_fn_with_state, Next};
pub(crate) use axum::response::{IntoResponse, Response};
pub(crate) use axum::routing::{get, post};
pub(crate) use axum::{Json, Router};
pub(crate) use bijux_atlas::query::{
    classify_query, decode_cursor, encode_cursor, estimate_query_cost, query_genes, CursorPayload,
    GeneFields, GeneFilter, GeneQueryRequest, OrderMode, QueryClass, QueryLimits, RegionFilter,
    TranscriptFilter, TranscriptQueryRequest,
};
pub(crate) use bijux_atlas_core::sha256_hex;
pub(crate) use bijux_atlas_model::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
pub(crate) use hmac::{Hmac, Mac};
pub(crate) use rusqlite::Connection;
pub(crate) use sha2::Sha256;
pub(crate) use std::collections::{HashMap, HashSet, VecDeque};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
pub(crate) use std::sync::Arc;
pub(crate) use std::time::{Duration, Instant};
pub(crate) use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};
pub(crate) use tokio::time::timeout;
pub(crate) use tracing::{error, info, warn, Instrument};

pub mod api;
pub mod application;
pub mod bootstrap;
mod config;
pub mod contracts;
pub mod core;
pub mod domain;
pub mod effects;
pub mod errors;
pub mod infrastructure;
pub mod interfaces;
pub mod model;
pub mod ports;
pub mod server;
pub mod support;
pub mod types;

pub use crate::application::{
    build_router, chrono_like_unix_millis, effective_config_payload,
    effective_runtime_config_payload, load_runtime_config, load_runtime_startup_config,
    record_shed_reason, route_sli_class, runtime_config_contract_snapshot,
    runtime_startup_config_docs_markdown, runtime_startup_config_schema_json,
    validate_runtime_env_contract, validate_startup_config_contract, ApiConfig, AppState,
    CacheError, CatalogFetch, CatalogMode, DatasetCacheConfig, DatasetCacheManager,
    DatasetStoreBackend, FederatedBackend, LocalFsBackend, RateLimitConfig, RegistrySource,
    RegistrySourceHealth, RetryPolicy, RuntimeConfig, RuntimeConfigError, RuntimeStartupConfig,
    S3LikeBackend, StoreConfig, StoreMode,
};
pub use crate::domain::ingest;
pub use crate::domain::policy as policies;
pub use crate::domain::query;
pub use crate::domain::routing::consistent_route_dataset;
pub use crate::infrastructure::redis;
pub use crate::infrastructure::store;
pub use crate::infrastructure::telemetry;
pub use crate::interfaces::cli;
pub use crate::interfaces::cli::main_entry;
pub use crate::interfaces::client;
pub use crate::interfaces::http;
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
