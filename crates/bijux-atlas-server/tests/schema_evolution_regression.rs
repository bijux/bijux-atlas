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

fn old_schema_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("old.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "PRAGMA user_version=1;
         CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, exon_count INT DEFAULT 0, total_exon_span INT DEFAULT 0, cds_present INT DEFAULT 0, sequence_length INT);
         CREATE TABLE transcript_summary(id INTEGER PRIMARY KEY, transcript_id TEXT, parent_gene_id TEXT, transcript_type TEXT, biotype TEXT, seqid TEXT, start INT, end INT, exon_count INT, total_exon_span INT, cds_present INT);
         CREATE TABLE dataset_stats(dimension TEXT NOT NULL, value TEXT NOT NULL, gene_count INTEGER NOT NULL, PRIMARY KEY (dimension, value));
         CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
         CREATE INDEX idx_gene_summary_name ON gene_summary(name);
         CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
         CREATE INDEX idx_gene_summary_region ON gene_summary(seqid,start,end);
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10,1,10);
         INSERT INTO transcript_summary(id,transcript_id,parent_gene_id,transcript_type,biotype,seqid,start,end,exon_count,total_exon_span,cds_present) VALUES (1,'t1','g1','transcript','pc','chr1',1,10,1,10,1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('biotype','pc',1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('seqid','chr1',1);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite")
}

async fn send_raw(addr: std::net::SocketAddr, path: &str) -> (u16, String) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.expect("write");
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
    (status, body.to_string())
}

#[tokio::test]
async fn old_schema_artifact_remains_queryable() {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let sqlite = old_schema_sqlite();
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

    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds, sqlite);

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let app = build_router(AppState::new(mgr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });

    let (status, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1",
    )
    .await;
    assert_eq!(status, 200);
    assert!(body.contains("g1"));
}
