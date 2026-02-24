// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_server::{ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore};
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;

fn bench_bulkhead_tuning(c: &mut Criterion) {
    c.bench_function("bulkhead_acquire_release", |b| {
        let rt = tokio::runtime::Runtime::new().expect("rt");
        b.iter(|| {
            rt.block_on(async {
                let store = Arc::new(FakeStore::default());
                let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
                let api = ApiConfig {
                    concurrency_cheap: 64,
                    concurrency_medium: 32,
                    concurrency_heavy: 8,
                    heavy_worker_pool_size: 8,
                    ..ApiConfig::default()
                };
                let state = AppState::with_config(cache, api, Default::default());
                let cheap = state
                    .class_cheap
                    .clone()
                    .try_acquire_owned()
                    .expect("cheap");
                let medium = state
                    .class_medium
                    .clone()
                    .try_acquire_owned()
                    .expect("medium");
                let heavy = state
                    .class_heavy
                    .clone()
                    .try_acquire_owned()
                    .expect("heavy");
                drop((cheap, medium, heavy));
            });
        });
    });
}

criterion_group!(benches, bench_bulkhead_tuning);
criterion_main!(benches);
