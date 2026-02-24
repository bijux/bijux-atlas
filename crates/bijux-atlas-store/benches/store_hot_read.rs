use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ManifestStats,
};
use bijux_atlas_store::{canonical_catalog_json, ArtifactStore, LocalFsStore};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn dataset() -> DatasetId {
    DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset")
}

fn manifest(dataset: DatasetId) -> ArtifactManifest {
    let mut m = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        dataset,
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64),
        ),
        ManifestStats::new(1, 1, 1),
    );
    m.db_hash = m.checksums.sqlite_sha256.clone();
    m.artifact_hash = m.checksums.sqlite_sha256.clone();
    m.input_hashes.gff3_sha256 = "a".repeat(64);
    m.input_hashes.fasta_sha256 = "b".repeat(64);
    m.input_hashes.fai_sha256 = "c".repeat(64);
    m.input_hashes.policy_sha256 = "d".repeat(64);
    m.toolchain_hash = "e".repeat(64);
    m
}

fn bench_store_hot_read(c: &mut Criterion) {
    let root = tempdir().expect("tmp");
    let store = LocalFsStore::new(root.path().to_path_buf());
    let ds = dataset();
    let manifest = manifest(ds.clone());
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest bytes");
    let sqlite = b"sqlite".to_vec();

    store
        .put_dataset(
            &ds,
            &manifest_bytes,
            &sqlite,
            &sha256_hex(&manifest_bytes),
            &sha256_hex(&sqlite),
        )
        .expect("publish");

    let catalog = Catalog::new(vec![CatalogEntry::new(
        ds.clone(),
        "110/homo_sapiens/GRCh38/manifest.json".to_string(),
        "110/homo_sapiens/GRCh38/gene_summary.sqlite".to_string(),
    )]);
    let catalog_text = canonical_catalog_json(&catalog).expect("catalog text");
    std::fs::write(root.path().join("catalog.json"), catalog_text).expect("write catalog");

    c.bench_function("store_hot_read_manifest", |b| {
        b.iter(|| store.get_manifest(black_box(&ds)).expect("manifest"))
    });

    c.bench_function("store_hot_read_catalog_list", |b| {
        b.iter(|| store.list_datasets().expect("datasets"))
    });
}

criterion_group!(benches, bench_store_hot_read);
criterion_main!(benches);
