// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_store::{ArtifactStore, LocalFsStore};
#[cfg(feature = "backend-s3")]
use bijux_atlas_store::HttpReadonlyStore;
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

#[test]
fn local_backend_roundtrip_is_hermetic() {
    let root = tempdir().expect("tmp");
    let store = LocalFsStore::new(root.path().to_path_buf());
    let ds = dataset();
    let manifest = manifest(ds.clone());

    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest bytes");
    let sqlite_bytes = b"sqlite".to_vec();

    store
        .put_dataset(
            &ds,
            &manifest_bytes,
            &sqlite_bytes,
            &sha256_hex(&manifest_bytes),
            &sha256_hex(&sqlite_bytes),
        )
        .expect("publish");

    let loaded = store.get_manifest(&ds).expect("manifest");
    assert_eq!(loaded.dataset, ds);
}

#[test]
#[cfg(feature = "backend-s3")]
fn http_backend_reads_from_hermetic_cached_objects() {
    let cache = tempdir().expect("cache");
    let ds = dataset();
    let m = manifest(ds.clone());
    let manifest_bytes = serde_json::to_vec(&m).expect("manifest json");
    let lock = bijux_atlas_store::ManifestLock::from_bytes(&manifest_bytes, b"sqlite");
    let lock_bytes = serde_json::to_vec(&lock).expect("lock bytes");

    let catalog_json = "{\"model_version\":\"v1\",\"datasets\":[{\"dataset\":{\"release\":\"110\",\"species\":\"homo_sapiens\",\"assembly\":\"GRCh38\"},\"manifest_path\":\"110/homo_sapiens/GRCh38/manifest.json\",\"sqlite_path\":\"110/homo_sapiens/GRCh38/gene_summary.sqlite\"}]}";

    std::fs::write(cache.path().join("catalog.json"), catalog_json).expect("catalog cache");
    std::fs::write(
        cache
            .path()
            .join("110__homo_sapiens__GRCh38__manifest.json"),
        &manifest_bytes,
    )
    .expect("manifest cache");
    std::fs::write(
        cache
            .path()
            .join("110__homo_sapiens__GRCh38__manifest.lock"),
        &lock_bytes,
    )
    .expect("lock cache");
    std::fs::write(
        cache
            .path()
            .join("110__homo_sapiens__GRCh38__gene_summary.sqlite"),
        b"sqlite",
    )
    .expect("sqlite cache");

    let store = HttpReadonlyStore::new("http://example.invalid".to_string())
        .with_cache(cache.path().to_path_buf(), true);

    let datasets = store.list_datasets().expect("list");
    assert_eq!(datasets.len(), 1);

    let manifest_loaded = store.get_manifest(&ds).expect("manifest");
    assert_eq!(manifest_loaded.dataset, ds);

    let sqlite = store.get_sqlite_bytes(&dataset()).expect("sqlite");
    assert_eq!(sqlite, b"sqlite");
}
