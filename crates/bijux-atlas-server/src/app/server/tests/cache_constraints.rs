// SPDX-License-Identifier: Apache-2.0

use super::support::*;

#[tokio::test]
async fn cache_eviction_stress_respects_caps() {
    let store = Arc::new(FakeStore::default());
    for idx in 100..112 {
        let (ds, manifest, sqlite) = mk_dataset_for(&idx.to_string());
        store.manifest.lock().await.insert(ds.clone(), manifest);
        store.sqlite.lock().await.insert(ds, sqlite);
    }
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        max_dataset_count: 4,
        max_disk_bytes: 1_000_000,
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    for idx in 100..112 {
        let ds = DatasetId::new(&idx.to_string(), "homo_sapiens", "GRCh38").expect("id");
        let _ = mgr.open_dataset_connection(&ds).await.expect("open");
    }
    mgr.evict_background().await.expect("evict");
    let entries = mgr.entries.lock().await;
    assert!(
        entries.len() <= 4,
        "cache count must respect max_dataset_count"
    );
    let total_bytes: u64 = entries.values().map(|entry| entry.size_bytes).sum();
    assert!(
        total_bytes <= 1_000_000,
        "cache bytes must respect max_disk_bytes, got {total_bytes}"
    );
}

#[tokio::test]
async fn sqlite_connection_caps_are_enforced() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        max_total_connections: 1,
        max_connections_per_dataset: 1,
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let c1 = mgr.open_dataset_connection(&ds).await.expect("first conn");

    let m2 = Arc::clone(&mgr);
    let d2 = ds.clone();
    let wait = tokio::time::timeout(Duration::from_millis(120), async move {
        m2.open_dataset_connection(&d2).await
    })
    .await;
    assert!(wait.is_err(), "second connection should block under cap");
    drop(c1);
}

#[tokio::test]
async fn pinned_dataset_is_not_evicted() {
    let (pinned, manifest, sqlite) = mk_dataset();
    let (other, manifest2, sqlite2) = mk_dataset_for("111");
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(pinned.clone(), manifest);
    store.sqlite.lock().await.insert(pinned.clone(), sqlite);
    store.manifest.lock().await.insert(other.clone(), manifest2);
    store.sqlite.lock().await.insert(other.clone(), sqlite2);

    let tmp = tempdir().expect("tempdir");
    let mut pinned_set = HashSet::new();
    pinned_set.insert(pinned.clone());
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        max_dataset_count: 1,
        pinned_datasets: pinned_set,
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let _ = mgr
        .open_dataset_connection(&pinned)
        .await
        .expect("open pinned");
    let _ = mgr
        .open_dataset_connection(&other)
        .await
        .expect("open other");
    mgr.evict_background().await.expect("evict");
    let entries = mgr.entries.lock().await;
    assert!(entries.contains_key(&pinned), "pinned dataset must remain");
}

#[tokio::test]
async fn failed_download_leaves_no_partial_artifact() {
    let (ds, manifest, _sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);

    let err = match mgr.open_dataset_connection(&ds).await {
        Ok(_) => panic!("download should fail without sqlite bytes"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("sqlite missing"),
        "unexpected error: {err}"
    );

    let paths = local_cache_paths(tmp.path(), "1");
    assert!(
        !paths.sqlite.exists(),
        "partial sqlite file must not remain"
    );
    assert!(
        !paths.manifest.exists(),
        "partial manifest file must not remain"
    );
}

#[tokio::test]
async fn alias_like_release_switch_uses_hash_key_without_corruption() {
    let (ds_a, manifest_a, sqlite_a) = mk_dataset_for("110");
    let (ds_b, manifest_b, sqlite_b) = mk_dataset_for("111");
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds_a.clone(), manifest_a);
    store.sqlite.lock().await.insert(ds_a.clone(), sqlite_a);
    store.manifest.lock().await.insert(ds_b.clone(), manifest_b);
    store.sqlite.lock().await.insert(ds_b.clone(), sqlite_b);

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);

    let _ = mgr
        .open_dataset_connection(&ds_a)
        .await
        .expect("open first release");
    let _ = mgr
        .open_dataset_connection(&ds_b)
        .await
        .expect("open second release");
    let _ = mgr
        .open_dataset_connection(&ds_a)
        .await
        .expect("re-open first release");

    let key_a = std::fs::read_to_string(dataset_index_path(tmp.path(), &ds_a)).expect("index a");
    let key_b = std::fs::read_to_string(dataset_index_path(tmp.path(), &ds_b)).expect("index b");
    assert_eq!(
        key_a.trim(),
        key_b.trim(),
        "hash-keyed aliases should co-locate"
    );
}

#[tokio::test]
async fn failed_downloads_respect_retry_budget_and_prevent_refetch_loop() {
    let (ds, manifest, _sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    let tmp = tempdir().expect("tempdir");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            store_retry_budget: 1,
            ..Default::default()
        },
        store.clone(),
    );

    let err1 = match mgr.open_dataset_connection(&ds).await {
        Ok(_) => panic!("first download should fail"),
        Err(err) => err,
    };
    assert!(
        err1.to_string().contains("sqlite missing"),
        "unexpected first error: {err1}"
    );
    let calls_after_first = store.fetch_calls.load(std::sync::atomic::Ordering::Relaxed);
    assert!(calls_after_first >= 1);

    let err2 = match mgr.open_dataset_connection(&ds).await {
        Ok(_) => panic!("second download should be blocked by retry budget"),
        Err(err) => err,
    };
    assert!(
        err2.to_string().contains("retry budget exhausted"),
        "unexpected second error: {err2}"
    );
    let calls_after_second = store.fetch_calls.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        calls_after_second, calls_after_first,
        "retry budget exhaustion should prevent further refetch attempts"
    );
}
