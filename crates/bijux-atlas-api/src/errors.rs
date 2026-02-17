use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub type ApiErrorCode = bijux_atlas_core::ErrorCode;

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

const _: fn() = || {
    fn assert_traits<T: Serialize + for<'de> Deserialize<'de>>() {}
    assert_traits::<ApiErrorCode>();
};
