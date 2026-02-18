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
        dataset_key: None,
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
            name_prefix: Some("AL".to_string()),
            ..Default::default()
        },
        limit: 500,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let mut lim = limits();
    lim.max_prefix_cost_units = 100;
    let err = query_genes(&conn, &req, &lim, b"s").expect_err("prefix rejection");
    assert_eq!(err.code, QueryErrorCode::Validation);
    assert!(err.message.contains("name_prefix estimated cost"));
}

#[test]
fn unicode_normalization_policy_nfkc_is_stable() {
    let n1 = crate::filters::normalize_name_lookup("Å");
    let n2 = crate::filters::normalize_name_lookup("Å");
    assert_eq!(n1, n2);
}

#[test]
fn prefix_lower_bound_enforcement_rejects_single_char() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("A".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let err = query_genes(&conn, &req, &limits(), b"s").expect_err("reject short prefix");
    assert_eq!(err.code, QueryErrorCode::Validation);
    assert!(err.message.contains("name_prefix length must be >="));
}

#[test]
fn empty_result_pagination_returns_none_cursor() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: Some("missing-gene".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let resp = query_genes(&conn, &req, &limits(), b"s").expect("query");
    assert!(resp.rows.is_empty());
    assert!(resp.next_cursor.is_none());
}

#[test]
fn no_table_scan_assertion_for_indexed_query_plan() {
    let conn = setup_db();
    let req = GeneQueryRequest {
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
    let plan = explain_query_plan(&conn, &req, &limits(), b"s")
        .expect("plan")
        .join("\n")
        .to_ascii_lowercase();
    assert!(
        !plan.contains("scan gene_summary"),
        "unexpected table scan: {plan}"
    );
}

#[test]
fn region_overlap_edge_cases_are_correct() {
    let conn = setup_db();
    let mk = |start: u64, end: u64| GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start,
                end,
            }),
            ..Default::default()
        },
        limit: 50,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let point = query_genes(&conn, &mk(10, 10), &limits(), b"s").expect("point");
    assert!(point.rows.iter().any(|r| r.gene_id == "gene1"));
    let outside = query_genes(&conn, &mk(5000, 6000), &limits(), b"s").expect("outside");
    assert!(outside.rows.is_empty());
    let overlap = query_genes(&conn, &mk(35, 55), &limits(), b"s").expect("overlap");
    let ids = overlap
        .rows
        .iter()
        .map(|r| r.gene_id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"gene1"));
    assert!(ids.contains(&"gene2"));
    let exact = query_genes(&conn, &mk(50, 90), &limits(), b"s").expect("exact");
    assert!(exact.rows.iter().any(|r| r.gene_id == "gene2"));
}

#[test]
fn json_serialization_ordering_is_stable() {
    let conn = setup_db();
    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("lncRNA".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let resp = query_genes(&conn, &req, &limits(), b"s").expect("query");
    let a = bijux_atlas_core::canonical::stable_json_bytes(&resp).expect("a");
    let b = bijux_atlas_core::canonical::stable_json_bytes(&resp).expect("b");
    assert_eq!(a, b);
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
        dataset_key: None,
        allow_full_scan: false,
    };
    assert_eq!(classify_query(&req), QueryClass::Heavy);
    assert!(estimate_work_units(&req) > 1_000);
    assert!(estimate_query_cost(&req).work_units > 1_000);

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
        dataset_key: None,
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
        dataset_key: None,
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
        dataset_key: None,
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
        dataset_key: None,
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
        dataset_key: None,
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
        dataset_key: None,
        allow_full_scan: false,
    };
    let started = std::time::Instant::now();
    let _ = query_genes(&conn, &req, &limits(), b"bench-secret").expect("query");
    assert!(
        started.elapsed() < Duration::from_millis(50),
        "in-memory query exceeded baseline threshold"
    );
}

#[test]
fn shard_selection_targets_region_seqid_and_defaults_global() {
    let dataset =
        bijux_atlas_model::DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let catalog = bijux_atlas_model::ShardCatalog::new(
        dataset,
        "per-seqid".to_string(),
        vec![
            bijux_atlas_model::ShardEntry::new(
                "chr1".to_string(),
                vec!["chr1".to_string()],
                "gene_summary.chr1.sqlite".to_string(),
                "abc".to_string(),
            ),
            bijux_atlas_model::ShardEntry::new(
                "chr2".to_string(),
                vec!["chr2".to_string()],
                "gene_summary.chr2.sqlite".to_string(),
                "def".to_string(),
            ),
        ],
    );
    let region = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr2".to_string(),
                start: 1,
                end: 10,
            }),
            ..Default::default()
        },
        limit: 5,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    assert_eq!(
        select_shards_for_request(&region, &catalog),
        vec!["gene_summary.chr2.sqlite".to_string()]
    );

    let non_region = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: Some("gene1".to_string()),
            ..Default::default()
        },
        limit: 1,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    assert_eq!(
        select_shards_for_request(&non_region, &catalog),
        vec!["gene_summary.sqlite".to_string()]
    );
}

#[test]
fn sharded_and_monolithic_responses_are_identical_for_region() {
    let monolith = setup_db();
    let shard = setup_db();
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
        limit: 50,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    let mono = query_genes(&monolith, &req, &limits(), b"s").expect("monolith");
    let fanout = query_genes_fanout(&[&shard], &req, &limits(), b"s").expect("fanout");
    assert_eq!(mono.rows, fanout.rows);
    assert_eq!(mono.next_cursor, fanout.next_cursor);
}
