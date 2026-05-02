use super::*;
use crate::adapters::inbound::http::genes;
use crate::domain::query::query_gene_count;
use serde_json::json;

pub(crate) async fn genes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    genes::genes_handler(State(state), headers, axum::extract::Query(params)).await
}

pub(crate) async fn genes_count_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    if is_draining(&state) {
        let resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server draining; refusing new requests",
                json!({}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/genes/count",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let (dataset, req) =
        match crate::adapters::inbound::http::genes_support::build_dataset_query(&params, 500) {
            Ok(v) => v,
            Err(e) => {
                let resp = api_error_response(StatusCode::BAD_REQUEST, e);
                state
                    .metrics
                    .observe_request(
                        "/v1/genes/count",
                        StatusCode::BAD_REQUEST,
                        started.elapsed(),
                    )
                    .await;
                return with_request_id(resp, &request_id);
            }
        };

    match state.cache.open_dataset_connection(&dataset).await {
        Ok(c) => {
            let count = query_gene_count(&c.conn, &req);
            match count {
                Ok(v) => {
                    let epoch = state.cache.catalog_epoch().await;
                    let resp = Json(json!({
                        "dataset": dataset.canonical_string(),
                        "gene_count": v,
                        "catalog_epoch": epoch
                    }))
                    .into_response();
                    state
                        .metrics
                        .observe_request("/v1/genes/count", StatusCode::OK, started.elapsed())
                        .await;
                    with_request_id(resp, &request_id)
                }
                Err(e) => {
                    let resp = api_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_json(
                            ApiErrorCode::Internal,
                            "query failed",
                            json!({"message": e.to_string()}),
                        ),
                    );
                    state
                        .metrics
                        .observe_request(
                            "/v1/genes/count",
                            StatusCode::INTERNAL_SERVER_ERROR,
                            started.elapsed(),
                        )
                        .await;
                    with_request_id(resp, &request_id)
                }
            }
        }
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "dataset unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes/count",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            with_request_id(resp, &request_id)
        }
    }
}
