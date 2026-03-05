// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::contracts::reproducibility::scenario_catalog;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_reproducibility_scenario_catalog(c: &mut Criterion) {
    c.bench_function("reproducibility_scenario_catalog", |b| {
        b.iter(|| {
            let rows = scenario_catalog();
            black_box(rows.len())
        });
    });
}

criterion_group!(reproducibility, bench_reproducibility_scenario_catalog);
criterion_main!(reproducibility);
