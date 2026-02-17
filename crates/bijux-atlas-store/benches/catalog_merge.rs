use bijux_atlas_model::{Catalog, CatalogEntry, DatasetId};
use bijux_atlas_store::merge_catalogs;
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_catalog_merge(c: &mut Criterion) {
    let mk_catalog = |release_start: u32| {
        let mut entries = Vec::new();
        for i in 0..10_000_u32 {
            let release = (release_start + i % 50).to_string();
            let species = if i % 2 == 0 {
                "homo_sapiens"
            } else {
                "mus_musculus"
            };
            let assembly = if i % 3 == 0 { "GRCh38" } else { "GRCm39" };
            let dataset = DatasetId::new(&release, species, assembly).expect("dataset");
            entries.push(CatalogEntry {
                dataset,
                manifest_sha256: format!("{:064x}", i),
            });
        }
        Catalog::new(entries)
    };

    let catalogs = vec![mk_catalog(100), mk_catalog(120), mk_catalog(140)];
    c.bench_function("catalog_merge_10k", |b| {
        b.iter(|| {
            let merged = merge_catalogs(&catalogs);
            assert!(!merged.datasets.is_empty());
        });
    });
}

criterion_group!(benches, benchmark_catalog_merge);
criterion_main!(benches);
