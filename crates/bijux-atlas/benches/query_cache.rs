// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_query::{
    normalized_query_hash_ssot, query_genes, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open memory db");
    conn.execute_batch(
        "
        CREATE TABLE gene_summary (
          id INTEGER PRIMARY KEY,
          gene_id TEXT NOT NULL,
          name TEXT NOT NULL,
          name_normalized TEXT NOT NULL,
          biotype TEXT NOT NULL,
          seqid TEXT NOT NULL,
          start INTEGER NOT NULL,
          end INTEGER NOT NULL,
          transcript_count INTEGER NOT NULL,
          exon_count INTEGER NOT NULL DEFAULT 0,
          total_exon_span INTEGER NOT NULL DEFAULT 0,
          cds_present INTEGER NOT NULL DEFAULT 0,
          sequence_length INTEGER NOT NULL
        );
        CREATE TABLE dataset_stats (
          dimension TEXT NOT NULL,
          value TEXT NOT NULL,
          gene_count INTEGER NOT NULL,
          PRIMARY KEY (dimension, value)
        );
        CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(gene_rowid, start, end);
        CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
        CREATE INDEX idx_gene_summary_name_normalized ON gene_summary(name_normalized);
        CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
        ",
    )
    .expect("schema");

    for i in 1..=10_000_i64 {
        let seqid = if i % 2 == 0 { "chr1" } else { "chr2" };
        let start = i * 10;
        let end = start + 25;
        conn.execute(
            "INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,1,?9)",
            rusqlite::params![
                i,
                format!("gene{i}"),
                format!("GENE{i}"),
                format!("gene{i}"),
                "protein_coding",
                seqid,
                start,
                end,
                end - start + 1
            ],
        )
        .expect("insert");
        conn.execute(
            "INSERT INTO gene_summary_rtree(gene_rowid,start,end) VALUES(?1,?2,?3)",
            rusqlite::params![i, start as f64, end as f64],
        )
        .expect("rtree");
    }
    conn.execute_batch(
        "
        INSERT INTO dataset_stats (dimension, value, gene_count)
        SELECT 'biotype', biotype, COUNT(*) FROM gene_summary GROUP BY biotype;
        INSERT INTO dataset_stats (dimension, value, gene_count)
        SELECT 'seqid', seqid, COUNT(*) FROM gene_summary GROUP BY seqid;
        ",
    )
    .expect("stats");
    conn
}

fn request_for_gene(gene_id: String) -> GeneQueryRequest {
    GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: Some(gene_id),
            ..Default::default()
        },
        limit: 1,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    }
}

fn bench_query_cache(c: &mut Criterion) {
    let conn = setup_db();
    let limits = QueryLimits::default();

    c.bench_function("query_cache_performance", |b| {
        let mut cache: HashMap<String, Vec<u8>> = HashMap::new();
        let req = request_for_gene("gene42".to_string());
        b.iter(|| {
            let key = normalized_query_hash_ssot(&req).expect("hash");
            if let Some(payload) = cache.get(&key) {
                black_box(payload);
                return;
            }
            let response =
                query_genes(&conn, &req, &limits, b"bench-secret").expect("query from db");
            let encoded = serde_json::to_vec(&response).expect("serialize");
            cache.insert(key, encoded);
        });
    });

    c.bench_function("query_cache_hit_latency", |b| {
        let req = request_for_gene("gene42".to_string());
        let key = normalized_query_hash_ssot(&req).expect("hash");
        let response = query_genes(&conn, &req, &limits, b"bench-secret").expect("warm query");
        let mut cache: HashMap<String, Vec<u8>> = HashMap::new();
        cache.insert(
            key.clone(),
            serde_json::to_vec(&response).expect("serialize"),
        );

        b.iter(|| {
            let payload = cache.get(black_box(&key)).expect("cache hit");
            black_box(payload);
        });
    });

    c.bench_function("query_cache_miss_latency", |b| {
        let mut cache: HashMap<String, Vec<u8>> = HashMap::new();
        let counter = Cell::new(100_i64);
        b.iter(|| {
            let next = counter.get();
            counter.set(next + 1);
            let req = request_for_gene(format!("gene{next}"));
            let key = normalized_query_hash_ssot(&req).expect("hash");
            let response =
                query_genes(&conn, black_box(&req), &limits, b"bench-secret").expect("query");
            let encoded = serde_json::to_vec(&response).expect("serialize");
            cache.insert(key, encoded);
        });
    });

    c.bench_function("query_cache_eviction", |b| {
        let capacity = 64_usize;
        let mut cache: HashMap<String, Vec<u8>> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let counter = Cell::new(1_i64);
        b.iter(|| {
            let idx = counter.get();
            counter.set(idx + 1);
            let req = request_for_gene(format!("gene{}", (idx % 512) + 1));
            let key = normalized_query_hash_ssot(&req).expect("hash");

            if cache.contains_key(&key) {
                black_box(cache.get(&key));
                return;
            }

            let response =
                query_genes(&conn, black_box(&req), &limits, b"bench-secret").expect("query");
            let encoded = serde_json::to_vec(&response).expect("serialize");
            cache.insert(key.clone(), encoded);
            queue.push_back(key);

            while cache.len() > capacity {
                if let Some(oldest) = queue.pop_front() {
                    cache.remove(&oldest);
                }
            }
        });
    });

    c.bench_function("query_cache_warmup", |b| {
        let working_set: Vec<String> = (1..=128).map(|i| format!("gene{i}")).collect();
        b.iter(|| {
            let mut cache: HashMap<String, Vec<u8>> = HashMap::new();
            for gene_id in &working_set {
                let req = request_for_gene(gene_id.clone());
                let key = normalized_query_hash_ssot(&req).expect("hash");
                let response =
                    query_genes(&conn, &req, &limits, b"bench-secret").expect("query warmup");
                cache.insert(key, serde_json::to_vec(&response).expect("serialize"));
            }
            black_box(cache.len());
        });
    });
}

criterion_group!(benches, bench_query_cache);
criterion_main!(benches);
