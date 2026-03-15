// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

extern crate self as bijux_atlas;

#[allow(unused_imports)]
use bijux_atlas::{core as bijux_atlas_core, model as bijux_atlas_model};

use crate::api::{ApiError, ApiErrorCode};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bijux_atlas::query::{
    classify_query, decode_cursor, encode_cursor, estimate_query_cost, query_genes, CursorPayload,
    GeneFields, GeneFilter, GeneQueryRequest, OrderMode, QueryClass, QueryLimits, RegionFilter,
    TranscriptFilter, TranscriptQueryRequest,
};
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
use hmac::{Hmac, Mac};
use rusqlite::Connection;
use sha2::Sha256;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn, Instrument};

pub mod adapters;
pub mod api;
pub mod app;
#[path = "app/cache/mod.rs"]
mod cache;
pub mod cli;
mod config;
pub mod contracts;
pub mod core;
pub mod domain;
#[path = "core/effect_adapters/mod.rs"]
mod effect_adapters;
pub mod effects;
pub mod errors;
#[path = "core/generated/mod.rs"]
mod generated;
pub mod ingest;
#[path = "adapters/http/middleware/mod.rs"]
mod middleware;
pub mod model;
#[path = "core/adapters/mod.rs"]
mod platform_adapters;
pub mod ports;
pub mod server;
#[path = "adapters/store/registry/mod.rs"]
mod server_store;
#[path = "core/services/mod.rs"]
mod services;
pub mod support;
pub mod types;

pub use crate::adapters::client;
pub use crate::adapters::http;
pub use crate::adapters::store;
pub use crate::adapters::telemetry;
pub use crate::app::{
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
pub use crate::cli::main_entry;
pub use crate::domain::policy as policies;
pub use crate::domain::query;
pub use crate::domain::routing::consistent_route_dataset;
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
