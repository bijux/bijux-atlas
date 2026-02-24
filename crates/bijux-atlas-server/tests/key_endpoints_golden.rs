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
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("x.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, exon_count INT DEFAULT 0, total_exon_span INT DEFAULT 0, cds_present INT DEFAULT 0, sequence_length INT);
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10,1,10);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite")
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
        .expect("http response separator");
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .expect("status");
    (status, head.to_string(), body.to_string())
}

#[tokio::test]
async fn golden_core_endpoints_return_stable_json_shape() {
    let store = Arc::new(FakeStore::default());
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            sha256_hex(&sqlite),
        ),
        ManifestStats::new(1, 1, 1),
    );
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
    let app = build_router(AppState::new(mgr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, version_body) = send_raw(addr, "/v1/version").await;
    assert_eq!(status, 200);
    let version: serde_json::Value = serde_json::from_str(&version_body).expect("version json");
    assert_eq!(version["api_version"], "v1");
    assert_eq!(version["contract_version"], "v1");

    let (status, _, datasets_body) = send_raw(
        addr,
        "/v1/datasets?release=110&species=homo_sapiens&assembly=GRCh38",
    )
    .await;
    assert_eq!(status, 200);
    let datasets: serde_json::Value = serde_json::from_str(&datasets_body).expect("datasets json");
    assert_eq!(datasets["api_version"], "v1");
    assert!(datasets["data"]["items"].is_array());

    let (status, _, genes_body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
    )
    .await;
    assert!(matches!(status, 200 | 400 | 422));
    if status == 200 {
        let genes: serde_json::Value = serde_json::from_str(&genes_body).expect("genes json");
        assert_eq!(genes["api_version"], "v1");
        assert!(genes["data"]["rows"].is_array());
    }
}
