use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{DatasetCacheConfig, DatasetCacheManager, FakeStore};
use criterion::{criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use tempfile::tempdir;
use tokio::runtime::Runtime;

fn sqlite_bytes() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("bench.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
         CREATE TABLE dataset_stats(dimension TEXT NOT NULL, value TEXT NOT NULL, gene_count INTEGER NOT NULL, PRIMARY KEY (dimension, value));
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10,1,10);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('biotype','pc',1);
         INSERT INTO dataset_stats(dimension,value,gene_count) VALUES ('seqid','chr1',1);",
    )
    .expect("seed");
    std::fs::read(db).expect("read")
}

fn make_manifest(ds: &DatasetId, sqlite: &[u8]) -> ArtifactManifest {
    ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        ds.clone(),
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            sha256_hex(sqlite),
        ),
        ManifestStats::new(1, 1, 1),
    )
}

fn bench_cache_manager(c: &mut Criterion) {
    let rt = Runtime::new().expect("runtime");
    let tmp = tempdir().expect("tempdir");
    let sqlite = sqlite_bytes();
    let store = Arc::new(FakeStore::default());
    let datasets = (1..=16)
        .map(|i| DatasetId::new(&format!("{i}"), "homo_sapiens", "GRCh38").expect("dataset"))
        .collect::<Vec<_>>();
    rt.block_on(async {
        for ds in &datasets {
            store
                .manifest
                .lock()
                .await
                .insert(ds.clone(), make_manifest(ds, &sqlite));
            store.sqlite.lock().await.insert(ds.clone(), sqlite.clone());
        }
    });
    let mgr = DatasetCacheManager::new(
        DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            max_dataset_count: 8,
            ..Default::default()
        },
        store,
    );
    rt.block_on(async {
        for ds in datasets.iter().take(4) {
            mgr.open_dataset_connection(ds).await.expect("warm");
        }
    });

    c.bench_function("cache_manager_random_access_16", |b| {
        let mut idx = 0usize;
        b.iter(|| {
            let ds = datasets[idx % datasets.len()].clone();
            idx += 5;
            rt.block_on(async {
                let conn = mgr.open_dataset_connection(&ds).await.expect("open");
                let _: i64 = conn
                    .conn
                    .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0))
                    .expect("count");
            });
        });
    });
}

criterion_group!(benches, bench_cache_manager);
criterion_main!(benches);
