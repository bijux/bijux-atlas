use bijux_atlas_query::{
    parse_gene_query_request, plan_gene_query, query_genes, GeneFields, GeneFilter, GeneQueryRequest,
    QueryLimits, RegionFilter,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::Connection;

fn sample_request() -> GeneQueryRequest {
    GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("BRCA".to_string()),
            biotype: Some("protein_coding".to_string()),
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start: 1,
                end: 100_000,
            }),
            ..Default::default()
        },
        limit: 50,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    }
}

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open memory");
    conn.execute_batch(
        "CREATE TABLE gene_summary (
            id INTEGER PRIMARY KEY,
            gene_id TEXT NOT NULL,
            name TEXT,
            name_normalized TEXT,
            seqid TEXT,
            start INTEGER,
            end INTEGER,
            biotype TEXT,
            transcript_count INTEGER,
            sequence_length INTEGER
        );
        CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(gene_rowid, start, end);
        CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
        CREATE INDEX idx_gene_summary_name_norm ON gene_summary(name_normalized);
        CREATE INDEX idx_gene_summary_seqid_start ON gene_summary(seqid, start, gene_id);
        CREATE TABLE dataset_stats (dimension TEXT NOT NULL, value TEXT NOT NULL, gene_count INTEGER NOT NULL);
        INSERT INTO dataset_stats(dimension, value, gene_count) VALUES ('biotype', 'protein_coding', 1);
        INSERT INTO dataset_stats(dimension, value, gene_count) VALUES ('seqid', 'chr1', 1);",
    )
    .expect("schema");
    conn.execute(
        "INSERT INTO gene_summary(id,gene_id,name,name_normalized,seqid,start,end,biotype,transcript_count,sequence_length)
         VALUES (1,'ENSG000001','BRCA1','brca1','chr1',100,200,'protein_coding',1,101)",
        [],
    )
    .expect("insert");
    conn.execute(
        "INSERT INTO gene_summary_rtree(gene_rowid,start,end) VALUES (1,100,200)",
        [],
    )
    .expect("rtree insert");
    conn
}

fn bench_parse_plan_execute(c: &mut Criterion) {
    let req = sample_request();
    let limits = QueryLimits::default();
    let conn = setup_db();

    c.bench_function("query_parse_stage", |b| {
        b.iter(|| parse_gene_query_request(black_box(&req)).expect("parse"))
    });

    let ast = parse_gene_query_request(&req).expect("ast");
    c.bench_function("query_plan_stage", |b| {
        b.iter(|| plan_gene_query(black_box(&ast), black_box(&limits)).expect("plan"))
    });

    c.bench_function("query_execute_stage", |b| {
        b.iter(|| query_genes(black_box(&conn), black_box(&req), black_box(&limits), b"bench").expect("execute"))
    });
}

criterion_group!(benches, bench_parse_plan_execute);
criterion_main!(benches);
