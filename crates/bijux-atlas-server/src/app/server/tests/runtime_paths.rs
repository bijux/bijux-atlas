// SPDX-License-Identifier: Apache-2.0

use super::support::*;

#[tokio::test]
async fn shard_aware_selection_uses_seqid_mapping() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);

    let tmp = tempdir().expect("tempdir");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        store,
    );
    let _ = mgr
        .open_dataset_connection(&ds)
        .await
        .expect("prime cache entry");
    let paths = mgr.resolve_cache_paths(&ds).await.expect("resolve paths");
    std::fs::write(
        paths.derived_dir.join("catalog_shards.json"),
        r#"{
  "dataset": {"release":"110","species":"homo_sapiens","assembly":"GRCh38"},
  "mode": "contig",
  "shards": [
    {"shard_id":"s1","sqlite_path":"shard_chr1.sqlite","sqlite_sha256":"a","seqids":["chr1"]},
    {"shard_id":"s2","sqlite_path":"shard_chr2.sqlite","sqlite_sha256":"b","seqids":["chr2"]}
  ]
}"#,
    )
    .expect("write shard catalog");
    std::fs::write(paths.derived_dir.join("shard_chr1.sqlite"), b"stub").expect("shard1");
    std::fs::write(paths.derived_dir.join("shard_chr2.sqlite"), b"stub").expect("shard2");
    mgr.reverify_cached_datasets().await.expect("reverify");

    let chr1 = mgr
        .selected_shards_for_region(&ds, Some("chr1"))
        .await
        .expect("select seqid shard");
    assert_eq!(chr1.len(), 1);
    assert!(chr1[0].to_string_lossy().ends_with("shard_chr1.sqlite"));

    let all = mgr
        .selected_shards_for_region(&ds, None)
        .await
        .expect("select all shards");
    assert_eq!(all.len(), 2);
}

#[cfg(unix)]
#[tokio::test]
async fn cache_root_permissions_are_hardened() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir().expect("tempdir");
    let disk_root = tmp.path().join("cache-root");
    std::fs::create_dir_all(&disk_root).expect("mkdir");
    std::fs::set_permissions(&disk_root, std::fs::Permissions::from_mode(0o777))
        .expect("set world writable");

    let _mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: disk_root.clone(),
            ..Default::default()
        },
        Arc::new(FakeStore::default()),
    );

    let mode = std::fs::metadata(&disk_root)
        .expect("metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o002, 0, "cache root must not remain world-writable");
}

#[tokio::test]
async fn relative_cache_root_is_anchored_to_the_workspace_artifacts_root() {
    let relative_cache_root = PathBuf::from("artifacts/test-relative-cache-root");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .map(std::path::Path::to_path_buf)
        .unwrap_or(manifest_dir);
    let expected_workspace_cache_root = repo_root.join(relative_cache_root.clone());
    let _cleanup = CreatedDirGuard::new(&expected_workspace_cache_root);
    let crate_local_cache_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("test-relative-cache-root");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: relative_cache_root,
            ..Default::default()
        },
        Arc::new(FakeStore::default()),
    );

    assert_eq!(mgr.disk_root(), expected_workspace_cache_root.as_path());
    assert!(
        !crate_local_cache_root.exists(),
        "relative cache roots must never create crate-local artifacts: {}",
        crate_local_cache_root.display()
    );
}
