// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::ShardCatalog;
use bijux_atlas_query::{
    explain_query_plan, query_genes, query_genes_fanout, query_gene_id_name_json_minimal_fast,
    select_shards_for_request, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits, RegionFilter,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;
use std::thread;

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
        CREATE INDEX idx_gene_summary_cover_lookup ON gene_summary(gene_id, name, seqid, start, end);
        CREATE INDEX idx_gene_summary_cover_region ON gene_summary(seqid, start, end, gene_id);
        ",
    )
    .expect("schema");

    for i in 1..=20_000_i64 {
        let seqid = match i % 4 {
            0 => "chr1",
            1 => "chr2",
            2 => "1",
            _ => "X",
        };
        let start = i * 10;
        let end = start + 50;
        let biotype = if i % 7 == 0 {
            "lncRNA"
        } else {
            "protein_coding"
        };
        conn.execute(
            "INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,1,?9)",
            rusqlite::params![
                i,
                format!("gene{i}"),
                format!("GENE{i}"),
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

fn setup_db_file() -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("tmp");
    let conn = Connection::open(file.path()).expect("open");
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
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
        ",
    )
    .expect("schema");
    for i in 1..=10_000_i64 {
        let seqid = if i % 2 == 0 { "chr1" } else { "chr2" };
        let start = i * 20;
        let end = start + 25;
        conn.execute(
            "INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length)
             VALUES(?1,?2,?3,?4,'protein_coding',?5,?6,?7,1,?8)",
            rusqlite::params![
                i,
                format!("gene{i}"),
                format!("GENE{i}"),
                format!("gene{i}"),
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
    file
}

fn gene_id_request(id: &str) -> GeneQueryRequest {
    GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: Some(id.to_string()),
            ..Default::default()
        },
        limit: 1,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    }
}

fn region_request(start: u64, end: u64) -> GeneQueryRequest {
    GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start,
                end,
            }),
            ..Default::default()
        },
        limit: 200,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    }
}

fn shard_catalog_fixture() -> ShardCatalog {
    serde_json::from_value(serde_json::json!({
        "model_version": "v1",
        "dataset": {"species":"homo_sapiens","assembly":"GRCh38","release":"110"},
        "mode": "seqid",
        "shards": [
            {
                "shard_id": "s1",
                "seqids": ["chr1", "1"],
                "sqlite_path": "shards/chr1.sqlite",
                "sqlite_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            },
            {
                "shard_id": "s2",
                "seqids": ["chr2"],
                "sqlite_path": "shards/chr2.sqlite",
                "sqlite_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            }
        ]
    }))
    .expect("parse shard catalog")
}

fn bench_query_routing_and_index(c: &mut Criterion) {
    let conn = setup_db();
    let limits = QueryLimits::default();

    c.bench_function("query_shard_routing", |b| {
        let req = region_request(1, 500_000);
        let catalog = shard_catalog_fixture();
        b.iter(|| {
            let selected = select_shards_for_request(black_box(&req), black_box(&catalog));
            black_box(selected);
        })
    });

    c.bench_function("query_distributed_query_simulation", |b| {
        let req = region_request(1, 500_000);
        let conn2 = setup_db();
        b.iter(|| {
            let response = query_genes_fanout(
                black_box(&[&conn, &conn2]),
                black_box(&req),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("fanout");
            black_box(response.rows.len());
        })
    });

    c.bench_function("query_planner_complexity", |b| {
        let req = GeneQueryRequest {
            fields: GeneFields::default(),
            filter: GeneFilter {
                biotype: Some("protein_coding".to_string()),
                name_prefix: Some("GENE".to_string()),
                region: Some(RegionFilter {
                    seqid: "chr1".to_string(),
                    start: 1,
                    end: 4_000_000,
                }),
                ..Default::default()
            },
            limit: 200,
            cursor: None,
            dataset_key: None,
            allow_full_scan: false,
        };
        b.iter(|| {
            let response = query_genes(
                black_box(&conn),
                black_box(&req),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("complex query");
            black_box(response.rows.len());
        })
    });

    c.bench_function("query_index_performance", |b| {
        let req = gene_id_request("gene1234");
        b.iter(|| {
            let response = query_genes(
                black_box(&conn),
                black_box(&req),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("indexed query");
            black_box(response.rows.len());
        })
    });

    c.bench_function("query_sqlite_index_scan", |b| {
        let req = gene_id_request("gene1234");
        b.iter(|| {
            let plan = explain_query_plan(
                black_box(&conn),
                black_box(&req),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("plan");
            black_box(plan);
        })
    });

    c.bench_function("query_covering_index", |b| {
        b.iter(|| {
            let payload =
                query_gene_id_name_json_minimal_fast(black_box(&conn), "gene1234").expect("json");
            black_box(payload);
        })
    });

    c.bench_function("query_region_overlap", |b| {
        let req = region_request(10_000, 250_000);
        b.iter(|| {
            let response = query_genes(
                black_box(&conn),
                black_box(&req),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("region overlap");
            black_box(response.rows.len());
        })
    });

    c.bench_function("query_cursor_pagination", |b| {
        let first = GeneQueryRequest {
            limit: 25,
            ..region_request(1, 250_000)
        };
        let first_page =
            query_genes(&conn, &first, &limits, b"bench-secret").expect("first page query");
        let second = GeneQueryRequest {
            cursor: first_page.next_cursor,
            ..first
        };
        b.iter(|| {
            let response = query_genes(
                black_box(&conn),
                black_box(&second),
                black_box(&limits),
                b"bench-secret",
            )
            .expect("second page query");
            black_box(response.rows.len());
        })
    });

    let db_file = setup_db_file();
    let db_path = Arc::new(db_file.path().to_path_buf());
    c.bench_function("query_concurrency_benchmark", |b| {
        b.iter(|| {
            let mut joins = Vec::new();
            for _ in 0..8 {
                let path = Arc::clone(&db_path);
                joins.push(thread::spawn(move || {
                    let conn = Connection::open_with_flags(
                        path.as_path(),
                        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
                    )
                    .expect("open");
                    let req = gene_id_request("gene42");
                    let response =
                        query_genes(&conn, &req, &QueryLimits::default(), b"bench-secret")
                            .expect("concurrent query");
                    response.rows.len()
                }));
            }
            for join in joins {
                black_box(join.join().expect("join"));
            }
        })
    });

    c.bench_function("query_multi_thread_scaling", |b| {
        b.iter(|| {
            let mut joins = Vec::new();
            for _ in 0..16 {
                let path = Arc::clone(&db_path);
                joins.push(thread::spawn(move || {
                    let conn = Connection::open_with_flags(
                        path.as_path(),
                        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
                    )
                    .expect("open");
                    let req = region_request(1, 150_000);
                    let response =
                        query_genes(&conn, &req, &QueryLimits::default(), b"bench-secret")
                            .expect("concurrent region query");
                    response.rows.len()
                }));
            }
            for join in joins {
                black_box(join.join().expect("join"));
            }
        })
    });
}

criterion_group!(benches, bench_query_routing_and_index);
criterion_main!(benches);
