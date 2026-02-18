use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, Catalog, CatalogEntry, DatasetId, ManifestStats,
};
use bijux_atlas_server::{
    CatalogFetch, DatasetStoreBackend, LocalFsBackend, RetryPolicy, S3LikeBackend,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct ServerState {
    catalog_calls: AtomicUsize,
    sqlite_calls: AtomicUsize,
    if_none_match_seen: AtomicUsize,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            catalog_calls: AtomicUsize::new(0),
            sqlite_calls: AtomicUsize::new(0),
            if_none_match_seen: AtomicUsize::new(0),
        }
    }
}

#[tokio::test]
async fn s3_like_backend_supports_retry_and_resume_download() {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let sqlite_bytes = b"0123456789abcdef".to_vec();
    let mut manifest = ArtifactManifest::new(
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
    manifest.db_hash = sha256_hex(&sqlite_bytes);
    manifest.artifact_hash = manifest.db_hash.clone();
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
            let if_none_match = req_text
                .lines()
                .find(|l| l.to_ascii_lowercase().starts_with("if-none-match:"))
                .map(|l| {
                    l.replace("If-None-Match: ", "")
                        .replace("if-none-match: ", "")
                });

            if path == "/catalog.json" {
                let calls = state_bg.catalog_calls.fetch_add(1, Ordering::Relaxed) + 1;
                if if_none_match.is_some() {
                    state_bg.if_none_match_seen.fetch_add(1, Ordering::Relaxed);
                }
                if calls == 1 {
                    let _ = stream
                        .write_all(
                            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n",
                        )
                        .await;
                    continue;
                }
                if let Some(etag) = if_none_match {
                    if etag.trim() == "\"catalog-etag\"" {
                        let _ = stream
                            .write_all(
                                b"HTTP/1.1 304 Not Modified\r\nETag: \"catalog-etag\"\r\nContent-Length: 0\r\n\r\n",
                            )
                            .await;
                        continue;
                    }
                }
                let header = format!(
                    "HTTP/1.1 200 OK\r\nETag: \"catalog-etag\"\r\nContent-Length: {}\r\n\r\n",
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
        None,
        RetryPolicy {
            max_attempts: 4,
            base_backoff_ms: 10,
        },
        true,
    );

    let fetch = backend
        .fetch_catalog(None)
        .await
        .expect("fetch catalog with retry");
    let etag = match &fetch {
        CatalogFetch::Updated { etag, .. } => etag.clone(),
        CatalogFetch::NotModified => String::new(),
        _ => String::new(),
    };
    match fetch {
        CatalogFetch::Updated { catalog, .. } => assert_eq!(catalog.datasets.len(), 1),
        CatalogFetch::NotModified => panic!("expected updated catalog"),
        _ => panic!("unexpected catalog fetch variant"),
    }
    let second = backend
        .fetch_catalog(Some(&etag))
        .await
        .expect("fetch catalog with if-none-match");
    match second {
        CatalogFetch::NotModified => {}
        CatalogFetch::Updated { .. } => panic!("expected not-modified response"),
        _ => panic!("unexpected catalog fetch variant"),
    }
    assert!(
        state.if_none_match_seen.load(Ordering::Relaxed) >= 1,
        "server should observe if-none-match request header"
    );

    let m = backend.fetch_manifest(&ds).await.expect("manifest");
    assert_eq!(m.dataset, ds);

    let sqlite = backend
        .fetch_sqlite_bytes(&m.dataset)
        .await
        .expect("sqlite bytes via resume");
    assert_eq!(sqlite, sqlite_bytes);
}

#[tokio::test]
async fn s3_like_backend_blocks_private_hosts_by_default() {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let backend = S3LikeBackend::new(
        "http://127.0.0.1:1".to_string(),
        None,
        None,
        RetryPolicy {
            max_attempts: 1,
            base_backoff_ms: 1,
        },
        false,
    );
    let err = backend
        .fetch_manifest(&ds)
        .await
        .expect_err("private host should be blocked");
    assert!(err.to_string().contains("blocked private store host"));
}

#[tokio::test]
async fn s3_like_backend_reports_404_for_missing_manifest() {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut req = vec![0u8; 4096];
            let _ = stream.read(&mut req).await;
            let _ = stream
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await;
        }
    });
    let backend = S3LikeBackend::new(
        format!("http://{addr}"),
        None,
        None,
        RetryPolicy {
            max_attempts: 1,
            base_backoff_ms: 1,
        },
        true,
    );
    let err = backend
        .fetch_manifest(&ds)
        .await
        .expect_err("missing manifest should fail");
    assert!(
        err.to_string().contains("404"),
        "error must include stable http status detail: {err}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn local_fs_backend_blocks_symlink_path_traversal() {
    use std::os::unix::fs::symlink;

    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let tmp = tempfile::tempdir().expect("tmp");
    let outside = tempfile::tempdir().expect("outside");
    let paths = bijux_atlas_model::artifact_paths(tmp.path(), &ds);
    std::fs::create_dir_all(paths.dataset_root.parent().expect("dataset parent"))
        .expect("create parent");
    symlink(outside.path(), &paths.dataset_root).expect("symlink dataset root");
    std::fs::create_dir_all(paths.derived_dir.clone()).expect("create derived under symlink");
    std::fs::write(&paths.manifest, b"{}").expect("write manifest");

    let backend = LocalFsBackend::new(tmp.path().to_path_buf());
    let err = backend
        .fetch_manifest(&ds)
        .await
        .expect_err("must block traversal via symlink");
    assert!(err.to_string().contains("path traversal blocked"));
}
