use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats, ReleaseGeneIndex,
    ReleaseGeneIndexEntry,
};
use bijux_atlas_server::{
    build_router, ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use hmac::{Hmac, Mac};
use rusqlite::Connection;
use serde_json::Value;
use sha2::Sha256;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

fn mk_dataset() -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let (fasta, fai) = fixture_fasta_and_fai();
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        ArtifactChecksums::new(
            "a".repeat(64),
            sha256_hex(&fasta),
            sha256_hex(&fai),
            sqlite_sha,
        ),
        ManifestStats::new(1, 1, 1),
    );
    (ds, manifest, sqlite)
}

fn fixture_fasta_and_fai() -> (Vec<u8>, Vec<u8>) {
    let fasta = b">chr1\nACGTACGTAC\nGGGGnnnnTT\n".to_vec();
    let fai = b"chr1\t20\t6\t10\t11\n".to_vec();
    (fasta, fai)
}

fn fixture_release_index(dataset: &DatasetId, rows: Vec<(&str, &str, u64, u64, &str)>) -> Vec<u8> {
    let mut entries: Vec<ReleaseGeneIndexEntry> = rows
        .into_iter()
        .map(|(gene_id, seqid, start, end, sig)| {
            ReleaseGeneIndexEntry::new(
                gene_id.to_string(),
                seqid.to_string(),
                start,
                end,
                sig.to_string(),
            )
        })
        .collect();
    entries.sort();
    serde_json::to_vec(&ReleaseGeneIndex::new(
        "1".to_string(),
        dataset.clone(),
        entries,
    ))
    .expect("index json")
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

    let (status, _, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);

    let (status, _, body) = send_raw(addr, "/metrics", &[]).await;
    assert_eq!(status, 200);
    assert!(body.contains("bijux_request_stage_latency_p95_seconds"));
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
    assert!(body.contains("bijux_overload_shedding_active"));
    assert!(body.contains("bijux_cached_only_mode"));

    let (status, _, _) = send_raw(addr, "/debug/datasets", &[]).await;
    assert_eq!(status, 404);
    let (status, _, _) = send_raw(
        addr,
        "/debug/dataset-health?release=110&species=homo_sapiens&assembly=GRCh38",
        &[],
    )
    .await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn overload_health_endpoint_reports_state() {
    let tmp = tempfile::tempdir().expect("tmp");
    let store = Arc::new(FakeStore::default());
    let cache = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            cached_only_mode: true,
            ..DatasetCacheConfig::default()
        },
        store,
    );
    let api = ApiConfig {
        shed_load_enabled: true,
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(
        cache,
        api,
        bijux_atlas_query::QueryLimits::default(),
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });
    let (status, _, body) = send_raw(addr, "/healthz/overload", &[]).await;
    assert!(status == 200 || status == 503);
    let json: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert!(json.get("overloaded").is_some());
}

#[tokio::test]
async fn readiness_allows_cached_only_without_catalog() {
    let store = Arc::new(FakeStore::default());
    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        cached_only_mode: true,
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let api = ApiConfig {
        readiness_requires_catalog: true,
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
}

#[tokio::test]
async fn memory_pressure_guards_reject_large_response_without_cascading_failure() {
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
        response_max_bytes: 64,
        ..ApiConfig::default()
    };
    let state = AppState::with_config(mgr, api, Default::default());
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        &[],
    )
    .await;
    assert!(status == 413 || status == 422);

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    assert!(body.contains("gene_count"));
}

#[tokio::test]
async fn sequence_endpoint_boundary_conditions_are_enforced() {
    let (ds, manifest, sqlite) = mk_dataset();
    let (fasta, fai) = fixture_fasta_and_fai();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    store.fasta.lock().await.insert(ds.clone(), fasta);
    store.fai.lock().await.insert(ds.clone(), fai);

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let api = ApiConfig {
        max_sequence_bases: 8,
        sequence_api_key_required_bases: 6,
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(mgr, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(
        addr,
        "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chrX:1-2",
        &[],
    )
    .await;
    assert_eq!(status, 422);
    assert!(body.contains("contig not found"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:10-2",
        &[],
    )
    .await;
    assert_eq!(status, 400);
    assert!(body.contains("invalid region"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-30",
        &[],
    )
    .await;
    assert_eq!(status, 422);
    assert!(body.contains("requested region exceeds max bases"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-7",
        &[],
    )
    .await;
    assert_eq!(status, 401);
    assert!(body.contains("api key required"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes/g1/sequence?release=110&species=homo_sapiens&assembly=GRCh38",
        &[("x-api-key", "k1")],
    )
    .await;
    assert_eq!(status, 422);
    assert!(body.contains("requested region exceeds max bases"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-5&include_stats=1",
        &[("x-api-key", "k1")],
    )
    .await;
    assert_eq!(status, 200);
    assert!(body.contains("\"gc_fraction\""));
}

#[tokio::test]
async fn diff_endpoints_return_added_removed_changed_and_support_latest_alias() {
    let (ds_from, manifest_from, sqlite_from) = mk_dataset();
    let ds_to = DatasetId::new("111", "homo_sapiens", "GRCh38").expect("dataset id");
    let (fasta, fai) = fixture_fasta_and_fai();
    let manifest_to = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds_to.clone(),
        ArtifactChecksums::new(
            "a".repeat(64),
            sha256_hex(&fasta),
            sha256_hex(&fai),
            sha256_hex(&sqlite_from),
        ),
        ManifestStats::new(2, 2, 1),
    );

    let store = Arc::new(FakeStore::default());
    store
        .manifest
        .lock()
        .await
        .insert(ds_from.clone(), manifest_from);
    store
        .manifest
        .lock()
        .await
        .insert(ds_to.clone(), manifest_to);
    store
        .sqlite
        .lock()
        .await
        .insert(ds_from.clone(), sqlite_from.clone());
    store.sqlite.lock().await.insert(ds_to.clone(), sqlite_from);
    store
        .fasta
        .lock()
        .await
        .insert(ds_from.clone(), fasta.clone());
    store.fasta.lock().await.insert(ds_to.clone(), fasta);
    store.fai.lock().await.insert(ds_from.clone(), fai.clone());
    store.fai.lock().await.insert(ds_to.clone(), fai);
    store.release_gene_index.lock().await.insert(
        ds_from.clone(),
        fixture_release_index(
            &ds_from,
            vec![
                ("gA", "chr1", 1, 10, "sig-a1"),
                ("gB", "chr1", 20, 30, "sig-b1"),
            ],
        ),
    );
    store.release_gene_index.lock().await.insert(
        ds_to.clone(),
        fixture_release_index(
            &ds_to,
            vec![
                ("gB", "chr1", 20, 30, "sig-b2"),
                ("gC", "chr1", 40, 50, "sig-c1"),
            ],
        ),
    );
    let catalog = bijux_atlas_model::Catalog::new(vec![
        bijux_atlas_model::CatalogEntry::new(ds_from.clone(), "m1".to_string(), "s1".to_string()),
        bijux_atlas_model::CatalogEntry::new(ds_to.clone(), "m2".to_string(), "s2".to_string()),
    ]);
    *store.catalog.lock().await = catalog;
    *store.etag.lock().await = "catalog-diff".to_string();

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let app = build_router(AppState::with_config(
        mgr,
        ApiConfig::default(),
        Default::default(),
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(
        addr,
        "/v1/diff/genes?from_release=110&to_release=latest&species=homo_sapiens&assembly=GRCh38&limit=10",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    assert!(body.contains("\"Added\"") || body.contains("\"added\""));
    assert!(body.contains("\"Removed\"") || body.contains("\"removed\""));
    assert!(body.contains("\"Changed\"") || body.contains("\"changed\""));

    let (status, _, body) = send_raw(
        addr,
        "/v1/diff/region?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&region=chr1:35-60&limit=10",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    assert!(body.contains("\"gC\""));
    assert!(!body.contains("\"gA\""));
}

fn sign_hmac(secret: &str, method: &str, uri: &str, ts: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac init");
    let payload = format!("{method}\n{uri}\n{ts}\n");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[tokio::test]
async fn security_limits_api_key_hmac_and_cors_are_enforced() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds, sqlite);
    let tmp = tempdir().expect("tempdir");
    let cache = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..DatasetCacheConfig::default()
        },
        store,
    );
    let api = ApiConfig {
        max_uri_bytes: 128,
        max_header_bytes: 1024,
        require_api_key: true,
        allowed_api_keys: vec!["k1".to_string()],
        hmac_secret: Some("s3cr3t".to_string()),
        hmac_required: true,
        cors_allowed_origins: vec!["https://atlas.example.org".to_string()],
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(cache, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(addr, "/v1/datasets", &[]).await;
    assert_eq!(status, 401);
    assert!(body.contains("api key required"));

    let (status, _, body) = send_raw(addr, "/v1/datasets", &[("x-api-key", "bad")]).await;
    assert_eq!(status, 401);
    assert!(body.contains("invalid api key"));

    let ts = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("unix")
        .as_secs())
    .to_string();
    let uri = "/v1/datasets";
    let sig = sign_hmac("s3cr3t", "GET", uri, &ts);
    let (status, _, _) = send_raw(
        addr,
        uri,
        &[
            ("x-api-key", "k1"),
            ("x-bijux-timestamp", &ts),
            ("x-bijux-signature", &sig),
            ("Origin", "https://atlas.example.org"),
        ],
    )
    .await;
    assert_eq!(status, 200);

    let (status, headers, _) = send_raw(
        addr,
        uri,
        &[
            ("x-api-key", "k1"),
            ("x-bijux-timestamp", &ts),
            ("x-bijux-signature", &sig),
            ("Origin", "https://atlas.example.org"),
        ],
    )
    .await;
    assert_eq!(status, 200);
    assert!(headers.contains("access-control-allow-origin: https://atlas.example.org"));
    let (status, headers, _) = send_raw(
        addr,
        uri,
        &[
            ("x-api-key", "k1"),
            ("x-bijux-timestamp", &ts),
            ("x-bijux-signature", &sig),
            ("Origin", "https://evil.example.org"),
        ],
    )
    .await;
    assert_eq!(status, 200);
    assert!(!headers.contains("access-control-allow-origin"));

    let (status, _, body) = send_raw(
        addr,
        uri,
        &[("x-api-key", "k1"), ("x-bijux-timestamp", &ts)],
    )
    .await;
    assert_eq!(status, 401);
    assert!(body.contains("missing required HMAC headers"));
}

#[tokio::test]
async fn rate_limit_bypass_prevention_uses_normalized_forwarded_ip() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    let tmp = tempdir().expect("tempdir");
    let cache = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..DatasetCacheConfig::default()
        },
        store,
    );
    let api = ApiConfig {
        rate_limit_per_ip: bijux_atlas_server::RateLimitConfig {
            capacity: 1.0,
            refill_per_sec: 0.0,
        },
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(cache, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let path = "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1";
    let (status, _, _) = send_raw(addr, path, &[("x-forwarded-for", "1.2.3.4, 9.9.9.9")]).await;
    assert_ne!(status, 429);
    let (status, _, _) = send_raw(addr, path, &[("x-forwarded-for", "1.2.3.4, 8.8.8.8")]).await;
    assert_eq!(status, 429);
}

#[tokio::test]
async fn release_metadata_endpoint_and_explain_mode_are_available() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();
    store.catalog.lock().await.datasets = vec![bijux_atlas_model::CatalogEntry::new(
        ds.clone(),
        "manifest.json".to_string(),
        "gene_summary.sqlite".to_string(),
    )];

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let state = AppState::with_config(
        mgr,
        ApiConfig {
            enable_debug_datasets: true,
            ..ApiConfig::default()
        },
        Default::default(),
    );
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(
        addr,
        "/v1/releases/110/species/homo_sapiens/assemblies/GRCh38?include_bom=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("release metadata json");
    assert!(json.get("manifest_summary").is_some());
    assert!(json.get("qc_summary").is_some());
    assert!(json.get("bill_of_materials").is_some());

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1&explain=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("genes explain json");
    assert!(json.get("explain").is_some());

    let (status, _, body) = send_raw(
        addr,
        "/debug/dataset-health?release=110&species=homo_sapiens&assembly=GRCh38",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("dataset health json");
    assert!(json
        .get("health")
        .and_then(|h| h.get("cached"))
        .and_then(Value::as_bool)
        .is_some());

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes/g1/transcripts?release=110&species=homo_sapiens&assembly=GRCh38&limit=10",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("gene transcripts json");
    assert_eq!(
        json.get("response")
            .and_then(|r| r.get("rows"))
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        Some(1)
    );

    let (status, _, body) = send_raw(
        addr,
        "/v1/transcripts/tx1?release=110&species=homo_sapiens&assembly=GRCh38",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("transcript summary json");
    assert_eq!(
        json.get("transcript")
            .and_then(|t| t.get("transcript_id"))
            .and_then(Value::as_str),
        Some("tx1")
    );
}
