use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ApiErrorCode {
    InvalidQueryParameter,
    MissingDatasetDimension,
    InvalidCursor,
    QueryRejectedByPolicy,
    RateLimited,
    Timeout,
    PayloadTooLarge,
    ResponseTooLarge,
    NotReady,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
    pub details: Value,
}

impl ApiError {
    #[must_use]
    pub fn new(code: ApiErrorCode, message: impl Into<String>, details: Value) -> Self {
        Self {
            code,
            message: message.into(),
            details,
        }
    }

    #[must_use]
    pub fn invalid_param(name: &str, value: &str) -> Self {
        Self::new(
            ApiErrorCode::InvalidQueryParameter,
            format!("invalid query parameter: {name}"),
            json!({"parameter": name, "value": value}),
        )
    }

    #[must_use]
    pub fn missing_dataset_dim(name: &str) -> Self {
        Self::new(
            ApiErrorCode::MissingDatasetDimension,
            format!("missing dataset dimension: {name}"),
            json!({"dimension": name}),
        )
    }

    #[must_use]
    pub fn invalid_cursor(value: &str) -> Self {
        Self::new(
            ApiErrorCode::InvalidCursor,
            "invalid cursor",
            json!({"cursor": value}),
        )
    }
}
