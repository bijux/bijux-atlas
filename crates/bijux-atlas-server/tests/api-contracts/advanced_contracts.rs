use super::*;

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
    let payload: serde_json::Value = serde_json::from_str(&body).expect("json payload");
    assert!(
        payload.get("data").is_some(),
        "payload missing data: {body}"
    );
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
    let (status, _, body) = send_raw(addr, path, &[("x-forwarded-for", "1.2.3.4, 8.8.8.8")]).await;
    assert_eq!(status, 429);
    let json: Value = serde_json::from_str(&body).expect("rate limit error json");
    assert_eq!(
        json.get("error")
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str),
        Some("RateLimited")
    );
    assert!(json
        .get("error")
        .and_then(|e| e.get("request_id"))
        .and_then(Value::as_str)
        .is_some());
}

#[tokio::test]
async fn request_length_limits_return_400_error_envelope() {
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
        max_uri_bytes: 80,
        max_header_bytes: 80,
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(cache, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&name_prefix=abcdefghijklmnopqrstuvwxyz",
        &[],
    )
    .await;
    assert_eq!(status, 400);
    assert!(body.contains("request URI too large"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1",
        &[("x-overflow-header", &"x".repeat(256))],
    )
    .await;
    assert_eq!(status, 400);
    assert!(body.contains("request headers too large"));
}

#[tokio::test]
async fn query_budget_caps_return_expected_status_codes() {
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
    let limits = bijux_atlas_query::QueryLimits {
        max_region_span: 10,
        ..Default::default()
    };
    let app = build_router(AppState::with_config(cache, ApiConfig::default(), limits));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=99999",
        &[],
    )
    .await;
    assert_eq!(status, 400);
    assert!(body.contains("invalid query parameter: limit"));

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-1000",
        &[],
    )
    .await;
    assert_eq!(status, 413);
    let json: Value = serde_json::from_str(&body).expect("range error json");
    assert_eq!(
        json.get("error")
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str),
        Some("RangeTooLarge")
    );
}

#[tokio::test]
async fn safety_valve_disables_heavy_endpoints_but_keeps_cheap_endpoints_available() {
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
        disable_heavy_endpoints: true,
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(cache, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (heavy_status, _, heavy_body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1",
        &[],
    )
    .await;
    assert_eq!(heavy_status, 503);
    let heavy_json: Value = serde_json::from_str(&heavy_body).expect("heavy error json");
    let heavy_code = heavy_json
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(Value::as_str)
        .or_else(|| heavy_json.get("code").and_then(Value::as_str));
    assert_eq!(heavy_code, Some("QueryRejectedByPolicy"));

    let (cheap_status, _, cheap_body) = send_raw(addr, "/v1/version", &[]).await;
    assert_eq!(cheap_status, 200, "cheap endpoint should stay available");
    assert!(cheap_body.contains("\"api_version\""));
}

#[tokio::test]
async fn cheap_endpoint_remains_available_while_noncheap_is_shed() {
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
        shed_load_enabled: true,
        memory_pressure_shed_enabled: true,
        memory_pressure_rss_bytes: 1,
        ..ApiConfig::default()
    };
    let app = build_router(AppState::with_config(cache, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let (cheap_status, cheap_headers, _) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        &[],
    )
    .await;
    assert_eq!(cheap_status, 200);
    assert!(cheap_headers.contains("x-atlas-query-class: cheap"));

    let (heavy_status, _, heavy_body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-10&limit=50",
        &[],
    )
    .await;
    assert!(matches!(heavy_status, 503 | 413 | 422));
    assert!(heavy_body.contains("\"error\""));
}

#[tokio::test]
async fn canonical_dataset_endpoint_and_legacy_redirect_are_available() {
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

    let (status, headers, _) = send_raw(
        addr,
        "/v1/releases/110/species/homo_sapiens/assemblies/GRCh38?include_bom=1&x=1",
        &[],
    )
    .await;
    assert_eq!(status, 308);
    assert!(headers.contains("location: /v1/datasets/110/homo_sapiens/GRCh38?include_bom=1&x=1"));
    assert!(headers.contains(
        "link: </v1/datasets/110/homo_sapiens/GRCh38?include_bom=1&x=1>; rel=\"canonical\""
    ));

    let (status, _, body) = send_raw(
        addr,
        "/v1/datasets/110/homo_sapiens/GRCh38?include_bom=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let etag = header_value(
        &send_raw(
            addr,
            "/v1/datasets/110/homo_sapiens/GRCh38?include_bom=1",
            &[],
        )
        .await
        .1,
        "etag",
    )
    .expect("etag");
    let (status, _, _) = send_raw(
        addr,
        "/v1/datasets/110/homo_sapiens/GRCh38?include_bom=1",
        &[("If-None-Match", &etag)],
    )
    .await;
    assert_eq!(status, 304);
    let json: Value = serde_json::from_str(&body).expect("release metadata json");
    assert!(json.get("data").and_then(|d| d.get("item")).is_some());
    assert!(json
        .get("data")
        .and_then(|d| d.get("item"))
        .and_then(|i| i.get("artifact_hash"))
        .is_some());
    assert!(json
        .get("data")
        .and_then(|d| d.get("item"))
        .and_then(|i| i.get("bill_of_materials"))
        .is_some());

    let (status, _, _) = send_raw(addr, "/v1/datasets/110/homo%20sapiens/GRCh38", &[]).await;
    assert_eq!(status, 400);

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1&explain=1",
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let json: Value = serde_json::from_str(&body).expect("genes explain json");
    assert!(json.get("data").and_then(|d| d.get("explain")).is_some());

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
        json.get("data")
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
        json.get("data")
            .and_then(|d| d.get("transcript"))
            .and_then(|t| t.get("transcript_id"))
            .and_then(Value::as_str),
        Some("tx1")
    );
}

#[tokio::test]
async fn datasets_endpoint_supports_dimension_filters_and_cursor_pagination() {
    let (ds, manifest, sqlite) = mk_dataset();
    let ds2 = DatasetId::new("111", "homo_sapiens", "GRCh38").expect("dataset id");
    let ds3 = DatasetId::new("110", "mus_musculus", "GRCm39").expect("dataset id");
    let store = Arc::new(FakeStore::default());
    store
        .manifest
        .lock()
        .await
        .insert(ds.clone(), manifest.clone());
    store.sqlite.lock().await.insert(ds.clone(), sqlite.clone());
    store
        .manifest
        .lock()
        .await
        .insert(ds2.clone(), manifest.clone());
    store
        .sqlite
        .lock()
        .await
        .insert(ds2.clone(), sqlite.clone());
    store.manifest.lock().await.insert(ds3.clone(), manifest);
    store.sqlite.lock().await.insert(ds3.clone(), sqlite);
    *store.etag.lock().await = "v1".to_string();
    store.catalog.lock().await.datasets = vec![
        bijux_atlas_model::CatalogEntry::new(
            ds,
            "manifest-1.json".to_string(),
            "gene_summary-1.sqlite".to_string(),
        ),
        bijux_atlas_model::CatalogEntry::new(
            ds2,
            "manifest-2.json".to_string(),
            "gene_summary-2.sqlite".to_string(),
        ),
        bijux_atlas_model::CatalogEntry::new(
            ds3,
            "manifest-3.json".to_string(),
            "gene_summary-3.sqlite".to_string(),
        ),
    ];

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

    let (status, _, body) = send_raw(addr, "/v1/datasets?release=110&limit=1", &[]).await;
    assert_eq!(status, 200);
    let page1: Value = serde_json::from_str(&body).expect("json");
    let items = page1["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    let cursor = page1["page"]["next_cursor"]
        .as_str()
        .expect("next cursor")
        .to_string();

    let (status, _, body) = send_raw(
        addr,
        &format!("/v1/datasets?release=110&limit=1&cursor={cursor}"),
        &[],
    )
    .await;
    assert_eq!(status, 200);
    let page2: Value = serde_json::from_str(&body).expect("json");
    let items = page2["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert!(page2["page"]["next_cursor"].is_null());
}

#[tokio::test]
async fn sqlite_progress_handler_timeout_aborts_query() {
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
    let conn = mgr
        .open_dataset_connection(&ds)
        .await
        .expect("open dataset connection");
    conn.conn.progress_handler(1, Some(|| true));
    let err = conn
        .conn
        .query_row(
            "SELECT count(*) FROM gene_summary a, gene_summary b, gene_summary c",
            [],
            |row| row.get::<_, i64>(0),
        )
        .expect_err("progress handler must interrupt query");
    assert!(
        err.to_string().to_lowercase().contains("interrupt"),
        "unexpected sqlite error: {err}"
    );
    conn.conn.progress_handler(1, None::<fn() -> bool>);
}
