// SPDX-License-Identifier: Apache-2.0

use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use bijux_atlas_api::{ApiError, ApiErrorCode};
use serde_json::{json, Value};

#[must_use]
#[allow(dead_code)]
pub(crate) fn api_error_status(code: ApiErrorCode) -> StatusCode {
    match code {
        ApiErrorCode::InvalidQueryParameter
        | ApiErrorCode::InvalidCursor
        | ApiErrorCode::MissingDatasetDimension
        | ApiErrorCode::ValidationFailed
        | ApiErrorCode::RangeTooLarge => StatusCode::BAD_REQUEST,
        ApiErrorCode::PayloadTooLarge | ApiErrorCode::ResponseTooLarge => {
            StatusCode::PAYLOAD_TOO_LARGE
        }
        ApiErrorCode::QueryRejectedByPolicy | ApiErrorCode::QueryTooExpensive => {
            StatusCode::UNPROCESSABLE_ENTITY
        }
        ApiErrorCode::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ApiErrorCode::DatasetNotFound | ApiErrorCode::GeneNotFound => StatusCode::NOT_FOUND,
        ApiErrorCode::NotReady | ApiErrorCode::UpstreamStoreUnavailable | ApiErrorCode::Timeout => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[must_use]
pub(crate) fn api_error_response(status: StatusCode, err: ApiError) -> Response {
    let body = Json(json!({"error": err}));
    let mut resp = (status, body).into_response();
    if matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE
    ) {
        resp.headers_mut()
            .insert("retry-after", HeaderValue::from_static("3"));
    }
    resp
}

#[must_use]
pub(crate) fn api_error(code: ApiErrorCode, message: &str, details: Value) -> ApiError {
    ApiError {
        code,
        message: message.to_string(),
        details,
        request_id: "req-unknown".to_string(),
    }
}
