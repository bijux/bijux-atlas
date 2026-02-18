use std::sync::Arc;
use std::time::{Duration, Instant};

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use tempfile::tempdir;

fn fixture_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("bench.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
         CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,100,1,100);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}

fn start_server() -> (tokio::runtime::Runtime, String) {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let addr = rt.block_on(async {
        let sqlite = fixture_sqlite();
        let fasta = b">chr1\nACGTACGTACGT\n".to_vec();
        let fai = b"chr1\t12\t6\t12\t13\n".to_vec();
        let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
        let manifest = ArtifactManifest::new(
            "1".to_string(),
            "1".to_string(),
            ds.clone(),
            ArtifactChecksums::new(
                "a".repeat(64),
                sha256_hex(&fasta),
                sha256_hex(&fai),
                sha256_hex(&sqlite),
            ),
            ManifestStats::new(1, 1, 1),
        );
        let store = Arc::new(FakeStore::default());
        store.manifest.lock().await.insert(ds.clone(), manifest);
        store.sqlite.lock().await.insert(ds, sqlite);
        store.fasta.lock().await.insert(
            DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id"),
            fasta,
        );
        store.fai.lock().await.insert(
            DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id"),
            fai,
        );
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
            .expect("bind");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });
        addr
    });
    (rt, format!("http://{addr}"))
}

fn burst_p99(url: &str, threads: usize, per_thread: usize) -> Duration {
    let samples = Arc::new(std::sync::Mutex::new(Vec::<Duration>::new()));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            let samples = Arc::clone(&samples);
            let url = url.to_string();
            scope.spawn(move || {
                let client = reqwest::blocking::Client::new();
                for _ in 0..per_thread {
                    let started = Instant::now();
                    let resp = client.get(&url).send().expect("request");
                    assert_eq!(resp.status().as_u16(), 200);
                    samples
                        .lock()
                        .expect("lock samples")
                        .push(started.elapsed());
                }
            });
        }
    });
    let mut durations = samples.lock().expect("lock").clone();
    durations.sort_unstable();
    let idx = (((durations.len() as f64) * 0.99).ceil() as usize).saturating_sub(1);
    durations[idx.min(durations.len().saturating_sub(1))]
}

fn bench_gene_lookup_p99_under_load(c: &mut Criterion) {
    let (_rt, base) = start_server();
    let url = format!(
        "{base}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1"
    );

    c.bench_function("gene_lookup_p99_under_load", |b| {
        b.iter(|| {
            let p99 = burst_p99(&url, 8, 20);
            black_box(p99);
        })
    });
}

criterion_group!(benches, bench_gene_lookup_p99_under_load);
criterion_main!(benches);
