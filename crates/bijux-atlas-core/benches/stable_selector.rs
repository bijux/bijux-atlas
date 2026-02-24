use bijux_atlas_core::canonical;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_stable_sort_by_key(c: &mut Criterion) {
    let data: Vec<(u32, &'static str)> = vec![
        (42, "g"),
        (7, "b"),
        (19, "d"),
        (1, "a"),
        (88, "z"),
        (33, "f"),
    ];

    c.bench_function("stable_sort_by_key_selector", |b| {
        b.iter(|| canonical::stable_sort_by_key(black_box(data.clone()), |item| item.0))
    });
}

criterion_group!(benches, bench_stable_sort_by_key);
criterion_main!(benches);
