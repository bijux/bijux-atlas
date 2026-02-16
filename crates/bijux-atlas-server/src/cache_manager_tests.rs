use super::*;
use tempfile::tempdir;

fn fixture_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("x.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
         INSERT INTO gene_summary(id,gene_id,name,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','pc','chr1',1,10,1,10);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}

fn mk_dataset() -> (DatasetId, ArtifactManifest, Vec<u8>) {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let sqlite = fixture_sqlite();
    let sqlite_sha = sha256_hex(&sqlite);
    let manifest = ArtifactManifest {
        manifest_version: "1".to_string(),
        db_schema_version: "1".to_string(),
        dataset: ds.clone(),
        checksums: bijux_atlas_model::ArtifactChecksums {
            gff3_sha256: "a".repeat(64),
            fasta_sha256: "b".repeat(64),
            fai_sha256: "c".repeat(64),
            sqlite_sha256: sqlite_sha,
        },
        stats: bijux_atlas_model::ManifestStats {
            gene_count: 1,
            transcript_count: 1,
            contig_count: 1,
        },
    };
    (ds, manifest, sqlite)
}

#[tokio::test]
async fn single_flight_download_shared_by_concurrent_calls() {
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
    let mgr = DatasetCacheManager::new(cfg, store.clone());

    let mut joins = Vec::new();
    for _ in 0..8 {
        let m = Arc::clone(&mgr);
        let d = ds.clone();
        joins.push(tokio::spawn(
            async move { m.open_dataset_connection(&d).await },
        ));
    }
    for j in joins {
        j.await.expect("join handle").expect("open connection");
    }

    let calls = store.fetch_calls.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(calls, 1, "single-flight should perform one manifest fetch");
}

#[tokio::test]
#[ignore]
async fn chaos_mode_slow_store_reads_graceful_errors() {
    let (ds, manifest, sqlite) = mk_dataset();
    let store = Arc::new(FakeStore {
        slow_read: true,
        ..Default::default()
    });
    store.manifest.lock().await.insert(ds.clone(), manifest);
    store.sqlite.lock().await.insert(ds.clone(), sqlite);

    let tmp = tempdir().expect("tempdir");
    let cfg = DatasetCacheConfig {
        disk_root: tmp.path().to_path_buf(),
        dataset_open_timeout: Duration::from_millis(1),
        ..Default::default()
    };
    let mgr = DatasetCacheManager::new(cfg, store);
    let result = mgr.open_dataset_connection(&ds).await;
    match result {
        Ok(_) => {}
        Err(err) => {
            assert!(err.to_string().contains("timeout") || err.to_string().contains("missing"))
        }
    }
}
