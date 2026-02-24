// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats};
use bijux_atlas_server::{
    build_router, ApiConfig, AppState, DatasetCacheConfig, DatasetCacheManager, FakeStore,
};
use criterion::{criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use tempfile::tempdir;

fn fixture_sqlite() -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("bench.sqlite");
    let conn = Connection::open(&db).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, name_normalized TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
         INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','g1','pc','chr1',1,10000,1,10000);",
    )
    .expect("seed sqlite");
    std::fs::read(db).expect("read sqlite bytes")
}

fn fixture_fasta_and_fai() -> (Vec<u8>, Vec<u8>) {
    let mut seq = String::new();
    let alphabet = ["A", "C", "G", "T"];
    for i in 0..20_000 {
        seq.push_str(alphabet[i % alphabet.len()]);
        if (i + 1) % 50 == 0 {
            seq.push('\n');
        }
    }
    let fasta = format!(">chr1\n{seq}").into_bytes();
    let fai = b"chr1\t20000\t6\t50\t51\n".to_vec();
    (fasta, fai)
}

fn start_server() -> (tokio::runtime::Runtime, String) {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let addr = rt.block_on(async {
        let (fasta, fai) = fixture_fasta_and_fai();
        let sqlite = fixture_sqlite();
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
            max_sequence_bases: 20_000,
            sequence_api_key_required_bases: 100,
            ..ApiConfig::default()
        };
        let app = build_router(AppState::with_config(mgr, api, Default::default()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind listener");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });
        addr
    });
    (rt, format!("http://{addr}"))
}

fn bench_sequence_fetch(c: &mut Criterion) {
    let (_rt, base) = start_server();
    let client = reqwest::blocking::Client::new();
    let dataset = "release=110&species=homo_sapiens&assembly=GRCh38";

    let mut group = c.benchmark_group("sequence_fetch");
    for (label, span) in [("100bp", "1-100"), ("1kb", "1-1000"), ("10kb", "1-10000")] {
        let url = format!("{base}/v1/sequence/region?{dataset}&region=chr1:{span}&include_stats=1");
        group.bench_function(label, |b| {
            b.iter(|| {
                let resp = client
                    .get(&url)
                    .header("x-api-key", "bench-key")
                    .send()
                    .expect("sequence response");
                assert_eq!(resp.status().as_u16(), 200);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_sequence_fetch);
criterion_main!(benches);
