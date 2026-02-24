// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use crate::generated::error_codes::ApiErrorCode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
    pub details: Value,
    pub request_id: String,
}

impl ApiError {
    #[must_use]
    pub fn new(
        code: ApiErrorCode,
        message: impl Into<String>,
        details: Value,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details,
            request_id: request_id.into(),
        }
    }

    #[must_use]
    pub fn invalid_param(name: &str, value: &str) -> Self {
        Self::new(
            ApiErrorCode::InvalidQueryParameter,
            format!("invalid query parameter: {name}"),
            json!({"field_errors":[{"parameter": name, "reason": "invalid", "value": value}]}),
            "req-unknown",
        )
    }

    #[must_use]
    pub fn validation_failed(field_errors: Value) -> Self {
        Self::new(
            ApiErrorCode::ValidationFailed,
            "validation failed",
            json!({"field_errors": field_errors}),
            "req-unknown",
        )
    }

    #[must_use]
    pub fn missing_dataset_dim(name: &str) -> Self {
        Self::new(
            ApiErrorCode::MissingDatasetDimension,
            format!("missing dataset dimension: {name}"),
            json!({"dimension": name}),
            "req-unknown",
        )
    }

    #[must_use]
    pub fn invalid_cursor(value: &str) -> Self {
        Self::new(
            ApiErrorCode::InvalidCursor,
            "invalid cursor",
            json!({"cursor": value}),
            "req-unknown",
        )
    }
}

const _: fn() = || {
    fn assert_traits<T: Serialize + for<'de> Deserialize<'de>>() {}
    assert_traits::<ApiErrorCode>();
};
