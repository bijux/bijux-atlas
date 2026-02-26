async fn append_request_and_latency_metrics(
    state: &AppState,
    body: &mut String,
    download_lat: &[u64],
) {
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
            body,
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
            percentile_ns(download_lat, 0.95) as f64 / 1_000_000_000.0
        ));
        push_histogram_from_samples(
            body,
            "atlas_store_request_duration_seconds",
            &format!(
                "subsystem=\"{}\",version=\"{}\",dataset=\"{}\",backend=\"{}\"",
                METRIC_SUBSYSTEM, METRIC_VERSION, METRIC_DATASET_ALL, backend
            ),
            download_lat,
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
}
