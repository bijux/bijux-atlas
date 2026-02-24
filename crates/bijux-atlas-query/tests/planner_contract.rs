// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_atlas_query::{
    parse_gene_query_request, plan_gene_query, GeneFields, GeneFilter, GeneQueryRequest,
    QueryLimits,
};

fn fixture(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join(path)).expect("read fixture")
}

fn request(filter: GeneFilter, limit: usize) -> GeneQueryRequest {
    GeneQueryRequest {
        fields: GeneFields::default(),
        filter,
        limit,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    }
}

#[test]
fn planner_output_matches_golden_contracts() {
    let cases = vec![
        (
            request(
                GeneFilter {
                    gene_id: Some("ENSG000001".to_string()),
                    ..Default::default()
                },
                5,
            ),
            "tests/fixtures/planner/point_lookup.json",
        ),
        (
            request(
                GeneFilter {
                    region: Some(bijux_atlas_query::RegionFilter {
                        seqid: "chr1".to_string(),
                        start: 10,
                        end: 10_000,
                    }),
                    ..Default::default()
                },
                7,
            ),
            "tests/fixtures/planner/region_scan.json",
        ),
        (
            request(
                GeneFilter {
                    name_prefix: Some("BRCA".to_string()),
                    ..Default::default()
                },
                11,
            ),
            "tests/fixtures/planner/prefix_search.json",
        ),
    ];

    for (req, golden_path) in cases {
        let ast = parse_gene_query_request(&req).expect("parse");
        let plan = plan_gene_query(&ast, &QueryLimits::default()).expect("plan");
        let actual = serde_json::to_string_pretty(&plan).expect("json");
        let expected = fixture(golden_path);
        assert_eq!(
            actual.trim(),
            expected.trim(),
            "golden mismatch: {golden_path}"
        );
    }
}
