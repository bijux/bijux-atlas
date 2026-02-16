use super::*;

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
            110,
            120,
            1,
            11,
        ),
    ];
    for r in rows {
        conn.execute(
                "INSERT INTO gene_summary (id, gene_id, name, biotype, seqid, start, end, transcript_count, sequence_length)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7, r.8],
            )
            .expect("insert row");
        conn.execute(
            "INSERT INTO gene_summary_rtree (gene_rowid, start, end) VALUES (?1, ?2, ?3)",
            rusqlite::params![r.0, r.5 as f64, r.6 as f64],
        )
        .expect("insert rtree");
    }
    conn
}

#[test]
fn determinism_pagination_and_cursor() {
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
    let limits = QueryLimits::default();
    let secret = b"test-secret";

    let first = query_genes(&conn, &req, &limits, secret).expect("first page");
    assert_eq!(first.rows.len(), 1);
    assert!(first.next_cursor.is_some());

    let mut req2 = req.clone();
    req2.cursor = first.next_cursor.clone();
    let second = query_genes(&conn, &req2, &limits, secret).expect("second page");
    assert_eq!(second.rows.len(), 1);
    assert_ne!(first.rows[0].gene_id, second.rows[0].gene_id);

    let repeat = query_genes(&conn, &req, &limits, secret).expect("repeat first page");
    assert_eq!(first.rows, repeat.rows);
    assert_eq!(first.next_cursor, repeat.next_cursor);
}

#[test]
fn region_query_uses_rtree_and_ordering() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start: 1,
                end: 100,
            }),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        allow_full_scan: false,
    };
    let resp = query_genes(&conn, &req, &QueryLimits::default(), b"secret").expect("region");
    assert_eq!(resp.rows.len(), 3);
    assert_eq!(resp.rows[0].gene_id, "gene1");
    assert_eq!(resp.rows[1].gene_id, "gene2");
}

#[test]
fn explain_query_plan_snapshot_contains_expected_indexes() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("BR".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        allow_full_scan: false,
    };
    let (sql, params) = build_sql(&req, OrderMode::GeneId, None).expect("build sql");
    let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let mut stmt = conn.prepare(&explain_sql).expect("prepare explain");
    let mut explain_params = params;
    explain_params.push(Value::Integer((req.limit as i64) + 1));
    let plan = stmt
        .query_map(params_from_iter(explain_params.iter()), |row| {
            row.get::<_, String>(3)
        })
        .expect("run explain")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect plan")
        .join("\n");

    assert!(
        plan.contains("idx_gene_summary_name") || plan.contains("idx_gene_summary_gene_id"),
        "plan must use indexed access, got: {plan}"
    );
}

#[test]
fn classification_and_max_work_guard() {
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
}

#[test]
fn cursor_is_stable_for_same_query_dataset_hash() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("BR".to_string()),
            ..Default::default()
        },
        limit: 2,
        cursor: None,
        allow_full_scan: false,
    };
    let a = query_genes(&conn, &req, &QueryLimits::default(), b"stable-secret").expect("a");
    let b = query_genes(&conn, &req, &QueryLimits::default(), b"stable-secret").expect("b");
    assert_eq!(a.next_cursor, b.next_cursor);
}

#[test]
fn region_boundaries_are_handled() {
    let conn = setup_db();
    let mk = |start, end| GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start,
                end,
            }),
            ..Default::default()
        },
        limit: 20,
        cursor: None,
        allow_full_scan: false,
    };

    let exact = query_genes(&conn, &mk(10, 10), &QueryLimits::default(), b"s").expect("exact");
    assert_eq!(exact.rows[0].gene_id, "gene1");

    let outside =
        query_genes(&conn, &mk(1000, 2000), &QueryLimits::default(), b"s").expect("outside");
    assert!(outside.rows.is_empty());

    let overlap = query_genes(&conn, &mk(38, 55), &QueryLimits::default(), b"s").expect("overlap");
    assert_eq!(
        overlap
            .rows
            .iter()
            .map(|r| r.gene_id.as_str())
            .collect::<Vec<_>>(),
        vec!["gene1", "gene2"]
    );

    let single = query_genes(&conn, &mk(50, 50), &QueryLimits::default(), b"s").expect("single");
    assert_eq!(single.rows[0].gene_id, "gene2");
}

#[test]
fn name_prefix_ascii_and_underscore() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("BRCA_".to_string()),
            ..Default::default()
        },
        limit: 20,
        cursor: None,
        allow_full_scan: false,
    };
    let rows = query_genes(&conn, &req, &QueryLimits::default(), b"s")
        .expect("prefix")
        .rows;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].gene_id, "gene5");
}

#[test]
fn biotype_missing_representation_unknown() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("unknown".to_string()),
            ..Default::default()
        },
        limit: 20,
        cursor: None,
        allow_full_scan: false,
    };
    let rows = query_genes(&conn, &req, &QueryLimits::default(), b"s")
        .expect("biotype")
        .rows;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].gene_id, "gene5");
}

#[test]
fn multiple_genes_same_name_order_is_stable() {
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
    let rows = query_genes(&conn, &req, &QueryLimits::default(), b"s")
        .expect("same-name")
        .rows;
    assert_eq!(
        rows.iter().map(|r| r.gene_id.as_str()).collect::<Vec<_>>(),
        vec!["gene6", "gene7"]
    );
}

#[test]
fn exact_name_matching_is_case_sensitive() {
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
        query_genes(&conn, &upper, &QueryLimits::default(), b"s")
            .expect("upper")
            .rows
            .len(),
        1
    );
    assert_eq!(
        query_genes(&conn, &lower, &QueryLimits::default(), b"s")
            .expect("lower")
            .rows
            .len(),
        0
    );
}
