use std::sync::Arc;
use std::time::{Duration, Instant};

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use rusqlite::Connection;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        ArtifactChecksums::new("a".repeat(64), "b".repeat(64), "c".repeat(64), sqlite_sha),
        ManifestStats::new(1, 1, 1),
    );
    (ds, manifest, sqlite)
}

#[tokio::test]
async fn latency_regression_guard_p95_under_threshold() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds, manifest);
    store.sqlite.lock().await.insert(
        DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
        sqlite,
    );
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let app = build_router(AppState::new(mgr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let mut samples = Vec::new();
    for _ in 0..120 {
        let started = Instant::now();
        let mut stream = tokio::net::TcpStream::connect(addr)
            .await
            .expect("connect server");
        let request = format!(
            "GET /v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38 HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            addr
        );
        stream
            .write_all(request.as_bytes())
            .await
            .expect("write request");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .expect("read response");
        assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
        samples.push(started.elapsed());
    }

    samples.sort_unstable();
    let p95_idx = ((samples.len() as f64) * 0.95).ceil() as usize - 1;
    let p95 = samples[p95_idx.min(samples.len() - 1)];
    assert!(
        p95 <= Duration::from_millis(120),
        "p95 latency regression: {:?}",
        p95
    );
}

#[tokio::test]
async fn db_open_is_cheap_regression_guard() {
    let (_, _, sqlite) = mk_dataset();
    let tmp = tempdir().expect("tempdir");
    let db_path = tmp.path().join("bench.sqlite");
    std::fs::write(&db_path, sqlite).expect("write sqlite");

    let mut samples = Vec::new();
    for _ in 0..200 {
        let started = Instant::now();
        let conn = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open");
        let _: i64 = conn
            .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0))
            .expect("count");
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    let p95_idx = ((samples.len() as f64) * 0.95).ceil() as usize - 1;
    let p95 = samples[p95_idx.min(samples.len() - 1)];
    assert!(
        p95 <= Duration::from_millis(10),
        "db-open p95 regression: {:?}",
        p95
    );
}

#[tokio::test]
#[ignore]
async fn mmap_read_only_experiment_baseline() {
    let (_, _, sqlite) = mk_dataset();
    let tmp = tempdir().expect("tempdir");
    let db_path = tmp.path().join("mmap.sqlite");
    std::fs::write(&db_path, sqlite).expect("write sqlite");

    let run = |mmap_size: i64| -> Duration {
        let mut samples = Vec::new();
        for _ in 0..150 {
            let started = Instant::now();
            let conn = rusqlite::Connection::open_with_flags(
                &db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                    | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .expect("open");
            let _ = conn.execute_batch(&format!(
                "PRAGMA query_only=ON; PRAGMA mmap_size={mmap_size}; PRAGMA temp_store=MEMORY;"
            ));
            let _: i64 = conn
                .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0))
                .expect("count");
            samples.push(started.elapsed());
        }
        samples.sort_unstable();
        samples[((samples.len() as f64) * 0.95).ceil() as usize - 1]
    };

    let p95_no_mmap = run(0);
    let p95_mmap = run(256 * 1024 * 1024);
    assert!(p95_no_mmap > Duration::from_nanos(0));
    assert!(p95_mmap > Duration::from_nanos(0));
}

#[tokio::test]
async fn shard_open_fd_cap_enforces_safety_under_many_shards() {
    let store = Arc::new(FakeStore::default());
    let cfg = DatasetCacheConfig {
        max_open_shards_per_pod: 2,
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);

    let p1 = mgr.acquire_shard_permit().await.expect("permit1");
    let p2 = mgr.acquire_shard_permit().await.expect("permit2");
    let third = tokio::time::timeout(Duration::from_millis(30), mgr.acquire_shard_permit()).await;
    assert!(
        third.is_err(),
        "third shard permit should be blocked by cap"
    );
    drop((p1, p2));
}
