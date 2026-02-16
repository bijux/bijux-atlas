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
            "INSERT INTO gene_summary (id, gene_id, name, name_normalized, biotype, seqid, start, end, transcript_count, sequence_length)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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

fn limits() -> QueryLimits {
    QueryLimits {
        max_limit: 500,
        max_region_span: 5_000_000,
        max_region_estimated_rows: 1_000,
        max_prefix_cost_units: 80_000,
        heavy_projection_limit: 200,
        min_prefix_len: 1,
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
fn case_sensitive_collation_is_consistent() {
    let conn = setup_db();
    let upper = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name: Some("BRCA1".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
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
        0
    );
}

#[test]
fn cursor_generation_is_concurrency_stable() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("lncRNA".to_string()),
            ..Default::default()
        },
        limit: 1,
        cursor: None,
        allow_full_scan: false,
    };

    let mut cursors = Vec::new();
    for _ in 0..16 {
        let resp = query_genes(&conn, &req, &limits(), b"stable-secret").expect("query");
        cursors.push(resp.next_cursor.expect("cursor"));
    }
    for c in &cursors[1..] {
        assert_eq!(c, &cursors[0]);
    }
}

#[test]
fn fast_path_gene_lookup_returns_single_row_without_cursor() {
    let conn = setup_db();
    let fields = GeneFields {
        gene_id: true,
        name: true,
        coords: false,
        biotype: false,
        transcript_count: false,
        sequence_length: false,
    };
    let row = query_gene_by_id_fast(&conn, "gene1", &fields)
        .expect("fast query")
        .expect("row");
    assert_eq!(row.gene_id, "gene1");
    assert_eq!(row.name.as_deref(), Some("BRCA1"));
    assert!(row.seqid.is_none());
}

#[test]
fn minimal_gene_id_name_json_fast_path_returns_compact_payload() {
    let conn = setup_db();
    let payload = query_gene_id_name_json_minimal_fast(&conn, "gene1")
        .expect("query")
        .expect("row");
    let txt = String::from_utf8(payload).expect("utf8");
    assert!(txt.contains("\"gene_id\":\"gene1\""));
    assert!(txt.contains("\"name\":\"BRCA1\""));
    assert!(!txt.contains("seqid"));
}

#[test]
fn pathological_prefix_is_rejected_by_cost_estimator() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("A".to_string()),
            ..Default::default()
        },
        limit: 500,
        cursor: None,
        allow_full_scan: false,
    };
    let mut lim = limits();
    lim.max_prefix_cost_units = 100;
    let err = query_genes(&conn, &req, &lim, b"s").expect_err("prefix rejection");
    assert_eq!(err.code, QueryErrorCode::Validation);
    assert!(err.message.contains("name_prefix estimated cost"));
}

#[test]
fn cost_estimator_and_limits_enforced() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start: 1,
                end: 20_000_000,
            }),
            ..Default::default()
        },
        limit: 200,
        cursor: None,
        allow_full_scan: false,
    };
    assert_eq!(classify_query(&req), QueryClass::Heavy);
    assert!(estimate_work_units(&req) > 1_000);

    let strict = QueryLimits {
        max_work_units: 100,
        ..limits()
    };
    let err = query_genes(&conn, &req, &strict, b"s").expect_err("cost rejection");
    assert_eq!(err.code, QueryErrorCode::Validation);
}

#[test]
fn fast_fail_rejects_impossible_filters_from_dataset_stats() {
    let conn = setup_db();
    let impossible_biotype = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("not_a_real_biotype".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        allow_full_scan: false,
    };
    let err = query_genes(&conn, &impossible_biotype, &limits(), b"s").expect_err("must fast fail");
    assert_eq!(err.code, QueryErrorCode::Validation);
    assert!(err.message.contains("biotype does not exist"));

    let impossible_seqid = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chrMissing".to_string(),
                start: 1,
                end: 100,
            }),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        allow_full_scan: false,
    };
    let err = query_genes(&conn, &impossible_seqid, &limits(), b"s").expect_err("fast fail");
    assert_eq!(err.code, QueryErrorCode::Validation);
    assert!(err.message.contains("region seqid does not exist"));
}

#[test]
fn normalization_hash_is_cursor_and_param_order_stable() {
    let req_a = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("protein_coding".to_string()),
            name_prefix: Some("BR".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: Some("abc".to_string()),
        allow_full_scan: false,
    };
    let mut req_b = req_a.clone();
    req_b.cursor = None;

    let h1 = query_normalization_hash(&req_a).expect("hash a");
    let h2 = query_normalization_hash(&req_b).expect("hash b");
    assert_eq!(h1, h2, "cursor must not affect normalization hash");
}

#[test]
fn region_ordering_is_stable() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start: 1,
                end: 200,
            }),
            ..Default::default()
        },
        limit: 20,
        cursor: None,
        allow_full_scan: false,
    };
    let rows = query_genes(&conn, &req, &limits(), b"s")
        .expect("region rows")
        .rows;
    let ids = rows.iter().map(|r| r.gene_id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, vec!["gene1", "gene2", "gene6", "gene7"]);
}

#[test]
fn cursor_error_maps_to_stable_code() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: Some("gene1".to_string()),
            ..Default::default()
        },
        limit: 1,
        cursor: Some("broken.cursor".to_string()),
        allow_full_scan: false,
    };
    let err = query_genes(&conn, &req, &limits(), b"s").expect_err("cursor reject");
    assert_eq!(err.code, QueryErrorCode::Cursor);
}

#[test]
fn query_crate_has_no_axum_or_server_dependency() {
    let cargo = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .expect("read Cargo.toml");
    for forbidden in ["axum", "bijux-atlas-server"] {
        assert!(
            !cargo.contains(forbidden),
            "forbidden dependency in query crate: {forbidden}"
        );
    }
}

#[test]
fn benchmark_threshold_sanity_non_regression() {
    let conn = setup_db();
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
    let started = std::time::Instant::now();
    let _ = query_genes(&conn, &req, &limits(), b"bench-secret").expect("query");
    assert!(
        started.elapsed() < Duration::from_millis(50),
        "in-memory query exceeded baseline threshold"
    );
}
