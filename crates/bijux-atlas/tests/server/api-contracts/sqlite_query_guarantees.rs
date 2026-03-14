// SPDX-License-Identifier: Apache-2.0

use super::*;

async fn spawn_with_store(
    store: Arc<FakeStore>,
    api: ApiConfig,
) -> (std::net::SocketAddr, tempfile::TempDir) {
    let tmp = tempdir().expect("tempdir");
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        },
        store,
    );
    let app = build_router(AppState::with_config(mgr, api, Default::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve app") });
    (addr, tmp)
}

async fn spawn_with_default_dataset(api: ApiConfig) -> (std::net::SocketAddr, tempfile::TempDir) {
    let (ds, manifest, sqlite) = mk_dataset();
    let (fasta, fai) = fixture_fasta_and_fai();
    let store = Arc::new(FakeStore::default());
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);
    store.fasta.lock().await.insert(ds.clone(), fasta);
    store.fai.lock().await.insert(ds, fai);
    spawn_with_store(store, api).await
}

#[tokio::test]
async fn genes_count_and_list_are_consistent_for_same_filter() {
    let (addr, _tmp) = spawn_with_default_dataset(ApiConfig::default()).await;

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1",
        &[],
    )
    .await;
    assert_eq!(status, 200, "count body={body}");
    let count = serde_json::from_str::<Value>(&body)
        .expect("count json")
        .get("gene_count")
        .and_then(Value::as_u64)
        .expect("gene_count");

    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
        &[],
    )
    .await;
    assert_eq!(status, 200, "list body={body}");
    let list_json = serde_json::from_str::<Value>(&body).expect("list json");
    let list_count = list_json
        .get("items")
        .and_then(Value::as_array)
        .map(|v| v.len() as u64)
        .or_else(|| {
            list_json
                .get("data")
                .and_then(|v| v.get("rows"))
                .and_then(Value::as_array)
                .map(|v| v.len() as u64)
        })
        .or_else(|| {
            list_json
                .get("data")
                .and_then(|v| v.get("data"))
                .and_then(Value::as_array)
                .map(|v| v.len() as u64)
        })
        .or_else(|| {
            list_json
                .get("data")
                .and_then(Value::as_object)
                .and_then(|m| m.values().find_map(Value::as_array))
                .map(|v| v.len() as u64)
        })
        .or_else(|| {
            if list_json.get("error").is_some() {
                None
            } else {
                Some(0)
            }
        })
        .expect("list items");
    assert_eq!(count, list_count);
}

#[tokio::test]
async fn malformed_range_filter_returns_validation_error() {
    let (addr, _tmp) = spawn_with_default_dataset(ApiConfig::default()).await;
    let (status, _, body) = send_raw(
        addr,
        "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&range=chr1:abc-def",
        &[],
    )
    .await;
    assert_eq!(status, 400, "body={body}");
    let json = serde_json::from_str::<Value>(&body).expect("error json");
    let code = json
        .get("error")
        .and_then(|v| v.get("code"))
        .and_then(Value::as_str);
    assert_eq!(code, Some("InvalidQueryParameter"));
}

#[tokio::test]
async fn sequence_region_is_stable_under_short_burst() {
    let (addr, _tmp) = spawn_with_default_dataset(ApiConfig {
        max_sequence_bases: 32,
        sequence_api_key_required_bases: usize::MAX,
        ..ApiConfig::default()
    })
    .await;
    for _ in 0..40 {
        let (status, _, body) = send_raw(
            addr,
            "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-8&include_stats=1",
            &[],
        )
        .await;
        assert_eq!(status, 200, "body={body}");
        assert!(body.contains("\"sequence\""));
    }
}
