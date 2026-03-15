// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use super::generated::error_codes::ApiErrorCode;

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
        let details =
            json!({"field_errors":[{"parameter": name, "reason": "invalid", "value": value}]});
        Self::new(
            ApiErrorCode::InvalidQueryParameter,
            format!("invalid query parameter: {name}"),
            details.clone(),
            fallback_request_id(ApiErrorCode::InvalidQueryParameter, &details),
        )
    }

    #[must_use]
    pub fn validation_failed(field_errors: Value) -> Self {
        let details = json!({"field_errors": field_errors});
        Self::new(
            ApiErrorCode::ValidationFailed,
            "validation failed",
            details.clone(),
            fallback_request_id(ApiErrorCode::ValidationFailed, &details),
        )
    }

    #[must_use]
    pub fn missing_dataset_dim(name: &str) -> Self {
        let details = json!({"dimension": name});
        Self::new(
            ApiErrorCode::MissingDatasetDimension,
            format!("missing dataset dimension: {name}"),
            details.clone(),
            fallback_request_id(ApiErrorCode::MissingDatasetDimension, &details),
        )
    }

    #[must_use]
    pub fn invalid_cursor(value: &str) -> Self {
        let details = json!({"cursor": value});
        Self::new(
            ApiErrorCode::InvalidCursor,
            "invalid cursor",
            details.clone(),
            fallback_request_id(ApiErrorCode::InvalidCursor, &details),
        )
    }
}

#[must_use]
pub(crate) fn fallback_request_id(code: ApiErrorCode, details: &Value) -> String {
    let canonical = crate::domain::canonical::stable_json_bytes(&(code.as_str(), details))
        .unwrap_or_else(|_| details.to_string().into_bytes());
    let digest = crate::domain::canonical::sha256_hex(&canonical);
    format!("req-{}", &digest[..16])
}

const _: fn() = || {
    fn assert_traits<T: Serialize + for<'de> Deserialize<'de>>() {}
    assert_traits::<ApiErrorCode>();
};
