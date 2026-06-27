// SPDX-License-Identifier: Apache-2.0

mod atlas_client;
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

#[cfg(test)]
mod tests {
    use super::{AtlasClient, ClientConfig, ErrorClass};

    #[test]
    fn client_rejects_non_http_base_url() {
        let config = ClientConfig {
            base_url: "ftp://invalid".to_string(),
            ..ClientConfig::default()
        };
        let err = match AtlasClient::new(config) {
            Ok(_) => panic!("invalid config should fail"),
            Err(err) => err,
        };
        assert_eq!(err.class, ErrorClass::InvalidConfig);
    }
}
