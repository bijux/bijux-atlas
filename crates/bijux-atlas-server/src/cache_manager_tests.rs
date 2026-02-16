use super::*;
use tempfile::tempdir;

fn fixture_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("x.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
         INSERT INTO gene_summary(id,gene_id,name,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','pc','chr1',1,10,1,10);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}

fn mk_dataset() -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let manifest = ArtifactManifest {
        manifest_version: "1".to_string(),
        db_schema_version: "1".to_string(),
        dataset: ds.clone(),
        checksums: bijux_atlas_model::ArtifactChecksums {
            gff3_sha256: "a".repeat(64),
            fasta_sha256: "b".repeat(64),
            fai_sha256: "c".repeat(64),
            sqlite_sha256: sqlite_sha,
        },
        stats: bijux_atlas_model::ManifestStats {
            gene_count: 1,
            transcript_count: 1,
            contig_count: 1,
        },
    };
    (ds, manifest, sqlite)
}

fn mk_dataset_for(release: &str) -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new(release, "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let manifest = ArtifactManifest {
        manifest_version: "1".to_string(),
        db_schema_version: "1".to_string(),
        dataset: ds.clone(),
        checksums: bijux_atlas_model::ArtifactChecksums {
            gff3_sha256: "a".repeat(64),
            fasta_sha256: "b".repeat(64),
            fai_sha256: "c".repeat(64),
            sqlite_sha256: sqlite_sha,
        },
        stats: bijux_atlas_model::ManifestStats {
            gene_count: 1,
            transcript_count: 1,
            contig_count: 1,
        },
    };
    (ds, manifest, sqlite)
}

#[tokio::test]
async fn single_flight_download_shared_by_concurrent_calls() {
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
    for _ in 0..8 {
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
#[ignore]
async fn chaos_mode_slow_store_reads_graceful_errors() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore {
        slow_read: true,
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
    let result = mgr.open_dataset_connection(&ds).await;
    match result {
        Ok(_) => {}
        Err(err) => {
            assert!(err.to_string().contains("timeout") || err.to_string().contains("missing"))
        }
    }
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

    let paths = artifact_paths(tmp.path(), &ds);
    std::fs::write(&paths.sqlite, b"corrupted").expect("corrupt sqlite");
    mgr.reverify_cached_datasets().await.expect("run reverify");
    let entries = mgr.entries.lock().await;
    assert!(
        !entries.contains_key(&ds),
        "corrupted dataset should be evicted from cache entries"
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
