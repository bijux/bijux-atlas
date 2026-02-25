pub(crate) async fn genes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    super::genes::genes_handler(State(state), headers, axum::extract::Query(params)).await
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
    let (dataset, req) = match crate::http::genes_support::build_dataset_query(&params, 500) {
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
            let count = query_gene_count_with_filters(&c.conn, &req);
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

include!("../transcript_endpoints.rs");

fn query_gene_count_with_filters(
    conn: &rusqlite::Connection,
    req: &bijux_atlas_query::GeneQueryRequest,
) -> Result<i64, rusqlite::Error> {
    let mut sql = "SELECT COUNT(*) FROM gene_summary g".to_string();
    let mut where_parts: Vec<String> = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(region) = &req.filter.region {
        sql.push_str(" JOIN gene_summary_rtree r ON r.gene_rowid = g.id");
        where_parts.push("g.seqid = ?".to_string());
        params.push(rusqlite::types::Value::Text(region.seqid.clone()));
        where_parts.push("r.start <= ?".to_string());
        params.push(rusqlite::types::Value::Real(region.end as f64));
        where_parts.push("r.end >= ?".to_string());
        params.push(rusqlite::types::Value::Real(region.start as f64));
    }
    if let Some(gene_id) = &req.filter.gene_id {
        where_parts.push("g.gene_id = ?".to_string());
        params.push(rusqlite::types::Value::Text(gene_id.clone()));
    }
    if let Some(name) = &req.filter.name {
        where_parts.push("g.name_normalized = ?".to_string());
        params.push(rusqlite::types::Value::Text(
            bijux_atlas_query::normalize_name_lookup(name),
        ));
    }
    if let Some(prefix) = &req.filter.name_prefix {
        where_parts.push("g.name_normalized LIKE ? ESCAPE '!'".to_string());
        params.push(rusqlite::types::Value::Text(format!(
            "{}%",
            bijux_atlas_query::escape_like_prefix(&bijux_atlas_query::normalize_name_lookup(prefix))
        )));
    }
    if let Some(biotype) = &req.filter.biotype {
        where_parts.push("g.biotype = ?".to_string());
        params.push(rusqlite::types::Value::Text(biotype.clone()));
    }
    if !where_parts.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_parts.join(" AND "));
    }

    conn.query_row(&sql, rusqlite::params_from_iter(params.iter()), |row| {
        row.get::<_, i64>(0)
    })
}
