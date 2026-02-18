use bijux_atlas_query::{query_genes, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits};
use criterion::{criterion_group, criterion_main, Criterion};
use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;
use std::thread;

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
        CREATE INDEX idx_gene_summary_name ON gene_summary(name);
        CREATE INDEX idx_gene_summary_name_normalized ON gene_summary(name_normalized);
        CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
        ",
    )
    .expect("schema");

    for i in 1..=10_000_i64 {
        let name = format!("GENE{i}");
        let seqid = if i % 2 == 0 { "chr1" } else { "chr2" };
        let biotype = if i % 5 == 0 {
            "lncRNA"
        } else {
            "protein_coding"
        };
        let start = i * 10;
        let end = start + 20;
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
    file
}

fn bench_query_concurrency(c: &mut Criterion) {
    let db = setup_db_file();
    let db_path = Arc::new(db.path().to_path_buf());

    c.bench_function("query_multi_threaded_concurrency", |b| {
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
                    let req = GeneQueryRequest {
                        fields: GeneFields::default(),
                        filter: GeneFilter {
                            biotype: Some("protein_coding".to_string()),
                            ..Default::default()
                        },
                        limit: 50,
                        cursor: None,
                        dataset_key: None,
                        allow_full_scan: false,
                    };
                    let _ =
                        query_genes(&conn, &req, &QueryLimits::default(), b"bench").expect("query");
                }));
            }
            for j in joins {
                j.join().expect("join");
            }
        });
    });

    c.bench_function("query_point_lookup_concurrency_scaling", |b| {
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
                    let req = GeneQueryRequest {
                        fields: GeneFields::default(),
                        filter: GeneFilter {
                            gene_id: Some("gene42".to_string()),
                            ..Default::default()
                        },
                        limit: 1,
                        cursor: None,
                        dataset_key: None,
                        allow_full_scan: false,
                    };
                    let _ =
                        query_genes(&conn, &req, &QueryLimits::default(), b"bench").expect("query");
                }));
            }
            for j in joins {
                j.join().expect("join");
            }
        });
    });

    c.bench_function("query_region_concurrency_scaling", |b| {
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
                    let req = GeneQueryRequest {
                        fields: GeneFields::default(),
                        filter: GeneFilter {
                            region: Some(bijux_atlas_query::RegionFilter {
                                seqid: "chr1".to_string(),
                                start: 1,
                                end: 150_000,
                            }),
                            ..Default::default()
                        },
                        limit: 100,
                        cursor: None,
                        dataset_key: None,
                        allow_full_scan: false,
                    };
                    let _ =
                        query_genes(&conn, &req, &QueryLimits::default(), b"bench").expect("query");
                }));
            }
            for j in joins {
                j.join().expect("join");
            }
        });
    });
}

criterion_group!(benches, bench_query_concurrency);
criterion_main!(benches);
