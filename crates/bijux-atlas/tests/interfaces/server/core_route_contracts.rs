// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use bijux_atlas::adapters::inbound::http::router::build_router;
use bijux_atlas::adapters::outbound::store::testing::FakeStore;
use bijux_atlas::app::server::{AppState, DatasetCacheConfig, DatasetCacheManager};
use serde_json::Value;
use tempfile::tempdir;

use super::api_contracts_support::{mk_dataset, send_raw};

fn header_value(headers: &str, name: &str) -> Option<String> {
    let prefix = format!("{}: ", name.to_ascii_lowercase());
    headers
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|s| s.trim().to_string())
}

#[tokio::test]
async fn core_route_contract_statuses_and_headers_are_exact() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds, sqlite);
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
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });

    let core_cases = [
        ("GET /healthz", "/healthz", 200, "text/plain"),
        ("GET /readyz", "/readyz", 503, "text/plain"),
        ("GET /v1/version", "/v1/version", 200, "application/json"),
        (
            "GET /v1/datasets",
            "/v1/datasets?limit=1",
            200,
            "application/json",
        ),
        (
            "GET /v1/genes",
            "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
            200,
            "application/json",
        ),
    ];

    for (name, path, expected_status, expected_content_type_prefix) in core_cases {
        let (status, headers, body) = send_raw(addr, path, &[]).await;
        assert_eq!(status, expected_status, "{name}");
        let content_type = header_value(&headers, "content-type").unwrap_or_default();
        assert!(
            content_type.starts_with(expected_content_type_prefix),
            "{name} content-type mismatch: {content_type}"
        );
        assert!(
            header_value(&headers, "x-request-id").is_some(),
            "{name} missing request id"
        );
        if expected_content_type_prefix == "application/json" {
            let _json: Value = serde_json::from_str(&body).expect("valid json body");
        }
    }
}
