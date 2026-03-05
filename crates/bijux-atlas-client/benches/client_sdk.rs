// SPDX-License-Identifier: Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bijux_atlas_client::{run_with_retry, ClientError, DatasetQuery, ErrorClass};
use reqwest as _;
use serde as _;
use serde_json as _;

fn dataset_query_builder_bench(c: &mut Criterion) {
    c.bench_function("dataset_query_builder", |b| {
        b.iter(|| {
            let q = DatasetQuery::new("110", "homo_sapiens", "GRCh38")
                .with_limit(black_box(50))
                .with_gene_id("ENSG000001")
                .with_biotype("protein_coding")
                .include_biotype()
                .include_coords();
            black_box(q);
        })
    });
}

fn retry_strategy_bench(c: &mut Criterion) {
    c.bench_function("retry_strategy_success", |b| {
        b.iter(|| {
            let mut tries = 0;
            let value = run_with_retry(black_box(3), 0, || {
                tries += 1;
                if tries == 1 {
                    Err(ClientError::new(ErrorClass::Transport, "transient"))
                } else {
                    Ok(1_u32)
                }
            })
            .expect("retry success");
            black_box(value);
        })
    });
}

criterion_group!(
    client_sdk,
    dataset_query_builder_bench,
    retry_strategy_bench
);
criterion_main!(client_sdk);
