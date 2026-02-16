use bijux_atlas_query::{
    query_genes, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits, RegionFilter,
};
use criterion::{criterion_group, criterion_main, Criterion};
use rusqlite::Connection;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open memory db");
    conn.execute_batch(
        "
        CREATE TABLE gene_summary (
          id INTEGER PRIMARY KEY,
          gene_id TEXT NOT NULL,
          name TEXT NOT NULL,
          biotype TEXT NOT NULL,
          seqid TEXT NOT NULL,
          start INTEGER NOT NULL,
          end INTEGER NOT NULL,
          transcript_count INTEGER NOT NULL,
          sequence_length INTEGER NOT NULL
        );
        CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(gene_rowid, start, end);
        CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
        CREATE INDEX idx_gene_summary_name ON gene_summary(name);
        CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
        ",
    )
    .expect("schema");

    for i in 1..=5000_i64 {
        let seqid = if i % 2 == 0 { "chr1" } else { "chr2" };
        let name = if i % 3 == 0 { "BRCA" } else { "GENE" };
        let biotype = if i % 5 == 0 {
            "lncRNA"
        } else {
            "protein_coding"
        };
        let start = i * 10;
        let end = start + 50;
        conn.execute(
            "INSERT INTO gene_summary (id, gene_id, name, biotype, seqid, start, end, transcript_count, sequence_length)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                i,
                format!("gene{i}"),
                format!("{name}{i}"),
                biotype,
                seqid,
                start,
                end,
                1,
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
    conn
}

fn bench_query_patterns(c: &mut Criterion) {
    let conn = setup_db();
    let limits = QueryLimits::default();
    let secret = b"bench-secret";

    c.bench_function("query_biotype_page", |b| {
        b.iter(|| {
            let req = GeneQueryRequest {
                fields: GeneFields::default(),
                filter: GeneFilter {
                    biotype: Some("protein_coding".to_string()),
                    ..Default::default()
                },
                limit: 100,
                cursor: None,
                allow_full_scan: false,
            };
            let _ = query_genes(&conn, &req, &limits, secret).expect("query");
        })
    });

    c.bench_function("query_region_page", |b| {
        b.iter(|| {
            let req = GeneQueryRequest {
                fields: GeneFields::default(),
                filter: GeneFilter {
                    region: Some(RegionFilter {
                        seqid: "chr1".to_string(),
                        start: 100,
                        end: 20000,
                    }),
                    ..Default::default()
                },
                limit: 100,
                cursor: None,
                allow_full_scan: false,
            };
            let _ = query_genes(&conn, &req, &limits, secret).expect("query");
        })
    });
}

criterion_group!(benches, bench_query_patterns);
criterion_main!(benches);
