// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
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

async fn send_raw(addr: std::net::SocketAddr, path: &str) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect server");
    let req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(req.as_bytes())
        .await
        .expect("write request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .expect("read response");
    let (head, body) = response
        .split_once("\r\n\r\n")
        .expect("http response must have separator");
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .expect("http status");
    (status, head.to_string(), body.to_string())
}

#[tokio::test]
async fn redis_unavailable_falls_back_without_service_failure() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let api = ApiConfig {
        redis_url: Some("redis://127.0.0.1:6390".to_string()),
        enable_redis_response_cache: true,
        redis_timeout_ms: 10,
        redis_retry_attempts: 1,
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(mgr, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let query = "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1";
    let (status, _, _) = send_raw(addr, query).await;
    assert_eq!(status, 200);

    let (status, _, metrics) = send_raw(addr, "/metrics").await;
    assert_eq!(status, 200);
    assert!(metrics.contains("bijux_redis_read_fallback_total"));
}

#[tokio::test]
#[ignore = "requires REDIS_URL and local Redis; non-CI integration test"]
async fn redis_exact_lookup_cache_hit_sets_cache_header() {
    let redis_url = match std::env::var("REDIS_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("skipping redis_exact_lookup_cache_hit_sets_cache_header: REDIS_URL not set");
            return;
        }
    };
    let can_connect = match redis::Client::open(redis_url.clone()) {
        Ok(client) => client.get_connection().is_ok(),
        Err(_) => false,
    };
    if !can_connect {
        eprintln!("skipping redis_exact_lookup_cache_hit_sets_cache_header: redis not reachable");
        return;
    }
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let api = ApiConfig {
        redis_url: Some(redis_url),
        redis_prefix: format!("atlas-test-{}", std::process::id()),
        enable_redis_response_cache: true,
        redis_timeout_ms: 50,
        redis_retry_attempts: 2,
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(mgr, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let query = "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1";
    let (status1, _, _) = send_raw(addr, query).await;
    assert_eq!(status1, 200);

    let (status2, headers2, _) = send_raw(addr, query).await;
    assert_eq!(status2, 200);
    if headers2.contains("x-atlas-cache: redis-hit") {
        return;
    }
    let (status3, _, metrics) = send_raw(addr, "/metrics").await;
    assert_eq!(status3, 200);
    let redis_hit_line = metrics
        .lines()
        .find(|line| line.starts_with("bijux_redis_cache_hits_total"))
        .unwrap_or_default();
    let hit_count = redis_hit_line
        .split_whitespace()
        .last()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    assert!(
        hit_count >= 1,
        "expected redis hit header or redis hit metric >= 1, got line: {redis_hit_line}"
    );
}
