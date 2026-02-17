use bijux_atlas_api::{
    openapi_v1_spec, parse_list_genes_params, parse_region_filter, ApiError, ApiErrorCode,
};
use bijux_atlas_core::canonical;
use std::collections::BTreeMap;

fn base_query() -> BTreeMap<String, String> {
    let mut q = BTreeMap::new();
    q.insert("release".to_string(), "110".to_string());
    q.insert("species".to_string(), "homo_sapiens".to_string());
    q.insert("assembly".to_string(), "GRCh38".to_string());
    q
}

#[test]
fn request_validation_limit_bounds_are_enforced() {
    let mut zero = base_query();
    zero.insert("limit".to_string(), "0".to_string());
    assert_eq!(
        parse_list_genes_params(&zero).expect_err("limit=0").code,
        ApiErrorCode::InvalidQueryParameter
    );

    let mut over = base_query();
    over.insert("limit".to_string(), "501".to_string());
    assert_eq!(
        parse_list_genes_params(&over).expect_err("limit>max").code,
        ApiErrorCode::InvalidQueryParameter
    );

    let mut max = base_query();
    max.insert("limit".to_string(), "500".to_string());
    assert_eq!(parse_list_genes_params(&max).expect("limit=max").limit, 500);
}

#[test]
fn request_validation_fields_are_strict_and_deduplicated() {
    let mut q = base_query();
    q.insert(
        "fields".to_string(),
        "name,gene_id,name,biotype".to_string(),
    );
    let parsed = parse_list_genes_params(&q).expect("fields");
    assert_eq!(
        parsed.fields.expect("fields"),
        vec![
            "name".to_string(),
            "gene_id".to_string(),
            "biotype".to_string()
        ]
    );

    let mut bad = base_query();
    bad.insert("fields".to_string(), "gene_id,,name".to_string());
    assert_eq!(
        parse_list_genes_params(&bad)
            .expect_err("invalid fields")
            .code,
        ApiErrorCode::InvalidQueryParameter
    );
}

#[test]
fn request_validation_region_parser_is_strict() {
    let valid = parse_region_filter(Some("chr1:10-20".to_string())).expect("valid region");
    assert_eq!(valid.expect("region").start, 10);

    for raw in ["chr1", "chr1:10", "chr1:0-10", "chr1:20-10", "chr1:x-10"] {
        let err = parse_region_filter(Some(raw.to_string())).expect_err("invalid region");
        assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    }
}

#[test]
fn request_validation_requires_explicit_dataset_dimensions() {
    for dim in ["release", "species", "assembly"] {
        let mut q = base_query();
        q.remove(dim);
        let err = parse_list_genes_params(&q).expect_err("missing dimension");
        assert_eq!(err.code, ApiErrorCode::MissingDatasetDimension);
    }
}

#[test]
fn request_validation_pretty_flag_contract() {
    let mut q = base_query();
    q.insert("pretty".to_string(), "true".to_string());
    assert!(parse_list_genes_params(&q).expect("pretty=true").pretty);

    q.insert("pretty".to_string(), "1".to_string());
    assert!(parse_list_genes_params(&q).expect("pretty=1").pretty);

    q.insert("pretty".to_string(), "false".to_string());
    assert!(!parse_list_genes_params(&q).expect("pretty=false").pretty);
}

#[test]
fn error_schema_rejects_unknown_fields() {
    let raw = r#"{"code":"InvalidQueryParameter","message":"bad","details":{},"extra":1}"#;
    let err = serde_json::from_str::<ApiError>(raw).expect_err("deny unknown fields");
    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn openapi_snapshot_is_deterministic_and_matches_committed_contract() {
    let generated = canonical::stable_json_bytes(&openapi_v1_spec()).expect("serialize generated");
    let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../openapi/v1/openapi.snapshot.json");
    let snapshot = std::fs::read(snapshot_path).expect("read snapshot");
    assert_eq!(generated, snapshot);
}
