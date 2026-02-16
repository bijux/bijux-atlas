use bijux_atlas_query::{
    query_genes, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits, RegionFilter,
};
use criterion::{criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use std::time::{Duration, Instant};

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

    for i in 1..=20_000_i64 {
        let seqid = match i % 4 {
            0 => "chr1",
            1 => "chr2",
            2 => "1",
            _ => "X",
        };
        let name_prefix = if i % 3 == 0 {
            "BRCA"
        } else if i % 7 == 0 {
            "GENE_"
        } else {
            "GENE"
        };
        let biotype = if i % 5 == 0 {
            "lncRNA"
        } else {
            "protein_coding"
        };
        let start = i * 10;
        let end = start + 50;
        conn.execute(
            "INSERT INTO gene_summary (id, gene_id, name, name_normalized, biotype, seqid, start, end, transcript_count, sequence_length)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                i,
                format!("gene{i}"),
                format!("{name_prefix}{i}"),
                format!("{name_prefix}{i}").to_ascii_lowercase(),
                biotype,
                seqid,
                start,
                end,
                (i % 4) + 1,
                end - start + 1
            ],
        )
        .expect("insert");
        conn.execute(
            "INSERT INTO gene_summary_rtree (gene_rowid, start, end) VALUES (?1, ?2, ?3)",
            rusqlite::params![i, start as f64, end as f64],
        )
        .expect("insert rtree");
    }
    conn.execute_batch(
        "
        INSERT INTO dataset_stats (dimension, value, gene_count)
        SELECT 'biotype', biotype, COUNT(*) FROM gene_summary GROUP BY biotype;
        INSERT INTO dataset_stats (dimension, value, gene_count)
        SELECT 'seqid', seqid, COUNT(*) FROM gene_summary GROUP BY seqid;
        ",
    )
    .expect("dataset stats");
    conn
}

fn run_pattern(conn: &Connection, req: &GeneQueryRequest) {
    let _ = query_genes(conn, req, &QueryLimits::default(), b"bench-secret").expect("query");
}

fn req(filter: GeneFilter, limit: usize, fields: GeneFields) -> GeneQueryRequest {
    GeneQueryRequest {
        fields,
        filter,
        limit,
        cursor: None,
        allow_full_scan: false,
    }
}

fn maybe_enforce_baseline(conn: &Connection) {
    if std::env::var("ATLAS_QUERY_BENCH_ENFORCE").as_deref() != Ok("1") {
        return;
    }

    let baseline_cases: [(&str, GeneQueryRequest, Duration); 3] = [
        (
            "gene_id_exact",
            req(
                GeneFilter {
                    gene_id: Some("gene1234".to_string()),
                    ..Default::default()
                },
                1,
                GeneFields::default(),
            ),
            Duration::from_millis(20),
        ),
        (
            "name_prefix",
            req(
                GeneFilter {
                    name_prefix: Some("BR".to_string()),
                    ..Default::default()
                },
                100,
                GeneFields::default(),
            ),
            Duration::from_millis(40),
        ),
        (
            "region_window",
            req(
                GeneFilter {
                    region: Some(RegionFilter {
                        seqid: "chr1".to_string(),
                        start: 10_000,
                        end: 200_000,
                    }),
                    ..Default::default()
                },
                100,
                GeneFields::default(),
            ),
            Duration::from_millis(50),
        ),
    ];

    for (name, request, max) in baseline_cases {
        let started = Instant::now();
        run_pattern(conn, &request);
        let elapsed = started.elapsed();
        assert!(
            elapsed <= max,
            "baseline regression for {name}: elapsed={elapsed:?}, max={max:?}"
        );
    }
}

fn bench_query_patterns(c: &mut Criterion) {
    let conn = setup_db();
    maybe_enforce_baseline(&conn);

    c.bench_function("query_gene_id_exact", |b| {
        let request = req(
            GeneFilter {
                gene_id: Some("gene1234".to_string()),
                ..Default::default()
            },
            1,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_name_exact", |b| {
        let request = req(
            GeneFilter {
                name: Some("BRCA3000".to_string()),
                ..Default::default()
            },
            20,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_name_prefix_short", |b| {
        let request = req(
            GeneFilter {
                name_prefix: Some("BR".to_string()),
                ..Default::default()
            },
            50,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_name_prefix_underscore", |b| {
        let request = req(
            GeneFilter {
                name_prefix: Some("GENE_".to_string()),
                ..Default::default()
            },
            50,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_biotype", |b| {
        let request = req(
            GeneFilter {
                biotype: Some("protein_coding".to_string()),
                ..Default::default()
            },
            100,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_region_small", |b| {
        let request = req(
            GeneFilter {
                region: Some(RegionFilter {
                    seqid: "chr1".to_string(),
                    start: 10_000,
                    end: 40_000,
                }),
                ..Default::default()
            },
            100,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_region_large", |b| {
        let request = req(
            GeneFilter {
                region: Some(RegionFilter {
                    seqid: "chr1".to_string(),
                    start: 10_000,
                    end: 200_000,
                }),
                ..Default::default()
            },
            100,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_projection_minimal", |b| {
        let request = req(
            GeneFilter {
                biotype: Some("lncRNA".to_string()),
                ..Default::default()
            },
            100,
            GeneFields {
                gene_id: true,
                name: false,
                coords: false,
                biotype: false,
                transcript_count: false,
                sequence_length: false,
            },
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_projection_full", |b| {
        let request = req(
            GeneFilter {
                biotype: Some("lncRNA".to_string()),
                ..Default::default()
            },
            100,
            GeneFields::default(),
        );
        b.iter(|| run_pattern(&conn, &request));
    });

    c.bench_function("query_pagination_second_page", |b| {
        let first = req(
            GeneFilter {
                biotype: Some("protein_coding".to_string()),
                ..Default::default()
            },
            25,
            GeneFields::default(),
        );
        let first_resp = query_genes(&conn, &first, &QueryLimits::default(), b"bench-secret")
            .expect("first page");
        let second = GeneQueryRequest {
            cursor: first_resp.next_cursor,
            ..first
        };
        b.iter(|| run_pattern(&conn, &second));
    });
}

criterion_group!(benches, bench_query_patterns);
criterion_main!(benches);
