use crate::*;
use crate::http::{genes_support, handlers};
use serde_json::json;

pub(super) async fn enforce_ip_rate_limit(
    state: &AppState,
    headers: &HeaderMap,
    adaptive_rl: f64,
    started: Instant,
    request_id: &str,
) -> Option<Response> {
    if let Some(ip) = handlers::normalized_forwarded_for(headers) {
        if !state
            .ip_limiter
            .allow_with_factor(&ip, &state.api.rate_limit_per_ip, adaptive_rl)
            .await
        {
            let resp = handlers::api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                handlers::error_json(
                    ApiErrorCode::RateLimited,
                    "rate limit exceeded",
                    json!({"scope":"ip"}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return Some(handlers::with_request_id(resp, request_id));
        }
    }
    None
}

pub(super) async fn enforce_api_key_rate_limit(
    state: &AppState,
    headers: &HeaderMap,
    adaptive_rl: f64,
    started: Instant,
    request_id: &str,
) -> Option<Response> {
    if state.api.enable_api_key_rate_limit {
        if let Some(key) = handlers::normalized_api_key(headers) {
            if !state
                .api_key_limiter
                .allow_with_factor(&key, &state.api.rate_limit_per_api_key, adaptive_rl)
                .await
            {
                let resp = handlers::api_error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    handlers::error_json(
                        ApiErrorCode::RateLimited,
                        "rate limit exceeded",
                        json!({"scope":"api_key"}),
                    ),
                );
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::TOO_MANY_REQUESTS,
                        started.elapsed(),
                    )
                    .await;
                return Some(handlers::with_request_id(resp, request_id));
            }
        }
    }
    None
}

pub(super) async fn acquire_heavy_worker_permit(
    state: &AppState,
    class: QueryClass,
    started: Instant,
    request_id: &str,
) -> Result<Option<tokio::sync::OwnedSemaphorePermit>, Response> {
    if class == QueryClass::Heavy {
        match state.heavy_workers.clone().try_acquire_owned() {
            Ok(permit) => Ok(Some(permit)),
            Err(_) => {
                let resp = handlers::api_error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    handlers::error_json(
                        ApiErrorCode::QueryRejectedByPolicy,
                        "heavy worker pool is saturated",
                        json!({"class":"heavy"}),
                    ),
                );
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::TOO_MANY_REQUESTS,
                        started.elapsed(),
                    )
                    .await;
                Err(handlers::with_request_id(resp, request_id))
            }
        }
    } else {
        Ok(None)
    }
}

pub(super) fn try_enter_request_queue(
    state: &AppState,
) -> Result<genes_support::QueueGuard, ApiError> {
    genes_support::try_enter_queue(state)
}
