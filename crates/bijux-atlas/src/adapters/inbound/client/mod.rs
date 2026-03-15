// SPDX-License-Identifier: Apache-2.0

mod atlas_client;
#[cfg(test)]
mod client_tests;
mod config;
mod error;
mod metrics;
mod pagination;
mod query;
mod request;
pub mod retry;
mod tracing;

pub use atlas_client::{AtlasClient, ClientLogger};
pub use config::ClientConfig;
pub use error::{ClientError, ErrorClass};
pub use metrics::{ClientMetrics, InMemoryMetrics};
pub use pagination::{Page, PaginationCursor};
pub use query::{DatasetQuery, QueryFilter, QueryProjection, QueryResult, StreamQuery};
pub use request::RequestBuilder;
pub use retry::run_with_retry;
pub use tracing::TraceContext;
