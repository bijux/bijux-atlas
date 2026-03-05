// SPDX-License-Identifier: Apache-2.0

use crate::config::ClientConfig;
use crate::error::{ClientError, ErrorClass};
use crate::metrics::ClientMetrics;
use crate::pagination::{Page, PaginationCursor};
use crate::query::{DatasetQuery, QueryResult, StreamQuery};
use crate::request::RequestBuilder;
use crate::retry::run_with_retry;
use crate::tracing::TraceContext;
use reqwest::blocking::Client as HttpClient;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub trait ClientLogger: Send + Sync {
    fn log(&self, message: &str);
}

#[derive(Clone)]
pub struct AtlasClient {
    config: ClientConfig,
    http: HttpClient,
    metrics: Option<Arc<dyn ClientMetrics>>,
    logger: Option<Arc<dyn ClientLogger>>,
}

impl AtlasClient {
    pub fn new(config: ClientConfig) -> Result<Self, ClientError> {
        if !config.base_url.starts_with("http://") && !config.base_url.starts_with("https://") {
            return Err(ClientError::new(
                ErrorClass::InvalidConfig,
                "base_url must start with http:// or https://",
            ));
        }

        let mut headers = HeaderMap::new();
        for (k, v) in &config.default_headers {
            let name = HeaderName::from_bytes(k.as_bytes())
                .map_err(|_| ClientError::new(ErrorClass::InvalidConfig, "invalid header name"))?;
            let value = HeaderValue::from_str(v)
                .map_err(|_| ClientError::new(ErrorClass::InvalidConfig, "invalid header value"))?;
            headers.insert(name, value);
        }

        let http = HttpClient::builder()
            .timeout(Duration::from_millis(config.timeout_millis))
            .default_headers(headers)
            .build()
            .map_err(|err| ClientError::new(ErrorClass::Transport, err.to_string()))?;

        Ok(Self {
            config,
            http,
            metrics: None,
            logger: None,
        })
    }

    pub fn with_metrics(mut self, metrics: Arc<dyn ClientMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_logger(mut self, logger: Arc<dyn ClientLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn dataset_query(
        &self,
        query: &DatasetQuery,
        trace: Option<&TraceContext>,
    ) -> Result<Page<QueryResult>, ClientError> {
        let mut builder = RequestBuilder::new("/v1/genes")
            .with_param("release", &query.release)
            .with_param("species", &query.species)
            .with_param("assembly", &query.assembly)
            .with_param("limit", query.limit.to_string());
        if let Some(cursor) = &query.cursor {
            builder = builder.with_param("cursor", cursor);
        }
        if let Some(gene_id) = &query.filter.gene_id {
            builder = builder.with_param("gene_id", gene_id);
        }
        if let Some(biotype) = &query.filter.biotype {
            builder = builder.with_param("biotype", biotype);
        }
        if let Some(contig) = &query.filter.contig {
            builder = builder.with_param("contig", contig);
        }
        let mut include = Vec::new();
        if query.projection.include_coords {
            include.push("coords");
        }
        if query.projection.include_counts {
            include.push("counts");
        }
        if query.projection.include_biotype {
            include.push("biotype");
        }
        if !include.is_empty() {
            builder = builder.with_param("include", include.join(","));
        }
        let json = self.send(builder, trace)?;
        let rows = json
            .get("data")
            .and_then(|v| v.get("rows"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let items = rows
            .into_iter()
            .map(|row| QueryResult { raw: row })
            .collect::<Vec<_>>();
        let next = json
            .get("page")
            .and_then(|v| v.get("next_cursor"))
            .and_then(Value::as_str)
            .map(|value| PaginationCursor(value.to_string()));
        Ok(Page { items, next })
    }

    pub fn dataset_scan(&self, query: &DatasetQuery) -> Result<Vec<QueryResult>, ClientError> {
        let mut current = query.clone();
        let mut all = Vec::new();
        loop {
            let page = self.dataset_query(&current, None)?;
            all.extend(page.items);
            match page.next {
                Some(next) => current.cursor = Some(next.0),
                None => break,
            }
        }
        Ok(all)
    }

    pub fn filtered_query(&self, query: &DatasetQuery) -> Result<Page<QueryResult>, ClientError> {
        self.dataset_query(query, None)
    }

    pub fn stream_query(&self, query: &DatasetQuery) -> Result<StreamQuery, ClientError> {
        let mut current = query.clone();
        let mut pages = Vec::new();
        loop {
            let page = self.dataset_query(&current, None)?;
            let next = page.next.clone();
            pages.push(page);
            match next {
                Some(cursor) => current.cursor = Some(cursor.0),
                None => break,
            }
        }
        Ok(StreamQuery { pages })
    }

    pub fn paginate(&self, query: &DatasetQuery) -> Result<Page<QueryResult>, ClientError> {
        self.dataset_query(query, None)
    }

    fn send(
        &self,
        request: RequestBuilder,
        trace: Option<&TraceContext>,
    ) -> Result<Value, ClientError> {
        run_with_retry(
            self.config.retry_attempts,
            self.config.retry_backoff_millis,
            || {
                let started = Instant::now();
                let url = format!("{}{}", self.config.base_url, request.path());
                let mut call = self.http.get(url).query(request.query());
                if let Some(trace) = trace {
                    if let Some(request_id) = &trace.request_id {
                        call = call.header("x-request-id", request_id);
                    }
                    if let Some(trace_id) = &trace.trace_id {
                        call = call.header("x-trace-id", trace_id);
                    }
                }
                let response = call.send().map_err(|err| {
                    if err.is_timeout() {
                        ClientError::new(ErrorClass::Timeout, err.to_string())
                    } else {
                        ClientError::new(ErrorClass::Transport, err.to_string())
                    }
                })?;
                let status = response.status();
                let body = response
                    .text()
                    .map_err(|err| ClientError::new(ErrorClass::Transport, err.to_string()))?;
                let elapsed = started.elapsed().as_millis();

                if let Some(logger) = &self.logger {
                    logger.log(&format!(
                        "atlas-client request path={} status={}",
                        request.path(),
                        status
                    ));
                }
                if let Some(metrics) = &self.metrics {
                    metrics.observe_request(request.path(), elapsed, status.is_success());
                }

                if status.as_u16() == 429 {
                    return Err(ClientError::new(ErrorClass::RateLimited, "rate limited"));
                }
                if status.is_server_error() {
                    return Err(ClientError::new(
                        ErrorClass::Server,
                        format!("server error {}", status),
                    ));
                }
                if status.is_client_error() {
                    return Err(ClientError::new(
                        ErrorClass::Client,
                        format!("client error {}", status),
                    ));
                }
                serde_json::from_str(&body)
                    .map_err(|err| ClientError::new(ErrorClass::Decode, err.to_string()))
            },
        )
    }
}
