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
    let download_ttfb = state
        .cache
        .metrics
        .store_download_ttfb_ns
        .lock()
        .await
        .clone();
    let download_bytes_total = state
        .cache
        .metrics
        .store_download_bytes_total
        .load(Ordering::Relaxed);
    let total_download_ns: u128 = download_lat.iter().map(|x| *x as u128).sum();
    let throughput_bps = if total_download_ns == 0 {
        0.0
    } else {
        (download_bytes_total as f64) / (total_download_ns as f64 / 1_000_000_000.0)
    };
    body.push_str(&format!(
        "bijux_runtime_policy_hash{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        u64::from_str_radix(
            &state
                .runtime_policy_hash
                .chars()
                .take(16)
                .collect::<String>(),
            16
        )
        .unwrap_or(0)
    ));
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
bijux_store_download_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n\
bijux_store_download_ttfb_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n\
bijux_store_download_throughput_bytes_per_second{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.3}\n\
bijux_store_download_retry_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_error_checksum_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_error_timeout_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_error_network_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_error_other_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        percentile_ns(&open_lat, 0.95) as f64 / 1_000_000_000.0,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        percentile_ns(&download_lat, 0.95) as f64 / 1_000_000_000.0,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        percentile_ns(&download_ttfb, 0.95) as f64 / 1_000_000_000.0,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        throughput_bps,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_download_retry_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_error_checksum_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_error_timeout_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_error_network_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .store_error_other_total
            .load(Ordering::Relaxed)
    ));
    let mut store_error_by = state
        .cache
        .metrics
        .store_errors_by_backend_and_class
        .lock()
        .await
        .iter()
        .map(|((backend, class), count)| (backend.clone(), class.clone(), *count))
        .collect::<Vec<_>>();
    store_error_by.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    for (backend, class, count) in store_error_by {
        body.push_str(&format!(
            "bijux_store_errors_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",backend=\"{}\",class=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, backend, class, count
        ));
    }
    let heavy_cap = state.api.concurrency_heavy as u64;
    let heavy_avail = state.class_heavy.available_permits() as u64;
    let heavy_inflight = heavy_cap.saturating_sub(heavy_avail);
    let shedding_active = if state.api.shed_load_enabled
        && state
            .metrics
            .should_shed_heavy(
                state.api.shed_latency_min_samples,
                state.api.shed_latency_p95_threshold_ms,
            )
            .await
    {
        1
    } else {
        0
    };
    let cached_only_mode = if state.cache.cached_only_mode() { 1 } else { 0 };
    let draining_mode = if !state.accepting_requests.load(Ordering::Relaxed) {
        1
    } else {
        0
    };
    let store_breaker_open = if state.cache.store_breaker_is_open().await {
        1
    } else {
        0
    };
    let queue_depth = state.queued_requests.load(Ordering::Relaxed);
    let disk_io_p95 = {
        let mut v = state.cache.metrics.disk_io_latency_ns.lock().await.clone();
        if v.is_empty() {
            0_u64
        } else {
            v.sort_unstable();
            let idx = ((v.len() as f64) * 0.95).ceil() as usize - 1;
            v[idx.min(v.len() - 1)]
        }
    };
    body.push_str(&format!(
        "bijux_inflight_heavy_queries{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_overload_shedding_active{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_cheap_queries_served_while_overloaded_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_cached_only_mode{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_draining_mode{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_breaker_open{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_request_queue_depth{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_disk_io_latency_p95_ns{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_fs_space_pressure_events_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        heavy_inflight,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shedding_active,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .cheap_queries_served_while_overloaded_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        cached_only_mode,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        draining_mode,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        store_breaker_open,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        queue_depth,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        disk_io_p95,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .fs_space_pressure_events_total
            .load(Ordering::Relaxed)
    ));
    body.push_str(&format!(
        "bijux_registry_invalidation_events_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_registry_freeze_mode{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .registry_invalidation_events_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        if state.cache.registry_freeze_mode() { 1 } else { 0 }
    ));
    if let Some(redis) = &state.redis_backend {
        let redis_metrics = redis.metrics_snapshot().await;
        body.push_str(&format!(
            "bijux_redis_cache_hits_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_cache_misses_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_read_fallback_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_write_fallback_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_rate_limit_fallback_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_breaker_open_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_breaker_reject_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_cache_key_reject_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_cache_cardinality_reject_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_redis_cache_tracked_keys{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.hits,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.misses,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.read_fallbacks,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.write_fallbacks,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.rate_limit_fallbacks,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.breaker_open_total,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.breaker_reject_total,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.key_reject_total,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.cardinality_reject_total,
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            redis_metrics.tracked_keys
        ));
    }
    for code in [
        "InvalidQueryParameter",
        "MissingDatasetDimension",
        "InvalidCursor",
        "QueryRejectedByPolicy",
        "RateLimited",
        "Timeout",
        "PayloadTooLarge",
        "ResponseTooLarge",
        "NotReady",
        "Internal",
    ] {
        body.push_str(&format!(
            "bijux_errors_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",code=\"{}\"}} 0\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, code
        ));
    }

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
