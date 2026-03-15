// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use bijux_atlas::adapters::inbound::http::router::build_router;
use bijux_atlas::app::server::{AppState, DatasetCacheConfig, DatasetCacheManager};
use bijux_atlas::runtime::config::ApiConfig;
use bijux_atlas::runtime::wiring::server::FakeStore;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tempfile::tempdir;

mod api_contracts_support;

use api_contracts_support::{mk_dataset, send_raw, send_raw_with_method};

fn sign_hmac(secret: &str, method: &str, uri: &str, ts: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
    let payload = format!("{method}\n{uri}\n{ts}\n");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

async fn spawn_server(api: ApiConfig) -> std::net::SocketAddr {
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
    let app = build_router(AppState::with_config(cache, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });
    addr
}

#[tokio::test]
async fn fuzz_api_endpoints_with_malformed_requests_stay_bounded() {
    let addr = spawn_server(ApiConfig::default()).await;
    let paths = [
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=' OR 1=1 --",
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=%00%0a%0d",
        "/v1/query/validate",
    ];

    for path in paths {
        let (status, _, _) = if path == "/v1/query/validate" {
            send_raw_with_method(
                addr,
                "POST",
                path,
                &[("content-type", "application/json")],
                Some("{\"query\":\"\\u0000<script>alert(1)</script>\"}"),
            )
            .await
        } else {
            send_raw(addr, path, &[]).await
        };
        assert_ne!(status, 500, "unexpected 500 for {path}");
    }
}

#[tokio::test]
async fn fuzz_authentication_headers_do_not_crash_and_reject_invalid_credentials() {
    let api = ApiConfig {
        require_api_key: true,
        allowed_api_keys: vec!["k1".to_string()],
        hmac_secret: Some("s3cr3t".to_string()),
        hmac_required: true,
        ..ApiConfig::default()
    };
    let addr = spawn_server(api).await;

    let bad_headers = [
        vec![("x-api-key", "bad")],
        vec![("x-api-key", "k1"), ("x-bijux-signature", "bad")],
        vec![("x-api-key", "k1"), ("x-bijux-timestamp", "-1")],
        vec![("authorization", "Bearer not.a.valid.token")],
    ];

    for headers in bad_headers {
        let (status, _, _) = send_raw(addr, "/v1/datasets", &headers).await;
        assert!(status == 401 || status == 400 || status == 422);
        assert_ne!(status, 500);
    }
}

#[tokio::test]
async fn fuzz_authorization_and_overload_paths_keep_policy_responses() {
    let api = ApiConfig {
        enable_admin_endpoints: true,
        require_api_key: true,
        allowed_api_keys: vec!["k1".to_string()],
        hmac_secret: Some("s3cr3t".to_string()),
        hmac_required: true,
        ..ApiConfig::default()
    };
    let addr = spawn_server(api).await;
    let ts = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("unix time")
        .as_secs())
    .to_string();
    let sig = sign_hmac("s3cr3t", "GET", "/debug/datasets", &ts);

    let (status, _, _) = send_raw(
        addr,
        "/debug/datasets",
        &[
            ("x-api-key", "k1"),
            ("x-bijux-timestamp", &ts),
            ("x-bijux-signature", &sig),
        ],
    )
    .await;
    assert!(status == 200 || status == 403 || status == 404);

    for _ in 0..40 {
        let (s, _, _) = send_raw(
            addr,
            "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=999999",
            &[],
        )
        .await;
        assert!(matches!(s, 200 | 400 | 401 | 403 | 413 | 422 | 429 | 503));
        assert_ne!(s, 500);
    }
}
