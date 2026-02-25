// SPDX-License-Identifier: Apache-2.0

pub(crate) async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    crate::telemetry::metrics_endpoint::metrics_handler(State(state)).await
}

pub(crate) async fn datasets_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
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
                "/v1/datasets",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    info!(request_id = %request_id, route = "/v1/datasets", "request start");
    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| Catalog::new(vec![]));
    let include_bom = bool_query_flag(&params, "include_bom");
    let release_filter = params.get("release").map(std::string::String::as_str);
    let species_filter = params.get("species").map(std::string::String::as_str);
    let assembly_filter = params.get("assembly").map(std::string::String::as_str);
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .map_or(100, |v| v.clamp(1, 500));
    let cursor = params.get("cursor").cloned();

    let mut rows = catalog
        .datasets
        .iter()
        .filter(|entry| {
            release_filter.is_none_or(|v| entry.dataset.release.as_str() == v)
                && species_filter.is_none_or(|v| entry.dataset.species.as_str() == v)
                && assembly_filter.is_none_or(|v| entry.dataset.assembly.as_str() == v)
        })
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.dataset.canonical_string().cmp(&b.dataset.canonical_string()));

    let start_idx = cursor
        .as_deref()
        .and_then(|c| rows.iter().position(|r| r.dataset.canonical_string() == c))
        .map_or(0, |idx| idx.saturating_add(1));
    let page = rows.into_iter().skip(start_idx).take(limit + 1).collect::<Vec<_>>();
    let has_more = page.len() > limit;
    let page = if has_more {
        page[..limit].to_vec()
    } else {
        page
    };
    let next_cursor = if has_more {
        page.last().map(|entry| entry.dataset.canonical_string())
    } else {
        None
    };

    let datasets_payload = if include_bom {
        let mut with_bom = Vec::with_capacity(page.len());
        for entry in &page {
            let bom = match state.cache.fetch_manifest_summary(&entry.dataset).await {
                Ok(manifest) => json!({
                    "manifest_version": manifest.manifest_version,
                    "db_schema_version": manifest.db_schema_version,
                    "checksums": manifest.checksums,
                    "stats": manifest.stats
                }),
                Err(_) => Value::Null,
            };
            with_bom.push(json!({
                "dataset": entry.dataset,
                "manifest_path": entry.manifest_path,
                "sqlite_path": entry.sqlite_path,
                "bill_of_materials": bom
            }));
        }
        Value::Array(with_bom)
    } else {
        serde_json::to_value(&page).unwrap_or(Value::Array(Vec::new()))
    };
    let payload = json_envelope(
        None,
        Some(json!({"next_cursor": next_cursor})),
        json!({
            "items": datasets_payload,
            "stats": {
                "limit": limit,
                "returned": page.len()
            }
        }),
        next_cursor.map(|cursor| json!({"next_cursor": cursor})),
        None,
    );
    let etag = format!(
        "\"{}\"",
        sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(
            resp.headers_mut(),
            state.api.discovery_ttl,
            &etag,
            CachePolicy::CatalogDiscovery,
        );
        state
            .metrics
            .observe_request("/v1/datasets", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let mut response = Json(payload).into_response();
    put_cache_headers(
        response.headers_mut(),
        state.api.discovery_ttl,
        &etag,
        CachePolicy::CatalogDiscovery,
    );
    state
        .metrics
        .observe_request("/v1/datasets", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

pub(crate) async fn dataset_identity_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path((release, species, assembly)): axum::extract::Path<(
        String,
        String,
        String,
    )>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = propagated_request_id(&headers, &state);
    let dataset = match DatasetId::new(&release, &species, &assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                error_json(
                    ApiErrorCode::InvalidQueryParameter,
                    "invalid dataset dimensions",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/datasets/{release}/{species}/{assembly}",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or_else(|| Catalog::new(vec![]));
    let entry = catalog
        .datasets
        .iter()
        .find(|e| e.dataset == dataset)
        .cloned();
    if entry.is_none() {
        let resp = api_error_response(
            StatusCode::NOT_FOUND,
            error_json(
                ApiErrorCode::DatasetNotFound,
                "dataset not found in catalog",
                json!({
                    "release": release,
                    "species": species,
                    "assembly": assembly
                }),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/datasets/{release}/{species}/{assembly}",
                StatusCode::NOT_FOUND,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let include_bom = bool_query_flag(&params, "include_bom");
    let manifest = match state.cache.fetch_manifest_summary(&dataset).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::UpstreamStoreUnavailable,
                    "dataset manifest unavailable",
                    json!({"message": e.to_string(), "retryable": true}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/datasets/{release}/{species}/{assembly}",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let endpoints = json!([
        "/v1/genes",
        "/v1/genes/count",
        "/v1/genes/{gene_id}/transcripts",
        "/v1/genes/{gene_id}/sequence",
        "/v1/transcripts/{tx_id}",
        "/v1/sequence/region"
    ]);
    let query = format!(
        "release={}&species={}&assembly={}",
        dataset.release.as_str(),
        dataset.species.as_str(),
        dataset.assembly.as_str()
    );
    let item = json!({
        "dataset": dataset,
        "artifact_hash": manifest.artifact_hash,
        "artifact_db_hash": manifest.db_hash,
        "checksums": {
            "sqlite_sha256": manifest.checksums.sqlite_sha256
        },
        "shard_info": {
            "plan": manifest.sharding_plan,
            "router": manifest.sharding_plan != bijux_atlas_model::ShardingPlan::None
        },
        "metadata": {
            "provenance": dataset_provenance(&state, &dataset).await,
            "catalog_entry": entry,
            "manifest_summary": {
                "manifest_version": manifest.manifest_version,
                "db_schema_version": manifest.db_schema_version,
                "stats": manifest.stats
            },
            "qc_summary": {
                "gene_count": manifest.stats.gene_count,
                "transcript_count": manifest.stats.transcript_count,
                "contig_count": manifest.stats.contig_count
            }
        },
        "available_endpoints": endpoints,
        "links": {
            "genes": format!("/v1/genes?{query}"),
            "genes_count": format!("/v1/genes/count?{query}"),
            "sequence_region": format!("/v1/sequence/region?{query}&region=chr1:1-10")
        }
    });
    let mut data = json!({ "item": item });
    if include_bom {
        data["item"]["bill_of_materials"] = json!({
            "checksums": manifest.checksums,
            "manifest_version": manifest.manifest_version,
            "db_schema_version": manifest.db_schema_version
        });
    }
    let payload = json_envelope(Some(json!(dataset)), None, data, None, None);
    let etag = format!(
        "\"{}\"",
        sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(
            resp.headers_mut(),
            state.api.immutable_gene_ttl,
            &etag,
            CachePolicy::ImmutableDataset,
        );
        state
            .metrics
            .observe_request(
                "/v1/datasets/{release}/{species}/{assembly}",
                StatusCode::NOT_MODIFIED,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }
    let mut resp = Json(payload).into_response();
    put_cache_headers(
        resp.headers_mut(),
        state.api.immutable_gene_ttl,
        &etag,
        CachePolicy::ImmutableDataset,
    );
    state
        .metrics
        .observe_request(
            "/v1/datasets/{release}/{species}/{assembly}",
            StatusCode::OK,
            started.elapsed(),
        )
        .await;
    with_request_id(resp, &request_id)
}

