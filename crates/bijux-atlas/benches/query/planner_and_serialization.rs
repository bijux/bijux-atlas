// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::query::{
    normalized_query_hash_ssot, parse_gene_query_request, plan_gene_query, query_genes, GeneFields,
    GeneFilter, GeneQueryRequest, QueryLimits, RegionFilter,
};
use criterion::{criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use std::hint::black_box;

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

    for i in 1..=5_000_i64 {
        let seqid = if i % 2 == 0 { "chr1" } else { "chr2" };
        let start = i * 25;
        let end = start + 100;
        let biotype = if i % 5 == 0 {
            "lncRNA"
        } else {
            "protein_coding"
        };
        let name = format!("GENE{i}");
        conn.execute(
            "INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,1,?9)",
            rusqlite::params![
                i,
                format!("gene{i}"),
                name,
                format!("gene{i}"),
                biotype,
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

fn sample_request() -> GeneQueryRequest {
    GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("GENE".to_string()),
            biotype: Some("protein_coding".to_string()),
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start: 1,
                end: 100_000,
            }),
            ..Default::default()
        },
        limit: 100,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    }
}

fn bench_query_planner_and_serialization(c: &mut Criterion) {
    let conn = setup_db();
    let req = sample_request();
    let limits = QueryLimits::default();

    c.bench_function("query_planner_latency", |b| {
        let ast = parse_gene_query_request(&req).expect("parse");
        b.iter(|| plan_gene_query(black_box(&ast), black_box(&limits)).expect("plan"))
    });

    c.bench_function("query_ast_normalization", |b| {
        b.iter(|| normalized_query_hash_ssot(black_box(&req)).expect("normalize"))
    });

    c.bench_function("query_cursor_generation", |b| {
        let request = GeneQueryRequest {
            limit: 25,
            ..req.clone()
        };
        b.iter(|| {
            let response = query_genes(
                black_box(&conn),
                black_box(&request),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("execute");
            black_box(response.next_cursor.clone());
        })
    });

    c.bench_function("query_response_serialization", |b| {
        b.iter(|| {
            let response = query_genes(
                black_box(&conn),
                black_box(&req),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("execute");
            let json = serde_json::to_vec(&response).expect("serialize");
            black_box(json);
        })
    });

    c.bench_function("query_json_encoding", |b| {
        let response = query_genes(&conn, &req, &limits, b"bench-secret").expect("execute");
        b.iter(|| {
            let json = serde_json::to_string(black_box(&response)).expect("json string");
            black_box(json);
        })
    });
}

criterion_group!(benches, bench_query_planner_and_serialization);
criterion_main!(benches);
