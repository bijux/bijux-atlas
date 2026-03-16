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
pub(crate) use bijux_atlas::domain::query::{
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
pub mod contracts;
pub mod domain;
pub mod runtime;
pub mod version;
#[allow(dead_code)]
pub(crate) mod version_support;

pub(crate) use crate::adapters::inbound::http::request_policies::{
    chrono_like_unix_millis, record_shed_reason, route_sli_class,
};
pub(crate) use crate::app::cache::{CacheError, RegistrySourceHealth};
pub(crate) use crate::app::ports::{CatalogFetch, DatasetStoreBackend};
pub(crate) use crate::app::server::{AppState, DatasetCacheConfig, DatasetCacheManager};
pub(crate) use crate::runtime::config::{RateLimitConfig, runtime_build_hash};

pub const CRATE_NAME: &str = "bijux-atlas";
pub const ENV_BIJUX_LOG_LEVEL: &str = "BIJUX_LOG_LEVEL";
pub const ENV_BIJUX_CACHE_DIR: &str = "BIJUX_CACHE_DIR";
pub const NO_RANDOMNESS_POLICY: &str = "Randomness is forbidden in bijux-atlas";

#[must_use]
pub const fn no_randomness_policy() -> &'static str {
    NO_RANDOMNESS_POLICY
}
