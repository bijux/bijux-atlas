// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[path = "../../metrics_helpers.rs"]
mod metrics_helpers;
use metrics_helpers::{
    make_request_id, percentile_ns, push_histogram_from_samples, shed_reason_class,
    with_request_id, METRIC_DATASET_ALL, METRIC_SUBSYSTEM, METRIC_VERSION,
};

#[cfg(target_os = "linux")]
fn current_process_rss_bytes() -> u64 {
    let page_size = 4096_u64;
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|raw| {
            let mut parts = raw.split_whitespace();
            let _size_pages = parts.next()?;
            let resident_pages = parts.next()?.parse::<u64>().ok()?;
            Some(resident_pages.saturating_mul(page_size))
        })
        .unwrap_or(0)
}

#[cfg(not(target_os = "linux"))]
fn current_process_rss_bytes() -> u64 {
    0
}

#[cfg(target_os = "linux")]
fn current_open_fd_count() -> u64 {
    std::fs::read_dir("/proc/self/fd")
        .ok()
        .map(|entries| entries.count() as u64)
        .unwrap_or(0)
}

#[cfg(not(target_os = "linux"))]
fn current_open_fd_count() -> u64 {
    0
}

pub(crate) async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    if !state.api.enable_metrics_endpoint {
        return with_request_id(
            StatusCode::SERVICE_UNAVAILABLE.into_response(),
            &make_request_id(&state),
        );
    }
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
    let invariants_total: u64 = state
        .cache
        .metrics
        .invariant_violations_by_name
        .lock()
        .await
        .values()
        .copied()
        .sum();
    let cache_hits = state.cache.metrics.dataset_hits.load(Ordering::Relaxed);
    let cache_misses = state.cache.metrics.dataset_misses.load(Ordering::Relaxed);
    let shard_hit_rate = if cache_hits + cache_misses == 0 {
        0.0
    } else {
        cache_hits as f64 / (cache_hits + cache_misses) as f64
    };
    let shard_miss_rate = if cache_hits + cache_misses == 0 {
        0.0
    } else {
        cache_misses as f64 / (cache_hits + cache_misses) as f64
    };
    let hot_cache = state.hot_query_cache.lock().await;
    let cache_memory_usage_bytes = hot_cache.approximate_memory_bytes() as u64;
    let cache_entries = hot_cache.entry_count() as u64;
    drop(hot_cache);
    let process_memory_bytes = current_process_rss_bytes();
    let process_open_fds = current_open_fd_count();
    let process_cpu_usage_ratio = 0.0_f64;
    let shard_registry_metrics = state.shard_registry.lock().await.metrics();
    let thread_pool_usage = if state.api.heavy_worker_pool_size == 0 {
        0.0
    } else {
        (state.api.heavy_worker_pool_size - state.heavy_workers.available_permits()) as f64
            / state.api.heavy_worker_pool_size as f64
    };
    let task_backlog = queue_depth;
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
    body.push_str(&format!(
        "atlas_ingest_throughput_bytes_per_second{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.3}\n\
atlas_ingest_rows_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_ingest_rejections_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_ingest_anomalies_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_dataset_load_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n\
atlas_shard_load_current{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_shard_evictions_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_shard_hit_rate{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n\
atlas_shard_miss_rate{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n\
atlas_cache_memory_usage_bytes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_cache_entry_count{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_process_memory_bytes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_process_cpu_usage_ratio{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n\
atlas_process_open_fds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_thread_pool_usage_ratio{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",pool=\"heavy_workers\"}} {:.6}\n\
atlas_runtime_queue_depth{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_task_backlog{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_slow_queries_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        throughput_bps,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        0,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        policy_violations_total,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        invariants_total,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        percentile_ns(&open_lat, 0.95) as f64 / 1_000_000_000.0,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        heavy_inflight,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state.cache.metrics.cache_evictions_total.load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_hit_rate,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_miss_rate,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        cache_memory_usage_bytes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        cache_entries,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        process_memory_bytes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        process_cpu_usage_ratio,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        process_open_fds,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        thread_pool_usage,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        queue_depth,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        task_backlog,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state.metrics.slow_queries_total(),
    ));
    body.push_str(&format!(
        "atlas_shard_registry_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_shard_healthy_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_shard_access_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_shard_cache_hits_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_shard_cache_misses_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_shard_latency_avg_ms{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_registry_metrics.shard_count,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_registry_metrics.healthy_shard_count,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_registry_metrics.total_access_count,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_registry_metrics.total_cache_hits,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_registry_metrics.total_cache_misses,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        shard_registry_metrics.average_latency_ms
    ));
    push_histogram_from_samples(
        &mut body,
        "atlas_ingest_pipeline_stage_duration_seconds",
        &format!(
            "subsystem=\"{}\",version=\"{}\",dataset=\"{}\",stage=\"artifact_open\"",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL
        ),
        &open_lat,
        &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5],
    );
    push_histogram_from_samples(
        &mut body,
        "atlas_ingest_pipeline_stage_duration_seconds",
        &format!(
            "subsystem=\"{}\",version=\"{}\",dataset=\"{}\",stage=\"artifact_download\"",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL
        ),
        &download_lat,
        &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],
    );
    let warmup_lock_wait_p95_ns = {
        let mut v = state.cache.metrics.warmup_lock_wait_ns.lock().await.clone();
        if v.is_empty() {
            0_u64
        } else {
            v.sort_unstable();
            let idx = ((v.len() as f64) * 0.95).ceil() as usize - 1;
            v[idx.min(v.len() - 1)]
        }
    };
    body.push_str(&format!(
        "bijux_warmup_lock_contention_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_warmup_lock_expired_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
bijux_warmup_lock_wait_p95_seconds{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {:.6}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .warmup_lock_contention_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        state
            .cache
            .metrics
            .warmup_lock_expired_total
            .load(Ordering::Relaxed),
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        warmup_lock_wait_p95_ns as f64 / 1_000_000_000.0
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
    let mut dataset_distribution = state
        .metrics
        .dataset_query_distribution_snapshot()
        .await
        .into_iter()
        .collect::<Vec<_>>();
    dataset_distribution.sort_by(|a, b| a.0.cmp(&b.0));
    for (dataset_bucket, count) in dataset_distribution {
        body.push_str(&format!(
            "atlas_dataset_query_distribution_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\",dataset_bucket=\"{}\"}} {}\n",
            METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, dataset_bucket, count
        ));
    }
    let membership_metrics = state.membership.lock().await.metrics();
    body.push_str(&format!(
        "atlas_membership_nodes_total{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_membership_active_nodes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_membership_timed_out_nodes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_membership_quarantined_nodes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_membership_maintenance_nodes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_membership_draining_nodes{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n\
atlas_membership_average_load_percent{{subsystem=\"{}\",version=\"{}\",dataset=\"{}\"}} {}\n",
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        membership_metrics.total_nodes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        membership_metrics.active_nodes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        membership_metrics.timed_out_nodes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        membership_metrics.quarantined_nodes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        membership_metrics.maintenance_nodes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        membership_metrics.draining_nodes,
        METRIC_SUBSYSTEM,
        METRIC_VERSION,
        METRIC_DATASET_ALL,
        membership_metrics.average_load_percent
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

    append_request_and_latency_metrics(&state, &mut body, &download_lat).await;
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
