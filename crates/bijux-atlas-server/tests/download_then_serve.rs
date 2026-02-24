// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

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
async fn integration_server_download_then_serve() {
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
    let app = build_router(AppState::new(mgr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });

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
    assert!(response.contains("\"gene_count\":1"));
}

#[tokio::test]
async fn integration_cached_only_mode_serves_when_store_unavailable() {
    let (ds, manifest, sqlite) = mk_dataset();
    let warm_store = Arc::new(FakeStore::default());
    warm_store
        .manifest
        .lock()
        .await
        .insert(ds.clone(), manifest);
    warm_store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *warm_store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let warm_mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        warm_store,
    );
    let _ = warm_mgr
        .open_dataset_connection(&ds)
        .await
        .expect("warm cache with dataset");

    let outage_store = Arc::new(FakeStore::default());
    let cached_only_mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            cached_only_mode: true,
            ..Default::default()
        },
        outage_store,
    );
    let app = build_router(AppState::new(cached_only_mgr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });

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
    assert!(response.contains("\"gene_count\":1"));
}

#[tokio::test]
async fn integration_partial_store_payload_fails_checksum_and_never_serves() {
    let (ds, manifest, sqlite_full) = mk_dataset();
    let sqlite_partial = sqlite_full[..(sqlite_full.len() / 2)].to_vec();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite_partial);
    *store.etag.lock().await = "v1".to_string();

    let tmp = tempdir().expect("tempdir");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        store,
    );
    let app = build_router(AppState::new(mgr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });

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
    assert!(
        response.starts_with("HTTP/1.1 503 Service Unavailable\r\n")
            || response.starts_with("HTTP/1.1 500 Internal Server Error\r\n")
    );
}
