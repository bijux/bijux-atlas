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

#[cfg(test)]
mod tests {
    use super::api_error_status;
    use axum::http::StatusCode;
    use bijux_atlas_api::ApiErrorCode;

    #[test]
    fn error_registry_matches_openapi_and_status_mapping() {
        let registry: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../configs/contracts/observability/error-codes.json"
        ))
        .unwrap_or_else(|err| panic!("error registry: {err}"));
        let spec = bijux_atlas_api::openapi_v1_spec().to_string();
        let expected = [
            (
                "InvalidQueryParameter",
                ApiErrorCode::InvalidQueryParameter,
                StatusCode::BAD_REQUEST,
            ),
            (
                "InvalidCursor",
                ApiErrorCode::InvalidCursor,
                StatusCode::BAD_REQUEST,
            ),
            (
                "MissingDatasetDimension",
                ApiErrorCode::MissingDatasetDimension,
                StatusCode::BAD_REQUEST,
            ),
            (
                "ValidationFailed",
                ApiErrorCode::ValidationFailed,
                StatusCode::BAD_REQUEST,
            ),
            (
                "RangeTooLarge",
                ApiErrorCode::RangeTooLarge,
                StatusCode::BAD_REQUEST,
            ),
            (
                "PayloadTooLarge",
                ApiErrorCode::PayloadTooLarge,
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            (
                "ResponseTooLarge",
                ApiErrorCode::ResponseTooLarge,
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            (
                "QueryRejectedByPolicy",
                ApiErrorCode::QueryRejectedByPolicy,
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
            (
                "QueryTooExpensive",
                ApiErrorCode::QueryTooExpensive,
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
            (
                "RateLimited",
                ApiErrorCode::RateLimited,
                StatusCode::TOO_MANY_REQUESTS,
            ),
            (
                "DatasetNotFound",
                ApiErrorCode::DatasetNotFound,
                StatusCode::NOT_FOUND,
            ),
            (
                "GeneNotFound",
                ApiErrorCode::GeneNotFound,
                StatusCode::NOT_FOUND,
            ),
            (
                "NotReady",
                ApiErrorCode::NotReady,
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                "UpstreamStoreUnavailable",
                ApiErrorCode::UpstreamStoreUnavailable,
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                "Timeout",
                ApiErrorCode::Timeout,
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                "Internal",
                ApiErrorCode::Internal,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                "ArtifactCorrupted",
                ApiErrorCode::ArtifactCorrupted,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                "ArtifactQuarantined",
                ApiErrorCode::ArtifactQuarantined,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];
        let registry_codes = registry["codes"]
            .as_array()
            .unwrap_or_else(|| panic!("codes array"));
        assert_eq!(registry_codes.len(), expected.len());
        for (code, variant, status) in expected {
            assert!(
                registry_codes.iter().any(|row| {
                    row["code"].as_str() == Some(code)
                        && row["http_status"].as_u64() == Some(status.as_u16() as u64)
                }),
                "missing registry row for {code}"
            );
            assert_eq!(
                api_error_status(variant),
                status,
                "status mismatch for {code}"
            );
            assert!(
                spec.contains(&format!("\"{code}\"")),
                "openapi missing {code}"
            );
        }
    }
}
