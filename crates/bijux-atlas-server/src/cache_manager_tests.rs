use super::*;
use tempfile::tempdir;

fn fixture_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("x.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
         CREATE TABLE dataset_stats(dimension TEXT NOT NULL, value TEXT NOT NULL, gene_count INTEGER NOT NULL, PRIMARY KEY (dimension, value));
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10,1,10);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('biotype','pc',1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('seqid','chr1',1);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}

fn mk_dataset() -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        bijux_atlas_model::ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            sqlite_sha,
        ),
        bijux_atlas_model::ManifestStats::new(1, 1, 1),
    );
    (ds, manifest, sqlite)
}

fn mk_dataset_for(release: &str) -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new(release, "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        bijux_atlas_model::ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            sqlite_sha,
        ),
        bijux_atlas_model::ManifestStats::new(1, 1, 1),
    );
    (ds, manifest, sqlite)
}

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
#[ignore]
async fn chaos_mode_slow_store_10x_latency_graceful_errors() {
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
#[ignore]
async fn chaos_mode_delete_dataset_mid_request_redownloads_cleanly() {
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
#[ignore]
async fn chaos_mode_random_byte_corruption_never_serves_results() {
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
