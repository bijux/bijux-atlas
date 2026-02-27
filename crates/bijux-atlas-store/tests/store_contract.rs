// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ManifestStats,
};
use bijux_atlas_store::{
    dataset_artifact_paths, manifest_lock_path, merge_catalogs, validate_backend_compiled,
    ArtifactStore, BackendKind, LocalFsStore, StoreErrorCode, StoreMetricsCollector,
};
#[cfg(feature = "backend-s3")]
use bijux_atlas_store::{HttpReadonlyStore, S3LikeStore};
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;
#[cfg(feature = "backend-s3")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "backend-s3")]
use std::thread;
#[cfg(feature = "backend-s3")]
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn mk_dataset() -> DatasetId {
    DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id")
}

fn mk_manifest(dataset: DatasetId) -> ArtifactManifest {
    let mut manifest = ArtifactManifest::new(
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
    manifest.db_hash = manifest.checksums.sqlite_sha256.clone();
    manifest.artifact_hash = manifest.checksums.sqlite_sha256.clone();
    manifest
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
fn local_publish_rejects_overwrite_of_existing_dataset() {
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
        .expect("initial publish");

    let err = store
        .put_dataset(
            &dataset,
            &manifest_bytes,
            b"new-sqlite-bytes",
            &expected_manifest,
            &sha256_hex(b"new-sqlite-bytes"),
        )
        .expect_err("overwrite must be rejected");
    assert_eq!(err.code, StoreErrorCode::Conflict);
    assert!(
        err.message.contains("must not be overwritten"),
        "immutability error message should be explicit: {}",
        err.message
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
#[cfg(feature = "backend-s3")]
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
    for forbidden in ["bijux-atlas-server", "axum", "tokio"] {
        assert!(
            !cargo_toml.contains(forbidden),
            "forbidden dependency in store crate: {forbidden}"
        );
    }
}

#[test]
fn store_error_code_maps_to_core_error_code() {
    use bijux_atlas_core::ErrorCode;
    assert_eq!(
        StoreErrorCode::CachedOnly.as_error_code(),
        ErrorCode::NotReady
    );
    assert_eq!(
        StoreErrorCode::Validation.as_error_code(),
        ErrorCode::InvalidQueryParameter
    );
}

#[test]
fn runtime_backend_validation_reports_feature_requirements() {
    let local = validate_backend_compiled(BackendKind::Local);
    assert!(local.is_ok());

    let s3 = validate_backend_compiled(BackendKind::S3Like);
    #[cfg(feature = "backend-s3")]
    assert!(s3.is_ok());
    #[cfg(not(feature = "backend-s3"))]
    assert!(
        s3
            .expect_err("s3 backend should be unavailable without backend-s3 feature")
            .contains("backend-s3"),
        "error must explain how to enable backend"
    );
}

#[test]
fn verified_sqlite_read_rejects_checksum_mismatch() {
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

    let sqlite_path = dataset_artifact_paths(root.path(), &dataset).sqlite;
    fs::write(sqlite_path, b"tampered").expect("tamper sqlite");

    let err = store
        .get_sqlite_bytes_verified(&dataset)
        .expect_err("checksum mismatch must fail");
    assert_eq!(err.code, StoreErrorCode::Validation);
}

#[test]
fn deterministic_catalog_merge_keeps_stable_ordering() {
    let dataset_a = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let dataset_b = DatasetId::new("111", "homo_sapiens", "GRCh38").expect("dataset");
    let dataset_c = DatasetId::new("112", "mus_musculus", "GRCm39").expect("dataset");
    let c1 = Catalog::new(vec![
        CatalogEntry::new(
            dataset_b.clone(),
            "111/homo_sapiens/GRCh38/manifest.json".to_string(),
            "111/homo_sapiens/GRCh38/gene_summary.sqlite".to_string(),
        ),
        CatalogEntry::new(
            dataset_a.clone(),
            "110/homo_sapiens/GRCh38/manifest.json".to_string(),
            "110/homo_sapiens/GRCh38/gene_summary.sqlite".to_string(),
        ),
    ]);
    let c2 = Catalog::new(vec![
        CatalogEntry::new(
            dataset_a,
            "override/manifest.json".to_string(),
            "override/gene_summary.sqlite".to_string(),
        ),
        CatalogEntry::new(
            dataset_c.clone(),
            "112/mus_musculus/GRCm39/manifest.json".to_string(),
            "112/mus_musculus/GRCm39/gene_summary.sqlite".to_string(),
        ),
    ]);
    let merged = merge_catalogs(&[c1, c2]);
    let ids: Vec<String> = merged
        .datasets
        .iter()
        .map(|entry| entry.dataset.canonical_string())
        .collect();
    assert_eq!(
        ids,
        vec![
            "110/homo_sapiens/GRCh38".to_string(),
            "111/homo_sapiens/GRCh38".to_string(),
            "112/mus_musculus/GRCm39".to_string(),
        ]
    );
    assert_eq!(
        merged.datasets[0].manifest_path,
        "110/homo_sapiens/GRCh38/manifest.json".to_string()
    );
    assert_eq!(
        merged
            .datasets
            .iter()
            .find(|entry| entry.dataset == dataset_c)
            .expect("dataset c exists")
            .manifest_path,
        "112/mus_musculus/GRCm39/manifest.json".to_string()
    );
}

#[test]
fn deterministic_catalog_merge_scales_with_stable_output() {
    let mut primary = Vec::new();
    let mut secondary = Vec::new();
    for i in (100..1300).rev() {
        let dataset = DatasetId::new(&i.to_string(), "homo_sapiens", "GRCh38").expect("dataset id");
        let canonical_root = format!("release={i}/species=homo_sapiens/assembly=GRCh38/derived");
        primary.push(CatalogEntry::new(
            dataset.clone(),
            format!("{canonical_root}/manifest.json"),
            format!("{canonical_root}/gene_summary.sqlite"),
        ));
        if i % 3 == 0 {
            secondary.push(CatalogEntry::new(
                dataset,
                format!("override/{i}/manifest.json"),
                format!("override/{i}/gene_summary.sqlite"),
            ));
        }
    }
    let c1 = Catalog::new(primary);
    let c2 = Catalog::new(secondary);

    let merged1 = merge_catalogs(&[c1.clone(), c2.clone()]);
    let merged2 = merge_catalogs(&[c1, c2]);
    assert_eq!(merged1, merged2);
    assert_eq!(merged1.datasets.len(), 1200);
    assert!(
        merged1
            .datasets
            .windows(2)
            .all(|w| w[0].dataset.canonical_string() < w[1].dataset.canonical_string()),
        "merged catalog must be strictly sorted by canonical dataset id"
    );
    assert!(
        merged1
            .datasets
            .iter()
            .all(|e| e.manifest_path.starts_with("release=")),
        "first catalog wins for duplicate dataset IDs"
    );
}

#[cfg(feature = "backend-s3")]
fn spawn_store_http_server() -> (String, Arc<AtomicUsize>, thread::JoinHandle<()>) {
    let server = Server::http("127.0.0.1:0").expect("http server");
    let base = format!("http://{}", server.server_addr());
    let catalog_etag = "\"catalog-v1\"".to_string();
    let catalog_requests = Arc::new(AtomicUsize::new(0));
    let catalog_requests_clone = Arc::clone(&catalog_requests);
    let handle = thread::spawn(move || loop {
        let req = match server.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok(Some(req)) => req,
            Ok(None) => break,
            Err(_) => break,
        };
        let url = req.url().to_string();
        if req.method() != &Method::Get {
            let _ = req.respond(Response::empty(StatusCode(405)));
            continue;
        }
        if url == "/catalog.json" || url == "/atlas/catalog.json" {
            let mut if_none_match_matches = false;
            for h in req.headers() {
                if h.field.equiv("If-None-Match") && h.value.as_str() == catalog_etag.as_str() {
                    if_none_match_matches = true;
                }
            }
            if if_none_match_matches {
                let _ = req.respond(Response::empty(StatusCode(304)).with_header(
                    Header::from_bytes("ETag", catalog_etag.as_bytes()).expect("etag header"),
                ));
                continue;
            }
            let body = serde_json::to_vec(&Catalog::new(vec![CatalogEntry::new(
                DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
                "110/homo_sapiens/GRCh38/manifest.json".to_string(),
                "110/homo_sapiens/GRCh38/gene_summary.sqlite".to_string(),
            )]))
            .expect("catalog json");
            let _ = req.respond(Response::from_data(body).with_header(
                Header::from_bytes("ETag", catalog_etag.as_bytes()).expect("etag header"),
            ));
            catalog_requests_clone.fetch_add(1, Ordering::Relaxed);
        } else {
            let _ = req.respond(Response::empty(StatusCode(404)));
        }
    });
    (base, catalog_requests, handle)
}

#[test]
#[cfg(feature = "backend-s3")]
fn s3_store_uses_etag_cache_and_handles_304_for_catalog() {
    let (base, catalog_requests, handle) = spawn_store_http_server();

    let cache = tempdir().expect("cache");
    let store =
        S3LikeStore::new(base, "atlas".to_string()).with_cache(cache.path().to_path_buf(), false);
    let first = store.list_datasets().expect("first catalog fetch");
    assert_eq!(first.len(), 1);
    let second = store.list_datasets().expect("second catalog fetch");
    assert_eq!(second.len(), 1);

    assert_eq!(catalog_requests.load(Ordering::Relaxed), 1);
    drop(store);
    handle.join().expect("server thread");
}

#[test]
fn random_publish_failures_do_not_create_partial_dataset() {
    let root = tempdir().expect("tempdir");
    let store = LocalFsStore::new(root.path().to_path_buf());
    let dataset = mk_dataset();
    let manifest = mk_manifest(dataset.clone());
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest json");
    let sqlite_bytes = b"sqlite-bytes".to_vec();

    for attempt in 0..32_u32 {
        let bad = attempt % 2 == 0;
        let expected_manifest = sha256_hex(&manifest_bytes);
        let expected_sqlite = sha256_hex(&sqlite_bytes);
        let publish = store.put_dataset(
            &dataset,
            &manifest_bytes,
            &sqlite_bytes,
            if bad {
                "badbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadb"
            } else {
                &expected_manifest
            },
            if bad {
                "badbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadb"
            } else {
                &expected_sqlite
            },
        );
        if bad {
            assert!(publish.is_err());
            assert!(!store.exists(&dataset).expect("exists"));
        } else {
            assert!(publish.is_ok());
            break;
        }
    }
}

#[test]
fn store_metrics_collector_tracks_upload_and_failure_classes() {
    let root = tempdir().expect("tempdir");
    let metrics = Arc::new(StoreMetricsCollector::default());
    let store = LocalFsStore::new(root.path().to_path_buf()).with_instrumentation(metrics.clone());
    let dataset = mk_dataset();
    let manifest = mk_manifest(dataset.clone());
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest json");
    let sqlite_bytes = b"sqlite-bytes".to_vec();

    store
        .put_dataset(
            &dataset,
            &manifest_bytes,
            &sqlite_bytes,
            &sha256_hex(&manifest_bytes),
            &sha256_hex(&sqlite_bytes),
        )
        .expect("publish succeeds");

    let snap = metrics.snapshot();
    assert!(snap.bytes_uploaded >= (manifest_bytes.len() + sqlite_bytes.len()) as u64);
    assert!(snap.request_count >= 1);
    assert!(snap.latency_ms_total < u128::MAX);

    let bad =
        store.get_manifest(&DatasetId::new("999", "homo_sapiens", "GRCh38").expect("dataset id"));
    assert!(bad.is_err());
}

#[test]
fn fuzzish_checksum_failures_never_leave_partial_files() {
    let root = tempdir().expect("tempdir");
    let store = LocalFsStore::new(root.path().to_path_buf());
    let dataset = mk_dataset();
    let manifest = mk_manifest(dataset.clone());
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest json");
    let sqlite_bytes = b"sqlite-bytes".to_vec();
    let expected_manifest = sha256_hex(&manifest_bytes);
    let expected_sqlite = sha256_hex(&sqlite_bytes);

    let mut seed: u64 = 0x9E3779B97F4A7C15;
    let paths = dataset_artifact_paths(root.path(), &dataset);
    for _ in 0..64 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let bad_manifest = (seed & 1) == 1;
        let bad_sqlite = (seed & 2) == 2;
        let manifest_sha = if bad_manifest {
            "00".repeat(32)
        } else {
            expected_manifest.clone()
        };
        let sqlite_sha = if bad_sqlite {
            "ff".repeat(32)
        } else {
            expected_sqlite.clone()
        };

        let _ = store.put_dataset(
            &dataset,
            &manifest_bytes,
            &sqlite_bytes,
            &manifest_sha,
            &sqlite_sha,
        );

        if manifest_sha != expected_manifest || sqlite_sha != expected_sqlite {
            assert!(
                !paths.manifest.exists() && !paths.sqlite.exists(),
                "failed publish must not leave partial final artifacts"
            );
            assert!(
                !paths.derived_dir.join("manifest.json.tmp").exists()
                    && !paths.derived_dir.join("gene_summary.sqlite.tmp").exists()
                    && !paths.derived_dir.join("manifest.lock.tmp").exists(),
                "failed publish must clean tmp files"
            );
        } else {
            break;
        }
    }
}

#[test]
#[cfg(feature = "backend-s3")]
fn s3_cached_only_mode_is_conformance_compatible() {
    let store = S3LikeStore::new("http://127.0.0.1:9".to_string(), "atlas".to_string())
        .with_cache(tempdir().expect("cache").path().to_path_buf(), true);
    let err = store
        .get_manifest(&mk_dataset())
        .expect_err("cached-only mode should fail");
    assert_eq!(err.code, StoreErrorCode::CachedOnly);
}

#[test]
#[cfg(feature = "backend-s3")]
fn http_store_blocks_private_ssrf_targets() {
    let store = HttpReadonlyStore::new("http://127.0.0.1:8080".to_string());
    let err = store
        .list_datasets()
        .expect_err("private host must be blocked");
    assert_eq!(err.code, StoreErrorCode::Validation);
}
