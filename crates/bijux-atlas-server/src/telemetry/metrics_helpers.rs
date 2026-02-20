use crate::*;

pub const METRIC_SUBSYSTEM: &str = "atlas";
pub const METRIC_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const METRIC_DATASET_ALL: &str = "all";

pub(super) fn percentile_ns(values: &[u64], pct: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    let idx = ((v.len() as f64 - 1.0) * pct).round() as usize;
    v[idx]
}

pub(super) fn push_histogram_from_samples(
    body: &mut String,
    metric_name: &str,
    base_labels: &str,
    samples_ns: &[u64],
    bounds_seconds: &[f64],
) {
    let mut count_le = vec![0_u64; bounds_seconds.len()];
    let mut sum_seconds = 0.0_f64;
    for sample in samples_ns {
        let seconds = *sample as f64 / 1_000_000_000.0;
        sum_seconds += seconds;
        for (i, bound) in bounds_seconds.iter().enumerate() {
            if seconds <= *bound {
                count_le[i] += 1;
            }
        }
    }
    for (i, bound) in bounds_seconds.iter().enumerate() {
        body.push_str(&format!(
            "{metric_name}_bucket{{{base_labels},le=\"{bound}\"}} {}\n",
            count_le[i]
        ));
    }
    body.push_str(&format!(
        "{metric_name}_bucket{{{base_labels},le=\"+Inf\"}} {}\n",
        samples_ns.len()
    ));
    body.push_str(&format!(
        "{metric_name}_sum{{{base_labels}}} {sum_seconds:.9}\n"
    ));
    body.push_str(&format!(
        "{metric_name}_count{{{base_labels}}} {}\n",
        samples_ns.len()
    ));
}

pub(super) fn make_request_id(state: &AppState) -> String {
    let id = state.request_id_seed.fetch_add(1, Ordering::Relaxed);
    format!("req-{id:016x}")
}

pub(super) fn shed_reason_class(reason: &str) -> &'static str {
    match reason {
        "bulkhead_shed_heavy" | "heavy_worker_saturated" => "heavy",
        "bulkhead_shed_noncheap" | "class_permit_saturated" | "queue_depth_exceeded" => "standard",
        _ => "cheap",
    }
}

pub(super) fn with_request_id(mut response: Response, request_id: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", v);
    }
    response
}
