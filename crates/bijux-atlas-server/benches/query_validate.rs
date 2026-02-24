use std::sync::Arc;

use bijux_atlas_server::{
    build_router, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use criterion::{criterion_group, criterion_main, Criterion};

type RequestTuple = (
    axum::http::Method,
    &'static str,
    Vec<(&'static str, &'static str)>,
);

fn bench_query_validate_endpoint(c: &mut Criterion) {
    let store = Arc::new(FakeStore::default());
    let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
    let app = build_router(AppState::new(cache));

    c.bench_function("http.query_validate.route_build", |b| {
        b.iter(|| {
            let req: RequestTuple = (
                axum::http::Method::POST,
                "/v1/query/validate",
                vec![("content-type", "application/json")],
            );
            let _ = (&app, req);
        });
    });
}

criterion_group!(benches, bench_query_validate_endpoint);
criterion_main!(benches);
