// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub(crate) use axum::body::Body;
pub(crate) use axum::extract::State;
pub(crate) use axum::http::{HeaderMap, HeaderValue, StatusCode};
pub(crate) use axum::response::{IntoResponse, Response};
pub(crate) use axum::Json;
pub(crate) use bijux_atlas_api::{ApiError, ApiErrorCode};
pub(crate) use bijux_atlas_core::sha256_hex;
pub(crate) use bijux_atlas_model::dataset::{ArtifactManifest, Catalog, DatasetId};
pub(crate) use bijux_atlas_query::{
    classify_query, decode_cursor, encode_cursor, estimate_query_cost, query_genes, CursorPayload,
    GeneFields, GeneQueryRequest, OrderMode, QueryClass, RegionFilter, TranscriptFilter,
    TranscriptQueryRequest,
};
pub(crate) use rusqlite::Connection;
pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::atomic::Ordering;
pub(crate) use std::time::{Duration, Instant};
pub(crate) use tokio::time::timeout;
pub(crate) use tracing::Instrument;

pub mod adapters;
pub mod app;
pub mod packaged;
pub mod query;
pub mod runtime;
pub mod version;

pub(crate) use crate::app::server::observability::{chrono_like_unix_millis, record_shed_reason};
pub(crate) use crate::app::server::AppState;
pub(crate) use bijux_atlas_runtime::app::cache::CacheError;
#[rustfmt::skip]
pub(crate) use crate::runtime::config::runtime_build_hash;

pub const CRATE_NAME: &str = "bijux-atlas-server";
