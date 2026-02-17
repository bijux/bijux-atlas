#![deny(clippy::redundant_clone)]

use crate::http::handlers::{
    api_error_response, bool_query_flag, error_json, if_none_match, maybe_compress_response,
    normalize_query, put_cache_headers, serialize_payload_with_capacity, wants_text,
    with_request_id,
};
use crate::*;
use axum::extract::Path as AxumPath;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use tracing::info;

#[derive(Debug, Clone)]
struct FaiRecord {
    len: u64,
    offset: u64,
    line_bases: u64,
    line_bytes: u64,
}

fn parse_dataset(params: &HashMap<String, String>) -> Result<DatasetId, ApiError> {
    let release = params
        .get("release")
        .ok_or_else(|| ApiError::missing_dataset_dim("release"))?;
    let species = params
        .get("species")
        .ok_or_else(|| ApiError::missing_dataset_dim("species"))?;
    let assembly = params
        .get("assembly")
        .ok_or_else(|| ApiError::missing_dataset_dim("assembly"))?;
    DatasetId::new(release, species, assembly)
        .map_err(|e| ApiError::invalid_param("dataset", &e.to_string()))
}

fn parse_region(raw: &str) -> Result<(String, u64, u64), ApiError> {
    let (seqid, span) = raw.split_once(':').ok_or_else(|| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"region": raw}),
        )
    })?;
    let (start, end) = span.split_once('-').ok_or_else(|| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"region": raw}),
        )
    })?;
    let start = start.parse::<u64>().map_err(|_| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"region": raw}),
        )
    })?;
    let end = end.parse::<u64>().map_err(|_| {
        error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"region": raw}),
        )
    })?;
    if start == 0 || end < start {
        return Err(error_json(
            ApiErrorCode::InvalidQueryParameter,
            "invalid region",
            json!({"region": raw}),
        ));
    }
    Ok((seqid.to_string(), start, end))
}

fn parse_fai(path: &std::path::Path) -> Result<HashMap<String, FaiRecord>, ApiError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        error_json(
            ApiErrorCode::Internal,
            "fai read failed",
            json!({"message": e.to_string()}),
        )
    })?;
    let mut out = HashMap::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            continue;
        }
        let rec = FaiRecord {
            len: parts[1].parse::<u64>().unwrap_or(0),
            offset: parts[2].parse::<u64>().unwrap_or(0),
            line_bases: parts[3].parse::<u64>().unwrap_or(0),
            line_bytes: parts[4].parse::<u64>().unwrap_or(0),
        };
        out.insert(parts[0].to_string(), rec);
    }
    Ok(out)
}

fn extract_sequence(
    fasta_path: &std::path::Path,
    rec: &FaiRecord,
    start: u64,
    end: u64,
) -> Result<String, ApiError> {
    let mut file = File::open(fasta_path).map_err(|e| {
        error_json(
            ApiErrorCode::Internal,
            "fasta open failed",
            json!({"message": e.to_string()}),
        )
    })?;
    let mut out = String::with_capacity((end - start + 1) as usize);
    let mut pos = start;
    while pos <= end {
        let zero = pos - 1;
        let line = zero / rec.line_bases;
        let col = zero % rec.line_bases;
        let line_remaining = rec.line_bases - col;
        let want = (end - pos + 1).min(line_remaining);
        let byte_offset = rec.offset + line * rec.line_bytes + col;
        file.seek(SeekFrom::Start(byte_offset)).map_err(|e| {
            error_json(
                ApiErrorCode::Internal,
                "fasta seek failed",
                json!({"message": e.to_string()}),
            )
        })?;
        let mut buf = vec![0_u8; want as usize];
        file.read_exact(&mut buf).map_err(|e| {
            error_json(
                ApiErrorCode::Internal,
                "fasta read failed",
                json!({"message": e.to_string()}),
            )
        })?;
        out.push_str(&String::from_utf8_lossy(&buf));
        pos += want;
    }
    Ok(out)
}

fn sequence_meta(sequence: &str) -> serde_json::Value {
    if sequence.is_empty() {
        return json!({"gc_fraction": 0.0, "masked_fraction": 0.0});
    }
    let mut gc = 0_u64;
    let mut masked = 0_u64;
    for ch in sequence.chars() {
        match ch {
            'G' | 'g' | 'C' | 'c' => gc += 1,
            _ => {}
        }
        if ch.is_ascii_lowercase() || ch == 'N' || ch == 'n' {
            masked += 1;
        }
    }
    let len = sequence.len() as f64;
    json!({
        "gc_fraction": (gc as f64) / len,
        "masked_fraction": (masked as f64) / len
    })
}

async fn acquire_class_permit_for_sequence(
    state: &AppState,
    class: QueryClass,
) -> Result<tokio::sync::OwnedSemaphorePermit, ApiError> {
    let sem = match class {
        QueryClass::Cheap => state.class_cheap.clone(),
        QueryClass::Medium => state.class_medium.clone(),
        QueryClass::Heavy => state.class_heavy.clone(),
    };
    sem.try_acquire_owned().map_err(|_| {
        error_json(
            ApiErrorCode::QueryRejectedByPolicy,
            "query class concurrency limit exceeded",
            json!({"class": format!("{class:?}")}),
        )
    })
}

async fn sequence_common(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
    route: &str,
    region_raw: String,
) -> Response {
    let started = Instant::now();
    let request_id = crate::http::handlers::propagated_request_id(&headers, &state);
    info!(request_id = %request_id, route = route, "request start");

    if let Some(ip) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if !state
            .sequence_ip_limiter
            .allow(ip, &state.api.sequence_rate_limit_per_ip)
            .await
        {
            let resp = api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                error_json(
                    ApiErrorCode::RateLimited,
                    "sequence rate limit exceeded",
                    json!({"scope":"sequence_ip"}),
                ),
            );
            state
                .metrics
                .observe_request(route, StatusCode::TOO_MANY_REQUESTS, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    }

    let dataset = match parse_dataset(&params) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request(route, StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let (seqid, start, end) = match parse_region(&region_raw) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request(route, StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let requested_bases = end - start + 1;
    let class = if requested_bases as usize >= state.api.sequence_api_key_required_bases {
        QueryClass::Heavy
    } else {
        QueryClass::Medium
    };
    let overloaded = crate::middleware::shedding::overloaded(&state).await;
    if crate::middleware::shedding::should_shed_noncheap(&state, class).await
        || (state.api.shed_load_enabled && class == QueryClass::Heavy && overloaded)
    {
        let resp = api_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "server is shedding non-cheap query load",
                json!({"class": format!("{class:?}")}),
            ),
        );
        state
            .metrics
            .observe_request(route, StatusCode::SERVICE_UNAVAILABLE, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let _class_permit = match acquire_class_permit_for_sequence(&state, class).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::TOO_MANY_REQUESTS, e);
            state
                .metrics
                .observe_request(route, StatusCode::TOO_MANY_REQUESTS, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    if requested_bases as usize > state.api.max_sequence_bases {
        let resp = api_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "requested region exceeds max bases",
                json!({"requested_bases": requested_bases, "max_sequence_bases": state.api.max_sequence_bases}),
            ),
        );
        state
            .metrics
            .observe_request(route, StatusCode::UNPROCESSABLE_ENTITY, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    if requested_bases as usize >= state.api.sequence_api_key_required_bases
        && headers.get("x-api-key").is_none()
    {
        let resp = api_error_response(
            StatusCode::UNAUTHORIZED,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "api key required for large sequence request",
                json!({"threshold_bases": state.api.sequence_api_key_required_bases}),
            ),
        );
        state
            .metrics
            .observe_request(route, StatusCode::UNAUTHORIZED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }

    let coalesce_key = format!(
        "sequence:{}:{}:{}",
        dataset.canonical_string(),
        region_raw,
        normalize_query(&params)
    );
    let _coalesce_guard = state.coalescer.acquire(&coalesce_key).await;

    let io_stage = Instant::now();
    let (fasta_path, fai_path) = match state.cache.ensure_sequence_inputs_cached(&dataset).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "sequence inputs unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(route, StatusCode::SERVICE_UNAVAILABLE, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let fai = match parse_fai(&fai_path) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
            state
                .metrics
                .observe_request(route, StatusCode::INTERNAL_SERVER_ERROR, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let Some(rec) = fai.get(&seqid) else {
        let resp = api_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            error_json(
                ApiErrorCode::InvalidQueryParameter,
                "contig not found",
                json!({"seqid": seqid}),
            ),
        );
        state
            .metrics
            .observe_request(route, StatusCode::UNPROCESSABLE_ENTITY, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    };
    if end > rec.len {
        let resp = api_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            error_json(
                ApiErrorCode::InvalidQueryParameter,
                "region outside contig bounds",
                json!({"seqid": seqid, "contig_length": rec.len, "end": end}),
            ),
        );
        state
            .metrics
            .observe_request(route, StatusCode::UNPROCESSABLE_ENTITY, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let sequence = match extract_sequence(&fasta_path, rec, start, end) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
            state
                .metrics
                .observe_request(route, StatusCode::INTERNAL_SERVER_ERROR, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    state
        .metrics
        .observe_stage("fasta_io", io_stage.elapsed())
        .await;

    let etag = format!(
        "\"{}\"",
        sha256_hex(
            format!(
                "{}|{}|{}",
                dataset.canonical_string(),
                region_raw,
                normalize_query(&params)
            )
            .as_bytes()
        )
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::NOT_MODIFIED;
        let mut h = HeaderMap::new();
        put_cache_headers(&mut h, state.api.sequence_ttl, &etag);
        *resp.headers_mut() = h;
        state
            .metrics
            .observe_request(route, StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }

    let include_stats = bool_query_flag(&params, "include_stats");
    let provenance = crate::http::handlers::dataset_provenance(&state, &dataset).await;
    let serialize_stage = Instant::now();
    if wants_text(&headers) {
        if sequence.len() > state.api.response_max_bytes {
            let resp = api_error_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                error_json(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "response size exceeds configured limit",
                    json!({"size_bytes": sequence.len(), "max": state.api.response_max_bytes}),
                ),
            );
            state
                .metrics
                .observe_request(route, StatusCode::PAYLOAD_TOO_LARGE, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
        let mut h = HeaderMap::new();
        put_cache_headers(&mut h, state.api.sequence_ttl, &etag);
        h.insert(
            "content-type",
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        let mut resp = (StatusCode::OK, h, sequence).into_response();
        state
            .metrics
            .observe_stage("serialization", serialize_stage.elapsed())
            .await;
        state
            .metrics
            .observe_request(route, StatusCode::OK, started.elapsed())
            .await;
        resp = with_request_id(resp, &request_id);
        return resp;
    }

    let payload = if include_stats {
        json!({
            "dataset": dataset,
            "provenance": provenance,
            "region": {"seqid": seqid, "start": start, "end": end},
            "length": sequence.len(),
            "sequence": sequence,
            "meta": sequence_meta(&sequence)
        })
    } else {
        json!({
            "dataset": dataset,
            "provenance": provenance,
            "region": {"seqid": seqid, "start": start, "end": end},
            "length": sequence.len(),
            "sequence": sequence
        })
    };
    let body =
        match serialize_payload_with_capacity(&payload, false, payload.to_string().len() + 64) {
            Ok(v) => v,
            Err(e) => {
                let resp = api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
                state
                    .metrics
                    .observe_request(route, StatusCode::INTERNAL_SERVER_ERROR, started.elapsed())
                    .await;
                return with_request_id(resp, &request_id);
            }
        };
    let (encoded, encoding) = match maybe_compress_response(&headers, &state, body) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
            state
                .metrics
                .observe_request(route, StatusCode::INTERNAL_SERVER_ERROR, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    if encoded.len() > state.api.response_max_bytes {
        let resp = api_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            error_json(
                ApiErrorCode::QueryRejectedByPolicy,
                "response size exceeds configured limit",
                json!({"size_bytes": encoded.len(), "max": state.api.response_max_bytes}),
            ),
        );
        state
            .metrics
            .observe_request(route, StatusCode::PAYLOAD_TOO_LARGE, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    state
        .metrics
        .observe_stage("serialization", serialize_stage.elapsed())
        .await;
    let mut out_headers = HeaderMap::new();
    put_cache_headers(&mut out_headers, state.api.sequence_ttl, &etag);
    out_headers.insert(
        "content-type",
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    if let Some(enc) = encoding {
        out_headers.insert("content-encoding", HeaderValue::from_static(enc));
    }
    let resp = (StatusCode::OK, out_headers, encoded).into_response();
    state
        .metrics
        .observe_request(route, StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

pub(crate) async fn sequence_region_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let Some(region) = params.get("region").cloned() else {
        let resp = api_error_response(
            StatusCode::BAD_REQUEST,
            ApiError::invalid_param("region", "missing"),
        );
        return with_request_id(resp, "req-missing-region");
    };
    sequence_common(state, headers, params, "/v1/sequence/region", region).await
}

pub(crate) async fn gene_sequence_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(gene_id): AxumPath<String>,
    axum::extract::Query(mut params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let request_id = crate::http::handlers::propagated_request_id(&headers, &state);
    let dataset = match parse_dataset(&params) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            return with_request_id(resp, &request_id);
        }
    };
    let conn = match state.cache.open_dataset_connection(&dataset).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "dataset unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            return with_request_id(resp, &request_id);
        }
    };
    let row = {
        let mut stmt = match conn
            .conn
            .prepare("SELECT seqid,start,end FROM gene_summary WHERE gene_id = ?1 LIMIT 1")
        {
            Ok(v) => v,
            Err(e) => {
                let resp = api_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_json(
                        ApiErrorCode::Internal,
                        "query prepare failed",
                        json!({"message": e.to_string()}),
                    ),
                );
                return with_request_id(resp, &request_id);
            }
        };
        match stmt.query_row([gene_id.as_str()], |r| {
            let seqid: String = r.get(0)?;
            let start: u64 = r.get(1)?;
            let end: u64 = r.get(2)?;
            Ok((seqid, start, end))
        }) {
            Ok(v) => v,
            Err(_) => {
                let resp = api_error_response(
                    StatusCode::NOT_FOUND,
                    error_json(
                        ApiErrorCode::InvalidQueryParameter,
                        "gene not found",
                        json!({"gene_id": gene_id}),
                    ),
                );
                return with_request_id(resp, &request_id);
            }
        }
    };
    let flank = params
        .get("flank")
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(0);
    let region = format!(
        "{}:{}-{}",
        row.0,
        row.1.saturating_sub(flank).max(1),
        row.2.saturating_add(flank)
    );
    params.insert("region".to_string(), region.clone());
    sequence_common(
        state,
        headers,
        params,
        "/v1/genes/:gene_id/sequence",
        region,
    )
    .await
}
