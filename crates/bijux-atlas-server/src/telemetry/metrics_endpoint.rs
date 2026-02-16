use crate::*;

const METRIC_SUBSYSTEM: &str = "atlas";
const METRIC_VERSION: &str = env!("CARGO_PKG_VERSION");
const METRIC_DATASET_ALL: &str = "all";

fn percentile_ns(values: &[u64], pct: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    let idx = ((v.len() as f64 - 1.0) * pct).round() as usize;
    v[idx]
}

fn make_request_id(state: &AppState) -> String {
    let id = state.request_id_seed.fetch_add(1, Ordering::Relaxed);
    format!("req-{id:016x}")
}

fn with_request_id(mut response: Response, request_id: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", v);
    }
    response
}

pub(crate) async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let mut body = String::from(
        "bijux_dataset_hits{subsystem=\"%SUB%\",version=\"%VER%\",dataset=\"%DS%\"} %HITS%\n\
bijux_dataset_misses{subsystem=\"%SUB%\",version=\"%VER%\",dataset=\"%DS%\"} %MISSES%\n\
bijux_dataset_count{subsystem=\"%SUB%\",version=\"%VER%\",dataset=\"%DS%\"} %COUNT%\n\
bijux_dataset_disk_usage_bytes{subsystem=\"%SUB%\",version=\"%VER%\",dataset=\"%DS%\"} %BYTES%\n",
    )
    .replace("%SUB%", METRIC_SUBSYSTEM)
    .replace("%VER%", METRIC_VERSION)
    .replace("%DS%", METRIC_DATASET_ALL)
    .replace(
        "%HITS%",
        &state
            .cache
            .metrics
            .dataset_hits
            .load(Ordering::Relaxed)
            .to_string(),
    )
    .replace(
        "%MISSES%",
        &state
            .cache
            .metrics
            .dataset_misses
            .load(Ordering::Relaxed)
            .to_string(),
    )
    .replace(
        "%COUNT%",
        &state
            .cache
            .metrics
            .dataset_count
            .load(Ordering::Relaxed)
            .to_string(),
    )
    .replace(
        "%BYTES%",
        &state
            .cache
            .metrics
            .disk_usage_bytes
            .load(Ordering::Relaxed)
            .to_string(),
    );
    let open_lat = state
        .cache
        .metrics
        .store_open_latency_ns
        .lock()
        .await
        .clone();
    let download_lat = state
        .cache
        .metrics
        .store_download_latency_ns
        .lock()
        .await
        .clone();
    body.push_str(&format!(
        "bijux_store_open_failure_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_download_failure_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_breaker_open_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_retry_budget_exhausted_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_verify_marker_fast_path_hits_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_verify_full_hash_checks_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_open_failures
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_download_failures
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_breaker_open_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_retry_budget_exhausted_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .verify_marker_fast_path_hits
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .verify_full_hash_checks
            .load(Ordering::Relaxed),
    ));
    body.push_str(&format!(
        "bijux_store_open_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n\
bijux_store_download_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        percentile_ns(&open_lat, 0.95) as f64 / 1_000_000_000.0,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        percentile_ns(&download_lat, 0.95) as f64 / 1_000_000_000.0
    ));

    let req_counts = state.metrics.counts.lock().await.clone();
    let req_exemplars = state.metrics.exemplars.lock().await.clone();
    for ((route, status), count) in req_counts {
        if state.api.enable_exemplars {
            if let Some((trace_id, ts_ms)) = req_exemplars.get(&(route.clone(), status)) {
                body.push_str(&format!(
                    "bijux_http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",status=\"{}\"}} {} # {{trace_id=\"{}\"}} {}\n",
                    METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, status, count, trace_id, ts_ms
                ));
            } else {
                body.push_str(&format!(
                    "bijux_http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                    METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, status, count
                ));
            }
        } else {
            body.push_str(&format!(
                "bijux_http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, status, count
            ));
        }
    }
    let req_lat = state.metrics.latency_ns.lock().await.clone();
    for (route, vals) in req_lat {
        body.push_str(&format!(
            "bijux_http_request_latency_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\"}} {:.6}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            route,
            percentile_ns(&vals, 0.95) as f64 / 1_000_000_000.0
        ));
    }
    let sql_lat = state.metrics.sqlite_latency_ns.lock().await.clone();
    for (query_type, vals) in sql_lat {
        body.push_str(&format!(
            "bijux_sqlite_query_latency_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",query_type=\"{}\"}} {:.6}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            query_type,
            percentile_ns(&vals, 0.95) as f64 / 1_000_000_000.0
        ));
    }
    let stage_lat = state.metrics.stage_latency_ns.lock().await.clone();
    for (stage, vals) in stage_lat {
        body.push_str(&format!(
            "bijux_request_stage_latency_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",stage=\"{}\"}} {:.6}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            stage,
            percentile_ns(&vals, 0.95) as f64 / 1_000_000_000.0
        ));
    }
    let resp = (StatusCode::OK, body).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/metrics",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(resp, &request_id)
}
