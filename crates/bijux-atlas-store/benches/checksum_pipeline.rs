use bijux_atlas_core::sha256_hex;
use bijux_atlas_store::{verify_expected_sha256, ManifestLock};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_checksum_pipeline(c: &mut Criterion) {
    let manifest_bytes = b"{\"manifest_version\":\"1\"}".to_vec();
    let sqlite_bytes = vec![42u8; 1024 * 256];

    c.bench_function("manifest_lock_and_verify", |b| {
        b.iter(|| {
            let lock =
                ManifestLock::from_bytes(black_box(&manifest_bytes), black_box(&sqlite_bytes));
            let expected_manifest = sha256_hex(&manifest_bytes);
            let expected_sqlite = sha256_hex(&sqlite_bytes);
            verify_expected_sha256(&manifest_bytes, &expected_manifest).expect("manifest hash");
            verify_expected_sha256(&sqlite_bytes, &expected_sqlite).expect("sqlite hash");
            lock.validate(&manifest_bytes, &sqlite_bytes)
                .expect("lock validate");
        })
    });
}

criterion_group!(benches, bench_checksum_pipeline);
criterion_main!(benches);
