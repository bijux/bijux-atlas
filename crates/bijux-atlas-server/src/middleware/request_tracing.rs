// SPDX-License-Identifier: Apache-2.0

use crate::http::request_tracing::extract_request_trace;
use crate::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
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
        method = %method,
        route = %route,
    );

    let mut response = next.run(request).instrument(span).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&trace.request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}
