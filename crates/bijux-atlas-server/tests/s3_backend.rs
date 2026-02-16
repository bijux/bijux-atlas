use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ManifestStats,
};
use bijux_atlas_server::{CatalogFetch, DatasetStoreBackend, RetryPolicy, S3LikeBackend};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct ServerState {
    catalog_calls: AtomicUsize,
    sqlite_calls: AtomicUsize,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            catalog_calls: AtomicUsize::new(0),
            sqlite_calls: AtomicUsize::new(0),
        }
    }
}

#[tokio::test]
async fn s3_like_backend_supports_retry_and_resume_download() {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let sqlite_bytes = b"0123456789abcdef".to_vec();
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            sha256_hex(&sqlite_bytes),
        ),
        ManifestStats::new(1, 1, 1),
    );
    let catalog = Catalog::new(vec![CatalogEntry::new(
        ds.clone(),
        "x".to_string(),
        "y".to_string(),
    )]);

    let state = Arc::new(ServerState::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("addr");

    let state_bg = Arc::clone(&state);
    let sqlite_bg = sqlite_bytes.clone();
    let manifest_json = serde_json::to_vec(&manifest).expect("manifest json");
    let catalog_json = serde_json::to_vec(&catalog).expect("catalog json");
    tokio::spawn(async move {
        loop {
            let (mut stream, _) = match listener.accept().await {
                Ok(v) => v,
                Err(_) => break,
            };
            let mut req = vec![0u8; 8192];
            let n = stream.read(&mut req).await.expect("read req");
            let req_text = String::from_utf8_lossy(&req[..n]);
            let mut lines = req_text.lines();
            let first = lines.next().unwrap_or_default().to_string();
            let path = first
                .split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .to_string();
            let range = req_text
                .lines()
                .find(|l| l.to_ascii_lowercase().starts_with("range:"))
                .map(|l| l.replace("Range: ", "").replace("range: ", ""));

            if path == "/catalog.json" {
                let calls = state_bg.catalog_calls.fetch_add(1, Ordering::Relaxed) + 1;
                if calls == 1 {
                    let _ = stream
                        .write_all(
                            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n",
                        )
                        .await;
                    continue;
                }
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                    catalog_json.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(&catalog_json).await;
                continue;
            }
            if path.ends_with("/manifest.json") {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                    manifest_json.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(&manifest_json).await;
                continue;
            }
            if path.ends_with("/gene_summary.sqlite") {
                let calls = state_bg.sqlite_calls.fetch_add(1, Ordering::Relaxed) + 1;
                if calls == 1 {
                    let first_chunk = &sqlite_bg[..8];
                    let header = format!(
                        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-7/16\r\nContent-Length: {}\r\n\r\n",
                        first_chunk.len()
                    );
                    let _ = stream.write_all(header.as_bytes()).await;
                    let _ = stream.write_all(first_chunk).await;
                    continue;
                }
                if let Some(r) = range {
                    if r.trim() == "bytes=8-" {
                        let rest = &sqlite_bg[8..];
                        let header = format!(
                            "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 8-15/16\r\nContent-Length: {}\r\n\r\n",
                            rest.len()
                        );
                        let _ = stream.write_all(header.as_bytes()).await;
                        let _ = stream.write_all(rest).await;
                        continue;
                    }
                }
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                    sqlite_bg.len()
                );
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(&sqlite_bg).await;
                continue;
            }
            let _ = stream
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await;
        }
    });

    let backend = S3LikeBackend::new(
        format!("http://{addr}"),
        None,
        RetryPolicy {
            max_attempts: 4,
            base_backoff_ms: 10,
        },
    );

    let fetch = backend
        .fetch_catalog(None)
        .await
        .expect("fetch catalog with retry");
    match fetch {
        CatalogFetch::Updated { catalog, .. } => assert_eq!(catalog.datasets.len(), 1),
        CatalogFetch::NotModified => panic!("expected updated catalog"),
    }

    let m = backend.fetch_manifest(&ds).await.expect("manifest");
    assert_eq!(m.dataset, ds);

    let sqlite = backend
        .fetch_sqlite_bytes(&m.dataset)
        .await
        .expect("sqlite bytes via resume");
    assert_eq!(sqlite, sqlite_bytes);
}
