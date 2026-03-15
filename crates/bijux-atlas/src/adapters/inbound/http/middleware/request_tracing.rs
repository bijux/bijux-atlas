// SPDX-License-Identifier: Apache-2.0

use crate::http::request_tracing::extract_request_trace;
use crate::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::info;
use tracing::Instrument;

pub(crate) async fn request_tracing_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let route = request.uri().path().to_string();
    let trace = extract_request_trace(request.headers(), &state);

    let span = tracing::info_span!(
        "http.request",
        request_id = %trace.request_id,
        correlation_id = trace.correlation_id.as_deref().unwrap_or(""),
        run_id = trace.run_id.as_deref().unwrap_or(""),
        traceparent = trace.traceparent.as_deref().unwrap_or(""),
        request_origin = %crate::telemetry::logging::redact_if_needed(
            true,
            trace.request_origin.as_deref().unwrap_or("")
        ),
        error = tracing::field::Empty,
        method = %method,
        route = %route,
    );

    let mut response = next.run(request).instrument(span).await;
    let status_code = response.status().as_u16();
    if status_code >= 500 {
        tracing::Span::current().record("error", true);
    }
    info!(
        event_id = "request_handled",
        release_id = %crate::runtime::config::runtime_release_id(),
        governance_version = %crate::runtime::config::runtime_governance_version(),
        request_id = %trace.request_id,
        route = %route,
        status = status_code,
        "request handled"
    );
    if let Ok(value) = axum::http::HeaderValue::from_str(&trace.request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    if let Some(correlation_id) = &trace.correlation_id {
        if let Ok(value) = axum::http::HeaderValue::from_str(correlation_id) {
            response.headers_mut().insert("x-correlation-id", value);
        }
    }
    if let Some(traceparent) = &trace.traceparent {
        if let Ok(value) = axum::http::HeaderValue::from_str(traceparent) {
            response.headers_mut().insert("traceparent", value);
        }
    }
    if let Some(request_origin) = &trace.request_origin {
        if let Ok(value) = axum::http::HeaderValue::from_str(request_origin) {
            response.headers_mut().insert("x-request-origin", value);
        }
    }
    response
}
