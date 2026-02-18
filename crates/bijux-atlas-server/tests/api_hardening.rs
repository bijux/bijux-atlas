use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use tempfile::tempdir;
mod api_hardening_support;

use api_hardening_support::{
    fixture_fasta_and_fai, fixture_release_index, mk_dataset, send_raw, send_raw_with_method,
};

fn header_value(headers: &str, name: &str) -> Option<String> {
    let prefix = format!("{}: ", name.to_ascii_lowercase());
    headers
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|s| s.trim().to_string())
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
    assert_eq!(json.get("api_version").and_then(Value::as_str), Some("v1"));
    assert_eq!(
        json.get("contract_version").and_then(Value::as_str),
        Some("v1")
    );
    assert_eq!(
        json.get("server")
            .and_then(|s| s.get("api_contract_version"))
            .and_then(Value::as_str),
        Some("v1")
    );
    assert!(json
        .get("server")
        .and_then(|s| s.get("runtime_policy_hash"))
        .and_then(Value::as_str)
        .is_some());
    assert_eq!(
        json.get("server")
            .and_then(|s| s.get("artifact_schema_versions"))
            .and_then(|v| v.get("manifest_schema_version"))
            .and_then(Value::as_str),
        Some("1")
    );

    let (status, _, body) = send_raw(addr, "/v1/openapi.json", &[]).await;
    assert_eq!(status, 200);
    let openapi: Value = serde_json::from_str(&body).expect("openapi json");
    assert_eq!(
        openapi
            .get("info")
            .and_then(|v| v.get("x-api-contract-version"))
            .and_then(Value::as_str),
        Some("v1")
    );
    assert!(openapi
        .get("info")
        .and_then(|v| v.get("x-build-id"))
        .and_then(Value::as_str)
        .is_some());

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&include=nope",
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
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&foo=bar",
        &[],
    )
    .await;
    assert_eq!(status, 400);
    let json: Value = serde_json::from_str(&body).expect("unknown filter json");
    assert!(json
        .get("error")
        .and_then(|e| e.get("details"))
        .and_then(|d| d.get("field_errors"))
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|row| row.get("value"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .contains("allowed"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&fields=gene_id",
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
    let etag = header_value(&headers, "etag").expect("etag header present");
    assert!(header_value(&headers, "cache-control")
        .unwrap_or_default()
        .contains("stale-while-revalidate"));
    assert_eq!(
        header_value(&headers, "vary").as_deref(),
        Some("accept-encoding")
    );
    let (status, _, _) = send_raw(addr, "/v1/datasets", &[("If-None-Match", &etag)]).await;
    assert_eq!(status, 304);

    let (status, _, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let (status, headers, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    assert!(headers.contains("x-atlas-query-class: cheap"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&range=chrX:1-2",
        &[],
    )
    .await;
    assert_eq!(status, 400);
    assert!(body.contains("did you mean chr1"));

    let (status, _, body) = send_raw(addr, "/metrics", &[]).await;
    assert_eq!(status, 200);
    assert!(body.contains("bijux_request_stage_latency_p95_seconds"));
}

#[tokio::test]
async fn etag_stable_across_restart_for_same_artifact_and_request() {
    let (ds, manifest, sqlite) = mk_dataset();
    let request = "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1";

    async fn etag_for_request(
        ds: DatasetId,
        manifest: ArtifactManifest,
        sqlite: Vec<u8>,
        request: &str,
    ) -> String {
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
        let (status, headers, _) = send_raw(addr, request, &[]).await;
        assert_eq!(status, 200);
        header_value(&headers, "etag").expect("etag")
    }

    let etag_a = etag_for_request(ds.clone(), manifest.clone(), sqlite.clone(), request).await;
    let etag_b = etag_for_request(ds, manifest, sqlite, request).await;
    assert_eq!(etag_a, etag_b);
}

#[tokio::test]
async fn etag_changes_when_artifact_hash_changes() {
    let (ds, mut manifest_a, sqlite) = mk_dataset();
    let mut manifest_b = manifest_a.clone();
    manifest_a.dataset_signature_sha256 = "artifact-hash-a".to_string();
    manifest_b.dataset_signature_sha256 = "artifact-hash-b".to_string();
    let request = "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1";

    async fn etag_for_request(
        ds: DatasetId,
        manifest: ArtifactManifest,
        sqlite: Vec<u8>,
        request: &str,
    ) -> String {
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
        let (status, headers, _) = send_raw(addr, request, &[]).await;
        assert_eq!(status, 200);
        header_value(&headers, "etag").expect("etag")
    }

    let etag_a = etag_for_request(ds.clone(), manifest_a, sqlite.clone(), request).await;
    let etag_b = etag_for_request(ds, manifest_b, sqlite, request).await;
    assert_ne!(etag_a, etag_b);
}

#[tokio::test]
async fn etag_changes_when_filters_change() {
    let (ds, manifest, sqlite) = mk_dataset();
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

    let (status, headers_a, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let (status, headers_b, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g2&limit=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);

    let etag_a = header_value(&headers_a, "etag").expect("etag a");
    let etag_b = header_value(&headers_b, "etag").expect("etag b");
    assert_ne!(etag_a, etag_b);
}

#[tokio::test]
async fn query_validate_endpoint_returns_classification() {
    let (ds, manifest, sqlite) = mk_dataset();
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
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let body = r#"{"release":"110","species":"homo_sapiens","assembly":"GRCh38","gene_id":"g1"}"#;
    let (status, headers, payload) =
        send_raw_with_method(addr, "POST", "/v1/query/validate", &[], Some(body)).await;
    assert_eq!(status, 200);
    assert!(headers.contains("x-atlas-query-class: cheap"));
    let json: Value = serde_json::from_str(&payload).expect("json");
    assert_eq!(json["data"]["query_class"], "cheap");
    assert!(json["data"]["limits"]["max_limit"].is_number());
    assert_eq!(json["data"]["reasons"][0], "gene_id");
}

#[tokio::test]
async fn debug_echo_is_gated_and_echoes_query_when_enabled() {
    let store = Arc::new(FakeStore::default());
    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let app = build_router(AppState::new(mgr.clone()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });
    let (status, _, _) = send_raw(addr, "/v1/_debug/echo?x=1", &[]).await;
    assert_eq!(status, 404);

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
    let (status, _, body) = send_raw(addr, "/v1/_debug/echo?x=1&y=2", &[]).await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("echo json");
    assert_eq!(json["data"]["query"]["x"], "1");
    assert_eq!(json["data"]["query"]["y"], "2");
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
async fn genes_count_applies_filters_consistently() {
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
    let state = AppState::with_config(mgr, ApiConfig::default(), Default::default());
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let payload: Value = serde_json::from_str(&body).expect("count json");
    assert_eq!(payload.get("gene_count").and_then(Value::as_i64), Some(1));

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38&biotype=nope",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let payload: Value = serde_json::from_str(&body).expect("count json");
    assert_eq!(payload.get("gene_count").and_then(Value::as_i64), Some(0));
}

#[tokio::test]
async fn expensive_include_is_policy_gated_by_projection_limits() {
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
    let state = AppState::with_config(mgr, ApiConfig::default(), Default::default());
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&include=length&limit=500",
        &[],
    )
    .await;
    assert_eq!(status, 422);
    let json: Value = serde_json::from_str(&body).expect("error json");
    assert_eq!(
        json.get("error")
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str),
        Some("QueryRejectedByPolicy")
    );
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

#[path = "api_hardening/advanced_contracts.rs"]
mod advanced_contracts;
