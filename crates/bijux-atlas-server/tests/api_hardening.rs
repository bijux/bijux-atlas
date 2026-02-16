use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use rusqlite::Connection;
use serde_json::Value;
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

async fn send_raw(
    addr: std::net::SocketAddr,
    path: &str,
    headers: &[(&str, &str)],
) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect server");
    let mut req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
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
async fn error_contract_and_etag_behaviors() {
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
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(addr, "/v1/version", &[]).await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("version json");
    assert_eq!(
        json.get("plugin")
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str),
        Some("bijux-atlas")
    );

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&fields=nope",
        &[],
    )
    .await;
    assert_eq!(status, 400);
    let json: Value = serde_json::from_str(&body).expect("error json");
    assert_eq!(
        json.get("error")
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str),
        Some("InvalidQueryParameter")
    );

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&cursor=bad.cursor",
        &[],
    )
    .await;
    assert_eq!(status, 400);
    let json: Value = serde_json::from_str(&body).expect("cursor error json");
    assert_eq!(
        json.get("error")
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str),
        Some("InvalidCursor")
    );

    let (status, headers, _) = send_raw(addr, "/v1/datasets", &[]).await;
    assert_eq!(status, 200);
    assert!(headers.contains("x-request-id: "));
    let etag = headers
        .lines()
        .find_map(|line| line.strip_prefix("etag: "))
        .expect("etag header present")
        .to_string();
    let (status, _, _) = send_raw(addr, "/v1/datasets", &[("If-None-Match", &etag)]).await;
    assert_eq!(status, 304);
}

#[tokio::test]
async fn readiness_metrics_and_debug_gate() {
    let store = Arc::new(FakeStore::default());
    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let api = ApiConfig {
        readiness_requires_catalog: false,
        ..ApiConfig::default()
    };
    let state = AppState::with_config(mgr, api, Default::default());
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(addr, "/readyz", &[]).await;
    assert_eq!(status, 200);
    assert!(body.contains("ready"));

    let (status, _, body) = send_raw(addr, "/metrics", &[]).await;
    assert_eq!(status, 200);
    assert!(body.contains("bijux_dataset_hits"));
    assert!(body.contains("bijux_http_requests_total"));

    let (status, _, _) = send_raw(addr, "/debug/datasets", &[]).await;
    assert_eq!(status, 404);
}
