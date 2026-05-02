// SPDX-License-Identifier: Apache-2.0

use crate::adapters::inbound::http::handlers;
use crate::contracts::api::ApiErrorCode;
use crate::AppState;
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub(crate) async fn error_envelope_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    let request_id = handlers::propagated_request_id(request.headers(), &state);
    let response = next.run(request).await;
    normalize_error_response(
        response,
        &request_id,
        &method,
        &path,
        state.api.response_max_bytes,
    )
    .await
}

async fn normalize_error_response(
    response: Response,
    request_id: &str,
    method: &str,
    path: &str,
    max_response_bytes: usize,
) -> Response {
    let status = response.status();
    if status.is_success() || status.is_informational() || status.is_redirection() {
        return response;
    }

    if response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"))
    {
        return normalize_json_error_envelope(response, max_response_bytes).await;
    }

    if !matches!(
        status,
        StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED
    ) {
        return response;
    }

    let (code, message) = if status == StatusCode::NOT_FOUND {
        (ApiErrorCode::DatasetNotFound, "route not found")
    } else {
        (
            ApiErrorCode::InvalidQueryParameter,
            "method not allowed for route",
        )
    };
    let resp = handlers::api_error_response(
        status,
        handlers::error_json(
            code,
            message,
            json!({
                "method": method,
                "path": path
            }),
        ),
    );
    handlers::with_request_id(resp, request_id)
}

async fn normalize_json_error_envelope(response: Response, max_response_bytes: usize) -> Response {
    let status = response.status();
    let (parts, body) = response.into_parts();
    let Ok(bytes) = to_bytes(body, max_response_bytes.max(4096)).await else {
        return Response::from_parts(parts, Body::empty());
    };
    let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    if parsed.get("error").is_some() {
        return Response::from_parts(parts, Body::from(bytes));
    }
    let Some(map) = parsed.as_object() else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    if !(map.contains_key("code")
        && map.contains_key("message")
        && map.contains_key("details")
        && map.contains_key("request_id"))
    {
        return Response::from_parts(parts, Body::from(bytes));
    }
    let wrapped = json!({ "error": parsed });
    let mut normalized = (status, axum::Json(wrapped)).into_response();
    normalized.headers_mut().extend(parts.headers);
    normalized
}

#[cfg(test)]
mod tests {
    use super::{normalize_error_response, normalize_json_error_envelope};
    use crate::contracts::api::{ApiError, ApiErrorCode};
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::json;

    #[tokio::test]
    async fn keeps_existing_json_error_untouched() {
        let response = (
            StatusCode::NOT_FOUND,
            [("content-type", "application/json")],
            "{\"error\":{}}",
        )
            .into_response();
        let normalized =
            normalize_error_response(response, "req-1", "GET", "/missing/resource", 1024).await;
        assert_eq!(normalized.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            normalized
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.starts_with("application/json")),
            Some(true)
        );
    }

    #[tokio::test]
    async fn wraps_plain_transport_not_found_as_error_envelope() {
        let response = (StatusCode::NOT_FOUND, "not found").into_response();
        let normalized =
            normalize_error_response(response, "req-1", "GET", "/missing/resource", 1024).await;
        assert_eq!(normalized.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            normalized
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.starts_with("application/json")),
            Some(true)
        );
    }

    #[tokio::test]
    async fn wraps_top_level_api_error_shape() {
        let response = (
            StatusCode::FORBIDDEN,
            axum::Json(ApiError::new(
                ApiErrorCode::AccessForbidden,
                "forbidden",
                json!({}),
                "req-1".to_string(),
            )),
        )
            .into_response();
        let normalized = normalize_json_error_envelope(response, 2048).await;
        assert_eq!(normalized.status(), StatusCode::FORBIDDEN);
        let bytes = to_bytes(normalized.into_body(), 2048).await.expect("body");
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(parsed["error"]["code"].as_str(), Some("AccessForbidden"));
    }
}
