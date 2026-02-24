// SPDX-License-Identifier: Apache-2.0

use crate::{ApiError, ApiErrorCode};

pub const API_ERROR_SCHEMA_REF: &str = "#/components/schemas/ApiError";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiErrorMapping {
    pub status_code: u16,
    pub schema_ref: &'static str,
}

#[must_use]
pub fn map_error(error: &ApiError) -> ApiErrorMapping {
    let status_code = match error.code {
        ApiErrorCode::InvalidQueryParameter
        | ApiErrorCode::MissingDatasetDimension
        | ApiErrorCode::InvalidCursor
        | ApiErrorCode::RangeTooLarge
        | ApiErrorCode::ValidationFailed => 400,
        ApiErrorCode::PayloadTooLarge | ApiErrorCode::ResponseTooLarge => 413,
        ApiErrorCode::QueryRejectedByPolicy => 422,
        ApiErrorCode::RateLimited => 429,
        ApiErrorCode::NotReady | ApiErrorCode::UpstreamStoreUnavailable => 503,
        ApiErrorCode::DatasetNotFound | ApiErrorCode::GeneNotFound => 404,
        _ => 500,
    };

    ApiErrorMapping {
        status_code,
        schema_ref: API_ERROR_SCHEMA_REF,
    }
}
