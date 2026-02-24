// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use bijux_atlas_query::QueryClass;

pub async fn overloaded(state: &AppState) -> bool {
    let latency_overloaded = state
        .metrics
        .should_shed_heavy(
            state.api.shed_latency_min_samples,
            state.api.shed_latency_p95_threshold_ms,
        )
        .await;
    latency_overloaded || memory_pressure_overloaded(state)
}

pub async fn should_shed_noncheap(state: &AppState, class: QueryClass) -> bool {
    if class == QueryClass::Cheap {
        return false;
    }
    if !state.api.enable_cheap_only_survival {
        return false;
    }
    overloaded(state).await
}

pub fn memory_pressure_overloaded(state: &AppState) -> bool {
    if !state.api.memory_pressure_shed_enabled {
        return false;
    }
    current_rss_bytes()
        .map(|rss| rss >= state.api.memory_pressure_rss_bytes)
        .unwrap_or(false)
}

#[must_use]
pub fn heavy_backoff_ms(state: &AppState) -> u64 {
    let cap = state.api.concurrency_heavy.max(1) as u64;
    let inflight = cap.saturating_sub(state.class_heavy.available_permits() as u64);
    let level = inflight.saturating_mul(4) / cap;
    25_u64.saturating_mul(2_u64.saturating_pow((level.min(5)) as u32))
}

fn current_rss_bytes() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
            return Some(kb.saturating_mul(1024));
        }
    }
    None
}
