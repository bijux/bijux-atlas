use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_store::{
    manifest_lock_path, ArtifactStore, HttpReadonlyStore, LocalFsStore, StoreErrorCode,
};
use std::fs;
use tempfile::tempdir;

fn mk_dataset() -> DatasetId {
    DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id")
}

fn mk_manifest(dataset: DatasetId) -> ArtifactManifest {
    ArtifactManifest::new(
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
    )
}

#[test]
fn local_publish_is_atomic_and_writes_manifest_lock() {
    let root = tempdir().expect("tempdir");
    let store = LocalFsStore::new(root.path().to_path_buf());
    let dataset = mk_dataset();
    let manifest = mk_manifest(dataset.clone());
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest json");
    let sqlite_bytes = b"sqlite-bytes".to_vec();

    let expected_manifest = sha256_hex(&manifest_bytes);
    let expected_sqlite = sha256_hex(&sqlite_bytes);

    store
        .put_dataset(
            &dataset,
            &manifest_bytes,
            &sqlite_bytes,
            &expected_manifest,
            &expected_sqlite,
        )
        .expect("publish dataset");

    let lock_path = manifest_lock_path(root.path(), &dataset);
    assert!(lock_path.exists(), "manifest.lock must exist");

    let loaded = store.get_manifest(&dataset).expect("read manifest");
    assert_eq!(
        loaded.dataset.canonical_string(),
        dataset.canonical_string()
    );
}

#[test]
fn local_publish_rejects_checksum_mismatch_without_finalizing() {
    let root = tempdir().expect("tempdir");
    let store = LocalFsStore::new(root.path().to_path_buf());
    let dataset = mk_dataset();
    let manifest = mk_manifest(dataset.clone());
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest json");
    let sqlite_bytes = b"sqlite-bytes".to_vec();

    let err = store
        .put_dataset(
            &dataset,
            &manifest_bytes,
            &sqlite_bytes,
            "deadbeef",
            "deadbeef",
        )
        .expect_err("checksum mismatch should fail");
    assert_eq!(err.code, StoreErrorCode::Validation);

    assert!(!store.exists(&dataset).expect("exists check"));
}

#[test]
fn cached_only_mode_never_touches_network() {
    let root = tempdir().expect("tempdir");
    let store = HttpReadonlyStore::new("http://127.0.0.1:9".to_string())
        .with_cache(root.path().to_path_buf(), true);
    let dataset = mk_dataset();

    let err = store
        .get_manifest(&dataset)
        .expect_err("cached only with empty cache must fail fast");
    assert_eq!(err.code, StoreErrorCode::CachedOnly);
}

#[test]
fn store_errors_have_stable_codes() {
    let root = tempdir().expect("tempdir");
    let store = LocalFsStore::new(root.path().to_path_buf());
    let dataset = mk_dataset();

    let err = store
        .get_manifest(&dataset)
        .expect_err("missing manifest should map to not_found");
    assert_eq!(err.code, StoreErrorCode::NotFound);
    assert!(err.to_string().contains("not_found:"));
}

#[test]
fn store_crate_has_no_server_or_axum_dependency() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    for forbidden in ["bijux-atlas-server", "axum"] {
        assert!(
            !cargo_toml.contains(forbidden),
            "forbidden dependency in store crate: {forbidden}"
        );
    }
}
