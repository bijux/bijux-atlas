use std::collections::HashMap;
use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use rusqlite::Connection;
use serde::Deserialize;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Deserialize)]
struct MetricsContract {
    required_metrics: HashMap<String, Vec<String>>,
    required_spans: Vec<String>,
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
async fn metrics_endpoint_matches_metrics_contract() {
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

    for path in [
        "/healthz",
        "/healthz/overload",
        "/readyz",
        "/v1/version",
        "/v1/datasets",
        "/v1/releases/110/species/homo_sapiens/assemblies/GRCh38",
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38&biotype=pc",
        "/v1/diff/genes?release=110&species=homo_sapiens&assembly=GRCh38&from_release=109&to_release=110&limit=1",
        "/v1/diff/region?release=110&species=homo_sapiens&assembly=GRCh38&from_release=109&to_release=110&region=chr1:1-10&limit=1",
        "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-10",
        "/v1/genes/g1/sequence?release=110&species=homo_sapiens&assembly=GRCh38",
        "/v1/genes/g1/transcripts?release=110&species=homo_sapiens&assembly=GRCh38&limit=1",
        "/v1/transcripts/tx1?release=110&species=homo_sapiens&assembly=GRCh38",
        "/debug/datasets",
        "/debug/dataset-health?release=110&species=homo_sapiens&assembly=GRCh38",
        "/debug/registry-health",
    ] {
        let (status, _, _) = send_raw(addr, path).await;
        assert!(
            matches!(status, 200 | 304 | 400 | 401 | 404 | 422 | 503),
            "unexpected status {status} for {path}"
        );
    }

    let (status, _, body) = send_raw(addr, "/metrics").await;
    assert_eq!(status, 200);

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let contract_path = root.join("observability").join("metrics_contract.json");
    let contract: MetricsContract =
        serde_json::from_slice(&std::fs::read(contract_path).expect("read contract"))
            .expect("parse contract");

    for (metric, labels) in contract.required_metrics {
        let line = body
            .lines()
            .find(|l| l.starts_with(&format!("{}{{", metric)))
            .unwrap_or_else(|| panic!("metric missing: {metric}"));
        for label in labels {
            assert!(
                line.contains(&format!("{label}=\"")),
                "metric {metric} missing label {label}"
            );
        }
    }

    let endpoint_contract: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("docs/contracts/ENDPOINTS.json")).expect("read endpoints"),
    )
    .expect("parse endpoints");
    let server_sources = [
        root.join("crates/bijux-atlas-server/src/runtime/server_runtime_app.rs"),
        root.join("crates/bijux-atlas-server/src/http/genes.rs"),
        root.join("crates/bijux-atlas-server/src/http/handlers.rs"),
        root.join("crates/bijux-atlas-server/src/http/diff.rs"),
        root.join("crates/bijux-atlas-server/src/http/sequence.rs"),
        root.join("crates/bijux-atlas-server/src/http/transcript_endpoints.rs"),
        root.join("crates/bijux-atlas-server/src/http/handlers_endpoints.rs"),
        root.join("crates/bijux-atlas-server/src/http/handlers_utilities.rs"),
    ];
    let source_concat = server_sources
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("read server source"))
        .collect::<Vec<_>>()
        .join("\n");
    for path in endpoint_contract["endpoints"]
        .as_array()
        .expect("endpoints array")
        .iter()
        .map(|e| e["path"].as_str().expect("path"))
    {
        assert!(
            source_concat.contains(path)
                || source_concat.contains(&path.replace("{", ":").replace('}', "")),
            "endpoint route not referenced in server sources: {path}"
        );
    }
    assert!(
        source_concat.contains("observe_request("),
        "server sources must emit request metrics"
    );

    let trace_generated = std::fs::read_to_string(
        root.join("crates/bijux-atlas-server/src/telemetry/generated/trace_spans_contract.rs"),
    )
    .expect("read generated spans");
    for span in contract.required_spans {
        assert!(
            trace_generated.contains(&format!("\"{span}\"")),
            "required span missing from generated span constants: {span}"
        );
    }
}

fn fixture_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("x.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, exon_count INT DEFAULT 0, total_exon_span INT DEFAULT 0, cds_present INT DEFAULT 0, sequence_length INT);
         CREATE TABLE transcript_summary(id INTEGER PRIMARY KEY, transcript_id TEXT, parent_gene_id TEXT, transcript_type TEXT, biotype TEXT, seqid TEXT, start INT, end INT, exon_count INT, total_exon_span INT, cds_present INT);
         CREATE TABLE dataset_stats(dimension TEXT NOT NULL, value TEXT NOT NULL, gene_count INTEGER NOT NULL, PRIMARY KEY (dimension, value));
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10,1,10);
         CREATE INDEX idx_transcript_summary_transcript_id ON transcript_summary(transcript_id);
         CREATE INDEX idx_transcript_summary_parent_gene_id ON transcript_summary(parent_gene_id);
         CREATE INDEX idx_transcript_summary_biotype ON transcript_summary(biotype);
         CREATE INDEX idx_transcript_summary_type ON transcript_summary(transcript_type);
         CREATE INDEX idx_transcript_summary_region ON transcript_summary(seqid,start,end);
         INSERT INTO transcript_summary(id,transcript_id,parent_gene_id,transcript_type,biotype,seqid,start,end,exon_count,total_exon_span,cds_present) VALUES (1,'tx1','g1','transcript','pc','chr1',1,10,1,10,1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('biotype','pc',1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('seqid','chr1',1);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}
