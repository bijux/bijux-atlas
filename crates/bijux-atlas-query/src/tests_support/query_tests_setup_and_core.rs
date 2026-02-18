use super::*;
use rusqlite::Connection;
use std::time::Duration;

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
            CREATE TABLE transcript_summary (
              id INTEGER PRIMARY KEY,
              transcript_id TEXT NOT NULL UNIQUE,
              parent_gene_id TEXT NOT NULL,
              transcript_type TEXT NOT NULL,
              biotype TEXT,
              seqid TEXT NOT NULL,
              start INTEGER NOT NULL,
              end INTEGER NOT NULL,
              exon_count INTEGER NOT NULL DEFAULT 0,
              total_exon_span INTEGER NOT NULL DEFAULT 0,
              cds_present INTEGER NOT NULL DEFAULT 0
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
            CREATE INDEX idx_transcript_summary_transcript_id ON transcript_summary(transcript_id);
            CREATE INDEX idx_transcript_summary_parent_gene_id ON transcript_summary(parent_gene_id);
            CREATE INDEX idx_transcript_summary_biotype ON transcript_summary(biotype);
            CREATE INDEX idx_transcript_summary_type ON transcript_summary(transcript_type);
            CREATE INDEX idx_transcript_summary_region ON transcript_summary(seqid, start, end);
            ",
    )
    .expect("schema");

    let rows = vec![
        (1, "gene1", "BRCA1", "protein_coding", "chr1", 10, 40, 2, 31),
        (2, "gene2", "BRCA2", "protein_coding", "chr1", 50, 90, 1, 41),
        (3, "gene3", "TP53", "lncRNA", "chr2", 5, 25, 1, 21),
        (4, "gene4", "TNF", "lncRNA", "chr2", 30, 45, 1, 16),
        (5, "gene5", "BRCA_ABC", "unknown", "chr2", 50, 60, 1, 11),
        (
            6,
            "gene6",
            "DUPNAME",
            "protein_coding",
            "chr1",
            95,
            105,
            1,
            11,
        ),
        (
            7,
            "gene7",
            "DUPNAME",
            "protein_coding",
            "chr1",
            95,
            105,
            1,
            11,
        ),
    ];
    for r in rows {
        conn.execute(
            "INSERT INTO gene_summary (id, gene_id, name, name_normalized, biotype, seqid, start, end, transcript_count, exon_count, total_exon_span, cds_present, sequence_length)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, 0, ?10)",
            rusqlite::params![
                r.0,
                r.1,
                r.2,
                r.2.to_ascii_lowercase(),
                r.3,
                r.4,
                r.5,
                r.6,
                r.7,
                r.8
            ],
        )
        .expect("insert row");
        conn.execute(
            "INSERT INTO gene_summary_rtree (gene_rowid, start, end) VALUES (?1, ?2, ?3)",
            rusqlite::params![r.0, r.5 as f64, r.6 as f64],
        )
        .expect("insert rtree");
    }
    let tx_rows = vec![
        (
            "tx1",
            "gene1",
            "transcript",
            Some("protein_coding"),
            "chr1",
            10,
            20,
        ),
        (
            "tx2",
            "gene1",
            "mRNA",
            Some("protein_coding"),
            "chr1",
            21,
            40,
        ),
        (
            "tx3",
            "gene2",
            "transcript",
            Some("protein_coding"),
            "chr1",
            50,
            90,
        ),
    ];
    for (id, parent, kind, biotype, seqid, start, end) in tx_rows {
        conn.execute(
            "INSERT INTO transcript_summary (transcript_id,parent_gene_id,transcript_type,biotype,seqid,start,end,exon_count,total_exon_span,cds_present)
             VALUES (?1,?2,?3,?4,?5,?6,?7,1,?8,1)",
            rusqlite::params![id, parent, kind, biotype, seqid, start, end, end - start + 1],
        )
        .expect("insert transcript row");
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

fn setup_legacy_v2_db() -> Connection {
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
            CREATE INDEX idx_gene_summary_name ON gene_summary(name);
            CREATE INDEX idx_gene_summary_name_normalized ON gene_summary(name_normalized);
            CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
            CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
            INSERT INTO gene_summary (id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,sequence_length)
            VALUES (1,'gene1','BRCA1','brca1','protein_coding','chr1',10,40,2,31);
            INSERT INTO gene_summary_rtree (gene_rowid,start,end) VALUES (1,10,40);
            INSERT INTO dataset_stats (dimension,value,gene_count) VALUES ('biotype','protein_coding',1);
            INSERT INTO dataset_stats (dimension,value,gene_count) VALUES ('seqid','chr1',1);
            PRAGMA user_version=2;
            ",
    )
    .expect("legacy schema");
    conn
}

#[test]
fn transcript_query_uses_indexes_and_paginates() {
    let conn = setup_db();
    let req = TranscriptQueryRequest {
        filter: TranscriptFilter {
            parent_gene_id: Some("gene1".to_string()),
            biotype: None,
            transcript_type: None,
            region: None,
        },
        limit: 1,
        cursor: None,
    };
    let plan = explain_transcript_query_plan(&conn, &req).expect("tx explain");
    let joined = plan.join(" | ").to_ascii_lowercase();
    assert!(joined.contains("idx_transcript_summary_parent_gene_id"));
    let page1 = query_transcripts(&conn, &req).expect("page1");
    assert_eq!(page1.rows.len(), 1);
    let page2 = query_transcripts(
        &conn,
        &TranscriptQueryRequest {
            cursor: page1.next_cursor,
            ..req
        },
    )
    .expect("page2");
    assert_eq!(page2.rows.len(), 1);
}

fn limits() -> QueryLimits {
    QueryLimits {
        max_limit: 500,
        max_transcript_limit: 500,
        max_region_span: 5_000_000,
        max_region_estimated_rows: 1_000,
        max_prefix_cost_units: 80_000,
        heavy_projection_limit: 200,
        min_prefix_len: 2,
        max_prefix_len: 64,
        max_work_units: 2_000,
        max_serialization_bytes: 512 * 1024,
    }
}

#[test]
fn explain_plan_snapshots_by_query_class() {
    let conn = setup_db();
    let secret = b"test-secret";

    let cheap = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: Some("gene1".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let medium = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("protein_coding".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let heavy = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start: 1,
                end: 1_000,
            }),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };

    let cheap_plan = explain_query_plan(&conn, &cheap, &limits(), secret)
        .expect("plan")
        .join("\n")
        .to_ascii_lowercase();
    assert!(
        cheap_plan.contains("idx_gene_summary_gene_id"),
        "cheap plan must use gene_id index: {cheap_plan}"
    );

    let medium_plan = explain_query_plan(&conn, &medium, &limits(), secret)
        .expect("plan")
        .join("\n")
        .to_ascii_lowercase();
    assert!(
        medium_plan.contains("idx_gene_summary_biotype"),
        "medium plan must use biotype index: {medium_plan}"
    );

    let heavy_plan = explain_query_plan(&conn, &heavy, &limits(), secret)
        .expect("plan")
        .join("\n")
        .to_ascii_lowercase();
    assert!(
        heavy_plan.contains("virtual table index") || heavy_plan.contains("rtree"),
        "heavy plan must use rtree: {heavy_plan}"
    );
}

#[test]
fn legacy_v2_schema_remains_queryable() {
    let conn = setup_legacy_v2_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: Some("gene1".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let resp = query_genes(&conn, &req, &limits(), b"legacy-secret").expect("legacy query");
    assert_eq!(resp.rows.len(), 1);
    assert_eq!(resp.rows[0].gene_id, "gene1");
}

#[test]
fn missing_index_produces_diagnostic_error() {
    let conn = Connection::open_in_memory().expect("mem db");
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
        INSERT INTO gene_summary VALUES (1,'gene1','X','x','pc','chr1',1,2,1,2);
        INSERT INTO dataset_stats VALUES ('biotype','pc',1);
        INSERT INTO dataset_stats VALUES ('seqid','chr1',1);
        ",
    )
    .expect("schema");

    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("pc".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };

    let err = query_genes(&conn, &req, &limits(), b"s").expect_err("expected policy error");
    assert_eq!(err.code, QueryErrorCode::Policy);
    assert!(err.message.contains("full table scan") || err.message.contains("SCAN"));
}

#[test]
fn tie_break_ordering_is_stable_for_same_coordinates() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name: Some("DUPNAME".to_string()),
            ..Default::default()
        },
        limit: 20,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let rows = query_genes(&conn, &req, &limits(), b"s")
        .expect("rows")
        .rows;
    assert_eq!(
        rows.iter().map(|r| r.gene_id.as_str()).collect::<Vec<_>>(),
        vec!["gene6", "gene7"]
    );
}

#[test]
fn collation_normalized_name_lookup_is_case_insensitive() {
    let conn = setup_db();
    let upper = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name: Some("BRCA1".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let lower = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name: Some("brca1".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    assert_eq!(
        query_genes(&conn, &upper, &limits(), b"s")
            .expect("upper")
            .rows
            .len(),
        1
    );
    assert_eq!(
        query_genes(&conn, &lower, &limits(), b"s")
            .expect("lower")
            .rows
            .len(),
        1
    );
}

#[test]
fn projection_specific_query_uses_covering_name_index() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields {
            gene_id: true,
            name: true,
            coords: false,
            biotype: false,
            transcript_count: false,
            sequence_length: false,
        },
        filter: GeneFilter {
            name_prefix: Some("BR".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let plan = explain_query_plan(&conn, &req, &limits(), b"s")
        .expect("plan")
        .join("\n")
        .to_ascii_lowercase();
    assert!(
        plan.contains("idx_gene_summary_name_normalized")
            || plan.contains("idx_gene_summary_gene_id"),
        "projection query must use an indexed path for projection query: {plan}"
    );
    assert!(
        !plan.contains("scan gene_summary"),
        "projection query must not table-scan: {plan}"
    );
}
