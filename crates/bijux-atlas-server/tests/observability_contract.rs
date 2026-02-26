// SPDX-License-Identifier: Apache-2.0

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

#[path = "observability/logging_format.rs"]
mod logging_format;

#[derive(Debug, Deserialize)]
struct MetricsContract {
    required_metrics: HashMap<String, Vec<String>>,
    #[serde(default)]
    required_metric_specs: HashMap<String, serde_json::Value>,
    required_spans: Vec<String>,
}

fn parse_metric_sum(metrics_body: &str, metric_name: &str) -> f64 {
    metrics_body
        .lines()
        .filter_map(|line| {
            if line.starts_with(&format!("{metric_name}{{")) || line.starts_with(metric_name) {
                line.rsplit_once(' ')
                    .and_then(|(_, v)| v.trim().parse::<f64>().ok())
            } else {
                None
            }
        })
        .sum()
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

async fn send_raw_with_headers(
    addr: std::net::SocketAddr,
    path: &str,
    headers: &[(&str, &str)],
) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect server");
    let mut req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (name, value) in headers {
        req.push_str(&format!("{name}: {value}\r\n"));
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
        "/v1/datasets/110/homo_sapiens/GRCh38",
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
            matches!(status, 200 | 304 | 308 | 400 | 401 | 404 | 422 | 503),
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
    let contract_path = root
        .join("ops")
        .join("observe")
        .join("contracts")
        .join("metrics-contract.json");
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

#[tokio::test]
async fn request_id_header_is_present_across_core_api_routes() {
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
    store.sqlite.lock().await.insert(ds, sqlite);

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

    for path in [
        "/v1/version",
        "/healthz",
        "/readyz",
        "/v1/datasets",
        "/v1/datasets/110/homo_sapiens/GRCh38",
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1",
        "/v1/transcripts/tx1?release=110&species=homo_sapiens&assembly=GRCh38",
        "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-5",
    ] {
        let (status, headers, _body) = send_raw(addr, path).await;
        assert!(
            matches!(status, 200 | 304 | 308 | 400 | 401 | 404 | 422 | 503),
            "unexpected status {status} for {path}"
        );
        let lower = headers.to_ascii_lowercase();
        assert!(
            lower.contains("x-request-id: req-") || lower.contains("x-request-id: trace-"),
            "missing x-request-id for {path}: {headers}"
        );
    }
}

#[tokio::test]
async fn request_tracing_preserves_explicit_request_id() {
    let store = Arc::new(FakeStore::default());
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

    let (status, headers, _body) = send_raw_with_headers(
        addr,
        "/healthz",
        &[
            ("x-request-id", "req-client-1"),
            ("x-correlation-id", "corr-client-1"),
            ("x-run-id", "run-client-1"),
        ],
    )
    .await;
    assert_eq!(status, 200);
    assert!(
        headers
            .to_ascii_lowercase()
            .contains("x-request-id: req-client-1"),
        "server must preserve explicit x-request-id: {headers}"
    );
}

#[tokio::test]
async fn generated_metrics_contract_covers_ops_metrics_contract_and_owners() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let ops_contract: MetricsContract = serde_json::from_slice(
        &std::fs::read(
            root.join("ops")
                .join("observe")
                .join("contracts")
                .join("metrics-contract.json"),
        )
        .expect("read ops metrics contract"),
    )
    .expect("parse ops metrics contract");

    let generated_metrics = std::fs::read_to_string(
        root.join("crates/bijux-atlas-server/src/telemetry/generated/metrics_contract.rs"),
    )
    .expect("read generated metrics contract");

    for metric in ops_contract.required_metrics.keys() {
        assert!(
            generated_metrics.contains(&format!("\"{metric}\"")),
            "generated metrics contract missing metric {metric}"
        );
    }

    if !ops_contract.required_metric_specs.is_empty() {
        for (metric, spec) in &ops_contract.required_metric_specs {
            let owner = spec.get("owner").expect("owner object");
            let owner_crate = owner
                .get("crate")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let owner_module = owner
                .get("module")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            assert!(
                !owner_crate.is_empty(),
                "metric {metric} missing owner.crate in contract"
            );
            assert!(
                !owner_module.is_empty(),
                "metric {metric} missing owner.module in contract"
            );
            let owner_path = root.join("crates").join(owner_crate).join(owner_module);
            assert!(
                owner_path.exists(),
                "metric {metric} owner.module path does not exist: {}",
                owner_path.display()
            );
        }
    }
}

#[tokio::test]
async fn policy_rejection_and_overload_emit_contract_metrics() {
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
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds, sqlite);

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

    let (status_before, _, metrics_before) = send_raw(addr, "/metrics").await;
    assert_eq!(status_before, 200);
    let policy_before = parse_metric_sum(&metrics_before, "atlas_policy_violations_total");
    let shed_before = parse_metric_sum(&metrics_before, "atlas_shed_total");

    let (policy_status, _, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&foo=bar",
    )
    .await;
    assert_eq!(policy_status, 400);

    let (_ov_status, _, _ov_body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-999999999&limit=500",
    )
    .await;
    let (_genes_ok_status, _, _genes_ok_body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
    )
    .await;
    let (_ok_status, _, _ok_body) = send_raw(addr, "/v1/version").await;

    let (status_after, _, metrics_after) = send_raw(addr, "/metrics").await;
    assert_eq!(status_after, 200);
    let policy_after = parse_metric_sum(&metrics_after, "atlas_policy_violations_total");
    let shed_after = parse_metric_sum(&metrics_after, "atlas_shed_total");
    assert!(
        policy_after >= policy_before,
        "policy violations total should not decrease"
    );
    assert!(shed_after >= shed_before, "shed total should not decrease");

    assert!(
        metrics_after.contains("bijux_http_request_size_p95_bytes"),
        "request size histogram metric missing"
    );
    assert!(
        metrics_after.contains("bijux_http_response_size_p95_bytes"),
        "response size histogram metric missing"
    );
    assert!(
        metrics_after.contains("atlas_client_requests_total"),
        "client fingerprint metric missing"
    );
    assert!(
        metrics_after.contains("bijux_store_breaker_half_open_total"),
        "store breaker half-open counter missing"
    );
    assert!(
        metrics_after.contains("bijux_store_breaker_open_current"),
        "store breaker current-open gauge missing"
    );
    assert!(
        metrics_after.contains("bijux_dataset_hits")
            || metrics_after.contains("bijux_dataset_misses"),
        "cache hit/miss metrics missing"
    );
}

#[tokio::test]
async fn slo_critical_metrics_present_after_smoke_query() {
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
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds, sqlite);
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

    let (status, _, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
    )
    .await;
    assert_eq!(status, 200);

    let (metrics_status, _, metrics_body) = send_raw(addr, "/metrics").await;
    assert_eq!(metrics_status, 200);
    for metric in [
        "http_requests_total",
        "http_request_duration_seconds_bucket",
        "atlas_overload_active",
        "atlas_shed_total",
        "atlas_cache_hits_total",
        "atlas_cache_misses_total",
        "atlas_store_request_duration_seconds_bucket",
        "atlas_store_errors_total",
        "atlas_registry_refresh_age_seconds",
        "atlas_registry_refresh_failures_total",
        "atlas_invariant_violations_total",
    ] {
        assert!(
            metrics_body.contains(metric),
            "slo-critical metric missing after smoke query: {metric}"
        );
    }
}

#[tokio::test]
async fn store_failures_emit_contract_metric_and_retryable_details() {
    let store = Arc::new(FakeStore::default());
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

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes/g1/sequence?release=110&species=homo_sapiens&assembly=GRCh38",
    )
    .await;
    assert_eq!(status, 503);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("parse json body");
    assert_eq!(
        parsed["error"]["code"].as_str(),
        Some("UpstreamStoreUnavailable")
    );
    assert_eq!(
        parsed["error"]["details"]["retryable"].as_bool(),
        Some(true)
    );

    let (metrics_status, _, metrics_body) = send_raw(addr, "/metrics").await;
    assert_eq!(metrics_status, 200);
    let store_errors = parse_metric_sum(&metrics_body, "atlas_store_errors_total");
    assert!(
        store_errors >= 1.0,
        "expected atlas_store_errors_total >= 1.0, got {store_errors}"
    );
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
