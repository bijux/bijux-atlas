use crate::AppState;
use bijux_atlas_query::QueryClass;

pub async fn should_shed_noncheap(state: &AppState, class: QueryClass) -> bool {
    if class == QueryClass::Cheap {
        return false;
    }
    if !state.api.enable_cheap_only_survival {
        return false;
    }
    state
        .metrics
        .should_shed_heavy(
            state.api.shed_latency_min_samples,
            state.api.shed_latency_p95_threshold_ms,
        )
        .await
}
