// SPDX-License-Identifier: Apache-2.0

use super::support::*;

#[tokio::test]
async fn single_flight_download_shared_by_high_concurrency_calls() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store.clone());

    let mut joins = Vec::new();
    for _ in 0..64 {
        let m = Arc::clone(&mgr);
        let d = ds.clone();
        joins.push(tokio::spawn(
            async move { m.open_dataset_connection(&d).await },
        ));
    }
    for j in joins {
        j.await.expect("join handle").expect("open connection");
    }

    let calls = store.fetch_calls.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(calls, 1, "single-flight should perform one manifest fetch");
}

#[tokio::test]
async fn cached_only_mode_serves_existing_and_rejects_missing() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let mgr_download = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        store.clone(),
    );
    let _ = mgr_download
        .open_dataset_connection(&ds)
        .await
        .expect("download into cache");

    let mgr_cached_only = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            cached_only_mode: true,
            ..Default::default()
        },
        store,
    );
    let _ = mgr_cached_only
        .open_dataset_connection(&ds)
        .await
        .expect("serve cached dataset in cached-only mode");

    let missing = DatasetId::new("999", "homo_sapiens", "GRCh38").expect("dataset id");
    let err = match mgr_cached_only.open_dataset_connection(&missing).await {
        Ok(_) => panic!("expected cached-only mode miss"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("cached-only mode"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn startup_warmup_honors_fail_readiness_flag() {
    let store = Arc::new(FakeStore::default());
    let tmp = tempdir().expect("tempdir");
    let missing = DatasetId::new("999", "homo_sapiens", "GRCh38").expect("dataset id");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            startup_warmup: vec![missing],
            fail_readiness_on_missing_warmup: true,
            ..Default::default()
        },
        store,
    );

    let err = mgr.startup_warmup().await.expect_err("warmup must fail");
    assert!(err.to_string().contains("warmup failed"));
}

#[tokio::test]
async fn read_only_sqlite_pragma_profile_is_applied() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();
    let tmp = tempdir().expect("tempdir");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        store,
    );
    let conn = mgr
        .open_dataset_connection(&ds)
        .await
        .expect("open dataset connection");
    let query_only: i64 = conn
        .conn
        .query_row("PRAGMA query_only", [], |r| r.get(0))
        .expect("query_only");
    let sync: i64 = conn
        .conn
        .query_row("PRAGMA synchronous", [], |r| r.get(0))
        .expect("synchronous");
    let temp_store: i64 = conn
        .conn
        .query_row("PRAGMA temp_store", [], |r| r.get(0))
        .expect("temp_store");
    assert_eq!(query_only, 1);
    assert_eq!(sync, 0);
    assert_eq!(temp_store, 2);

    let write_attempt = conn
        .conn
        .execute("CREATE TABLE __should_not_write (id INTEGER)", []);
    assert!(
        write_attempt.is_err(),
        "server must never write to artifact sqlite databases"
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn slow__store_timeout_returns_a_real_outcome() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore {
        slow_read: true,
        slow_read_delay: Duration::from_millis(500),
        ..Default::default()
    });
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        dataset_open_timeout: Duration::from_millis(1),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    match mgr.open_dataset_connection(&ds).await {
        Ok(conn) => {
            let count: i64 = conn
                .conn
                .query_row("SELECT COUNT(*) FROM gene_summary", [], |row| row.get(0))
                .expect("query after slow fetch");
            assert_eq!(count, 1);
        }
        Err(err) => {
            assert!(
                err.to_string().contains("timeout"),
                "expected timeout on failure path, got: {err}"
            );
        }
    }
}

#[tokio::test]
async fn missing_cached_sqlite_is_redownloaded_on_next_open() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store.clone());
    let conn = mgr
        .open_dataset_connection(&ds)
        .await
        .expect("open first connection");
    let count: i64 = conn
        .conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |row| row.get(0))
        .expect("query existing connection");
    assert_eq!(count, 1);
    drop(conn);

    let paths = mgr
        .resolve_cache_paths(&ds)
        .await
        .expect("resolve cached paths");
    std::fs::remove_file(&paths.sqlite).expect("delete sqlite mid-flight simulation");
    let second = mgr
        .open_dataset_connection(&ds)
        .await
        .expect("re-download and reopen after deletion");
    let second_count: i64 = second
        .conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |row| row.get(0))
        .expect("query after re-download");
    assert_eq!(second_count, 1);

    let calls = store.fetch_calls.load(std::sync::atomic::Ordering::Relaxed);
    assert!(calls >= 2, "expected re-fetch after deletion, got {calls}");
}

#[tokio::test]
async fn failover_across_replicas_one_fails_other_serves() {
    let (ds, manifest, sqlite) = mk_dataset();

    let failing_store = Arc::new(FakeStore::default());
    let tmp_a = tempdir().expect("tempdir");
    let mgr_a = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp_a.path().to_path_buf(),
            ..Default::default()
        },
        failing_store,
    );
    let err = match mgr_a.open_dataset_connection(&ds).await {
        Ok(_) => panic!("replica A should fail"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("manifest missing"));

    let healthy_store = Arc::new(FakeStore::default());
    healthy_store
        .manifest
        .lock()
        .await
        .insert(ds.clone(), manifest);
    healthy_store.sqlite.lock().await.insert(ds.clone(), sqlite);
    let tmp_b = tempdir().expect("tempdir");
    let mgr_b = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp_b.path().to_path_buf(),
            ..Default::default()
        },
        healthy_store,
    );
    let conn = mgr_b
        .open_dataset_connection(&ds)
        .await
        .expect("replica B should serve dataset");
    let count: i64 = conn
        .conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |row| row.get(0))
        .expect("query healthy replica");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn corruption_is_detected_by_reverification() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    mgr.open_dataset_connection(&ds)
        .await
        .expect("open cached dataset");

    let paths = mgr
        .resolve_cache_paths(&ds)
        .await
        .expect("resolve cached paths");
    std::fs::write(&paths.sqlite, b"corrupted").expect("corrupt sqlite");
    mgr.reverify_cached_datasets().await.expect("run reverify");
    let entries = mgr.entries.lock().await;
    assert!(
        !entries.contains_key(&ds),
        "corrupted dataset should be evicted from cache entries"
    );
}

#[tokio::test]
async fn byte_corrupted_cached_dataset_is_not_served_in_cached_only_mode() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        store.clone(),
    );
    mgr.open_dataset_connection(&ds)
        .await
        .expect("download and cache dataset");
    let paths = mgr
        .resolve_cache_paths(&ds)
        .await
        .expect("resolve cached paths");
    let mut bytes = std::fs::read(&paths.sqlite).expect("read sqlite");
    for i in (0..bytes.len()).step_by(257).take(32) {
        bytes[i] ^= 0xAA;
    }
    std::fs::write(&paths.sqlite, bytes).expect("write corrupted sqlite");
    mgr.reverify_cached_datasets().await.expect("reverify");

    let cached_only_mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            cached_only_mode: true,
            ..Default::default()
        },
        Arc::new(FakeStore::default()),
    );
    let err = match cached_only_mgr.open_dataset_connection(&ds).await {
        Ok(_) => panic!("corrupted dataset must not be served"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("cached-only mode")
            || err.to_string().contains("missing from cache"),
        "unexpected error: {err}"
    );
}
