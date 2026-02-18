use std::collections::BTreeMap;
use std::sync::Arc;

use bijux_atlas_core::{canonical::stable_json_bytes, sha256_hex};
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Serialize)]
struct SnapshotEntry {
    method: String,
    path: String,
    status: u16,
    location: Option<String>,
    body_shape: Value,
}

async fn send_raw(
    method: &str,
    addr: std::net::SocketAddr,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, String, Vec<u8>) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect server");
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    stream
        .write_all(req.as_bytes())
        .await
        .expect("write request head");
    if !body.is_empty() {
        stream.write_all(body).await.expect("write request body");
    }
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .expect("read response");
    let split = response
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("http response separator");
    let head = String::from_utf8(response[..split].to_vec()).expect("response head utf8");
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .expect("http status");
    (status, head, response[split + 4..].to_vec())
}

fn header_value(headers: &str, name: &str) -> Option<String> {
    let prefix = format!("{}:", name.to_ascii_lowercase());
    headers
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with(&prefix))
        .map(|line| line.split_once(':').map_or("", |(_, v)| v).trim().to_string())
}

fn normalize_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if k == "request_id" {
                    out.insert(k.clone(), Value::String("<request_id>".to_string()));
                } else {
                    out.insert(k.clone(), normalize_json(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(values) => Value::Array(values.iter().map(normalize_json).collect()),
        other => other.clone(),
    }
}

fn to_shape(value: &Value) -> Value {
    match value {
        Value::Null => Value::String("null".to_string()),
        Value::Bool(_) => Value::String("bool".to_string()),
        Value::Number(_) => Value::String("number".to_string()),
        Value::String(_) => Value::String("string".to_string()),
        Value::Array(values) => {
            let item = values
                .first()
                .map(to_shape)
                .unwrap_or(Value::String("empty".to_string()));
            serde_json::json!({ "type": "array", "item": item })
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), to_shape(v));
            }
            Value::Object(out)
        }
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
    .expect("schema");
    drop(conn);
    std::fs::read(db).expect("read sqlite")
}

#[tokio::test]
async fn api_surface_response_shapes_match_golden_snapshot() {
    let store = Arc::new(FakeStore::default());
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let sqlite = fixture_sqlite();
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        dataset.clone(),
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            sha256_hex(&sqlite),
        ),
        ManifestStats::new(1, 1, 1),
    );
    store.manifest.lock().await.insert(dataset.clone(), manifest);
    store.sqlite.lock().await.insert(dataset.clone(), sqlite);

    let tmp = tempdir().expect("tempdir");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        store,
    );
    let app = build_router(AppState::with_config(
        mgr,
        ApiConfig {
            enable_debug_datasets: true,
            ..ApiConfig::default()
        },
        Default::default(),
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let mut reqs = BTreeMap::new();
    reqs.insert("GET /debug/dataset-health", ("/debug/dataset-health?release=110&species=homo_sapiens&assembly=GRCh38", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /debug/datasets", ("/debug/datasets", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /debug/registry-health", ("/debug/registry-health", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /healthz", ("/healthz", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /healthz/overload", ("/healthz/overload", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /metrics", ("/metrics", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /readyz", ("/readyz", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/datasets", ("/v1/datasets", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/datasets/{release}/{species}/{assembly}", ("/v1/datasets/110/homo_sapiens/GRCh38?include_bom=1", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/diff/genes", ("/v1/diff/genes?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&limit=1", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/diff/region", ("/v1/diff/region?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-10&limit=1", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/_debug/echo", ("/v1/_debug/echo?x=1", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/genes", ("/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/genes/count", ("/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/genes/{gene_id}/sequence", ("/v1/genes/g1/sequence?release=110&species=homo_sapiens&assembly=GRCh38", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/genes/{gene_id}/transcripts", ("/v1/genes/g1/transcripts?release=110&species=homo_sapiens&assembly=GRCh38&limit=1", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/openapi.json", ("/v1/openapi.json", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("POST /v1/query/validate", ("/v1/query/validate", vec![("Content-Type", "application/json")], br#"{"endpoint":"/v1/genes","query":{"release":"110","species":"homo_sapiens","assembly":"GRCh38","limit":"1"}}"#.to_vec()));
    reqs.insert("GET /v1/releases/{release}/species/{species}/assemblies/{assembly}", ("/v1/releases/110/species/homo_sapiens/assemblies/GRCh38?include_bom=1", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/sequence/region", ("/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-10", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/transcripts/{tx_id}", ("/v1/transcripts/tx1?release=110&species=homo_sapiens&assembly=GRCh38", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));
    reqs.insert("GET /v1/version", ("/v1/version", Vec::<(&str, &str)>::new(), Vec::<u8>::new()));

    let mut snapshots = Vec::new();
    for (key, (path, headers, body)) in reqs {
        let (method, route) = key.split_once(' ').expect("method/path key");
        let (status, head, raw_body) = send_raw(method, addr, path, &headers, &body).await;
        let location = header_value(&head, "location");
        let body_shape = if route == "/metrics" {
            let text = String::from_utf8(raw_body).expect("metrics body utf8");
            let mut lines = text
                .lines()
                .filter(|line| line.starts_with("bijux_") || line.starts_with("atlas_"))
                .take(5)
                .map(str::to_string)
                .collect::<Vec<_>>();
            lines.sort();
            serde_json::json!({"type":"text","sample_metric_lines":lines})
        } else if raw_body.is_empty() {
            serde_json::json!({"type":"empty"})
        } else {
            let parsed = serde_json::from_slice::<Value>(&raw_body).unwrap_or_else(|_| {
                serde_json::json!({
                    "type": "text",
                    "prefix": String::from_utf8_lossy(&raw_body).chars().take(80).collect::<String>()
                })
            });
            to_shape(&normalize_json(&parsed))
        };

        snapshots.push(SnapshotEntry {
            method: method.to_string(),
            path: route.to_string(),
            status,
            location,
            body_shape,
        });
    }

    snapshots.sort_by(|a, b| (a.method.as_str(), a.path.as_str()).cmp(&(b.method.as_str(), b.path.as_str())));
    let current = stable_json_bytes(&snapshots).expect("snapshot bytes");
    let golden_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots/api-surface.responses.v1.json");

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        if let Some(parent) = golden_path.parent() {
            std::fs::create_dir_all(parent).expect("create golden dir");
        }
        std::fs::write(&golden_path, &current).expect("write golden snapshot");
        return;
    }

    let expected = std::fs::read(&golden_path).expect("read golden snapshot");
    assert_eq!(current, expected, "api surface response snapshot drift");
}
