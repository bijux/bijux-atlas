use bijux_atlas_query::{parse_gene_query_request, plan_gene_query, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits};

#[test]
fn ast_normalization_is_stable_across_filter_order() {
    let req_a = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            biotype: Some("protein_coding".to_string()),
            name_prefix: Some("BRCA".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: Some("release=110&species=homo_sapiens&assembly=GRCh38".to_string()),
        allow_full_scan: false,
    };

    let req_b = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            name_prefix: Some("BRCA".to_string()),
            biotype: Some("protein_coding".to_string()),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: Some("release=110&species=homo_sapiens&assembly=GRCh38".to_string()),
        allow_full_scan: false,
    };

    let ast_a = parse_gene_query_request(&req_a).expect("parse a");
    let ast_b = parse_gene_query_request(&req_b).expect("parse b");

    let plan_a = plan_gene_query(&ast_a, &QueryLimits::default()).expect("plan a");
    let plan_b = plan_gene_query(&ast_b, &QueryLimits::default()).expect("plan b");

    assert_eq!(plan_a.normalized, plan_b.normalized);
}
