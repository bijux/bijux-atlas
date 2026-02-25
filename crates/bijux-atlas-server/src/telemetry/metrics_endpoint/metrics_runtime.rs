// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[path = "../metrics_helpers.rs"]
mod metrics_helpers;
use metrics_helpers::{
    make_request_id, percentile_ns, push_histogram_from_samples, shed_reason_class,
    with_request_id, METRIC_DATASET_ALL, METRIC_SUBSYSTEM, METRIC_VERSION,
};

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
        "atlas_cache_hits_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",cache=\"dataset\"}} {}\n\
atlas_cache_misses_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",cache=\"dataset\"}} {}\n\
bijux_runtime_policy_hash{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .dataset_hits
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .dataset_misses
            .load(Ordering::Relaxed),
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
bijux_store_breaker_half_open_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
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
            .store_breaker_half_open_total
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
    for backend in ["http_s3", "local_fs", "federated", "unknown"] {
        for class in ["cheap", "standard", "heavy"] {
            if !store_error_by
                .iter()
                .any(|(b, c, _)| b == backend && c == class)
            {
                store_error_by.push((backend.to_string(), class.to_string(), 0));
            }
        }
    }
    store_error_by.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    for (backend, class, count) in store_error_by {
        body.push_str(&format!(
            "bijux_store_errors_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",backend=\"{}\",class=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, backend, class, count
        ));
        body.push_str(&format!(
            "atlas_store_errors_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",backend=\"{}\",class=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, backend, class, count
        ));
    }
    let heavy_cap = state.api.concurrency_heavy as u64;
    let heavy_avail = state.class_heavy.available_permits() as u64;
    let heavy_inflight = heavy_cap.saturating_sub(heavy_avail);
    let cheap_cap = state.api.concurrency_cheap as u64;
    let cheap_avail = state.class_cheap.available_permits() as u64;
    let cheap_inflight = cheap_cap.saturating_sub(cheap_avail);
    let medium_cap = state.api.concurrency_medium as u64;
    let medium_avail = state.class_medium.available_permits() as u64;
    let medium_inflight = medium_cap.saturating_sub(medium_avail);
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
    let policy_mode = state.runtime_policy_mode.as_str();
    let policy_relaxation_active = if policy_mode == "strict" { 0 } else { 1 };
    let policy_violations_total = state
        .cache
        .metrics
        .policy_violations_total
        .load(Ordering::Relaxed);
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
atlas_bulkhead_inflight{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",class=\"cheap\"}} {}\n\
atlas_bulkhead_inflight{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",class=\"medium\"}} {}\n\
atlas_bulkhead_inflight{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",class=\"heavy\"}} {}\n\
atlas_bulkhead_saturation{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",class=\"cheap\"}} {:.6}\n\
atlas_bulkhead_saturation{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",class=\"medium\"}} {:.6}\n\
atlas_bulkhead_saturation{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",class=\"heavy\"}} {:.6}\n\
bijux_overload_shedding_active{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_overload_active{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_cheap_queries_served_while_overloaded_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_cached_only_mode{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_draining_mode{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_breaker_open{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_store_breaker_open_current{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_request_queue_depth{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_policy_relaxation_active{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",mode=\"{}\"}} {}\n\
atlas_policy_violations_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",policy=\"all\"}} {}\n\
bijux_disk_io_latency_p95_ns{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_fs_space_pressure_events_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        heavy_inflight,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        cheap_inflight,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        medium_inflight,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        heavy_inflight,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        if cheap_cap == 0 {
            0.0
        } else {
            cheap_inflight as f64 / cheap_cap as f64
        },
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        if medium_cap == 0 {
            0.0
        } else {
            medium_inflight as f64 / medium_cap as f64
        },
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        if heavy_cap == 0 {
            0.0
        } else {
            heavy_inflight as f64 / heavy_cap as f64
        },
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shedding_active,
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
        store_breaker_open,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        queue_depth,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        policy_mode,
        policy_relaxation_active,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        policy_violations_total,
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
    let mut policy_counts = state
        .cache
        .metrics
        .policy_violations_by_policy
        .lock()
        .await
        .clone()
        .into_iter()
        .collect::<Vec<_>>();
    policy_counts.sort_by(|a, b| a.0.cmp(&b.0));
    for (policy, count) in policy_counts {
        body.push_str(&format!(
            "atlas_policy_violations_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",policy=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, policy, count
        ));
    }
    let shed_counts_map = state
        .cache
        .metrics
        .shed_total_by_reason
        .lock()
        .await
        .clone();
    for reason in [
        "queue_depth_exceeded",
        "class_permit_saturated",
        "bulkhead_shed_heavy",
        "bulkhead_shed_noncheap",
        "draining",
        "ip_rate_limited",
        "api_key_rate_limited",
        "heavy_worker_saturated",
    ] {
        let count = *shed_counts_map.get(reason).unwrap_or(&0);
        let class = shed_reason_class(reason);
        body.push_str(&format!(
            "atlas_shed_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",reason=\"{}\",class=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, reason, class, count
        ));
    }
    let mut shed_counts = shed_counts_map.into_iter().collect::<Vec<_>>();
    shed_counts.sort_by(|a, b| a.0.cmp(&b.0));
    for (reason, count) in shed_counts {
        if [
            "queue_depth_exceeded",
            "class_permit_saturated",
            "bulkhead_shed_heavy",
            "bulkhead_shed_noncheap",
            "draining",
            "ip_rate_limited",
            "api_key_rate_limited",
            "heavy_worker_saturated",
        ]
        .contains(&reason.as_str())
        {
            continue;
        }
        body.push_str(&format!(
            "atlas_shed_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",reason=\"{}\",class=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            reason,
            shed_reason_class(&reason),
            count
        ));
    }
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
    let client_fingerprints = state.metrics.client_fingerprint_counts.lock().await.clone();
    for ((route, method, status, class), count) in req_counts {
        if state.api.enable_exemplars {
            if let Some((trace_id, ts_ms)) =
                req_exemplars.get(&(route.clone(), method.clone(), status, class.clone()))
            {
                body.push_str(&format!(
                    "http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",method=\"{}\",status=\"{}\",class=\"{}\"}} {} # {{trace_id=\"{}\"}} {}\n\
bijux_http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",method=\"{}\",status=\"{}\",class=\"{}\"}} {} # {{trace_id=\"{}\"}} {}\n",
                    METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, method, status, class, count, trace_id, ts_ms,
                    METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, method, status, class, count, trace_id, ts_ms
                ));
            } else {
                body.push_str(&format!(
                    "http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",method=\"{}\",status=\"{}\",class=\"{}\"}} {}\n\
bijux_http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",method=\"{}\",status=\"{}\",class=\"{}\"}} {}\n",
                    METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, method, status, class, count,
                    METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, method, status, class, count
                ));
            }
        } else {
            body.push_str(&format!(
                "http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",method=\"{}\",status=\"{}\",class=\"{}\"}} {}\n\
bijux_http_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",method=\"{}\",status=\"{}\",class=\"{}\"}} {}\n",
                METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, method, status, class, count,
                METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, method, status, class, count
            ));
        }
    }
    let mut client_fingerprints_sorted = client_fingerprints.into_iter().collect::<Vec<_>>();
    client_fingerprints_sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for ((client_type, ua_family), count) in client_fingerprints_sorted {
        body.push_str(&format!(
            "atlas_client_requests_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",client_type=\"{}\",user_agent_family=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, client_type, ua_family, count
        ));
    }
    let req_lat = state.metrics.latency_ns.lock().await.clone();
    for (route, vals) in req_lat {
        let class = crate::route_sli_class(&route);
        body.push_str(&format!(
            "bijux_http_request_latency_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\"}} {:.6}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            route,
            percentile_ns(&vals, 0.95) as f64 / 1_000_000_000.0
        ));
        push_histogram_from_samples(
            &mut body,
            "http_request_duration_seconds",
            &format!(
                "subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\",class=\"{}\"",
                METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, route, class
            ),
            &vals,
            &[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],
        );
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
    let req_sizes = state.metrics.request_size_bytes.lock().await.clone();
    for (route, vals) in req_sizes {
        body.push_str(&format!(
            "bijux_http_request_size_p95_bytes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\"}} {:.3}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            route,
            percentile_ns(&vals, 0.95) as f64
        ));
    }
    let resp_sizes = state.metrics.response_size_bytes.lock().await.clone();
    for (route, vals) in resp_sizes {
        body.push_str(&format!(
            "bijux_http_response_size_p95_bytes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",route=\"{}\"}} {:.3}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            route,
            percentile_ns(&vals, 0.95) as f64
        ));
    }
    for backend in ["http_s3", "local_fs", "federated", "unknown"] {
        body.push_str(&format!(
            "bijux_store_fetch_latency_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",backend=\"{}\"}} {:.6}\n",
            METRIC_SUBSYSTEM,
            METRIC_VERSION,
            METRIC_DATASET_ALL,
            backend,
            percentile_ns(&download_lat, 0.95) as f64 / 1_000_000_000.0
        ));
        push_histogram_from_samples(
            &mut body,
            "atlas_store_request_duration_seconds",
            &format!(
                "subsystem=\"{}\",version=\"{}\",dataset=\"{}\",backend=\"{}\"",
                METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, backend
            ),
            &download_lat,
            &[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        );
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
    let registry_refresh_age_seconds = state.cache.registry_refresh_age_seconds().await;
    body.push_str(&format!(
        "atlas_registry_refresh_age_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_registry_refresh_failures_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        registry_refresh_age_seconds,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .registry_refresh_failures_total
            .load(Ordering::Relaxed)
    ));
    let mut dataset_missing = state
        .cache
        .metrics
        .dataset_missing_by_hash_bucket
        .lock()
        .await
        .iter()
        .map(|(bucket, count)| (bucket.clone(), *count))
        .collect::<Vec<_>>();
    dataset_missing.sort_by(|a, b| a.0.cmp(&b.0));
    for (dataset_hash, count) in dataset_missing {
        body.push_str(&format!(
            "atlas_dataset_missing_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",dataset_hash=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, dataset_hash, count
        ));
    }
    let mut invariants = state
        .cache
        .metrics
        .invariant_violations_by_name
        .lock()
        .await
        .iter()
        .map(|(name, count)| (name.clone(), *count))
        .collect::<Vec<_>>();
    if invariants.is_empty() {
        invariants.push(("genes-count-vs-list".to_string(), 0));
    }
    invariants.sort_by(|a, b| a.0.cmp(&b.0));
    for (invariant, count) in invariants {
        body.push_str(&format!(
            "atlas_invariant_violations_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",invariant=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, invariant, count
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
