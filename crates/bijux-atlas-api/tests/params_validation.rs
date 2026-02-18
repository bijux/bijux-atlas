use bijux_atlas_api::{
    openapi_v1_spec, parse_list_genes_params, parse_region_filter, ApiError, ApiErrorCode,
};
use bijux_atlas_core::canonical;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

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
        "include".to_string(),
        "coords,biotype,coords,length".to_string(),
    );
    let parsed = parse_list_genes_params(&q).expect("include");
    assert_eq!(parsed.include.expect("include").len(), 3);

    let mut bad = base_query();
    bad.insert("include".to_string(), "coords,,length".to_string());
    assert_eq!(
        parse_list_genes_params(&bad)
            .expect_err("invalid include")
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
fn request_validation_range_parser_is_strict_and_helpful() {
    let mut q = base_query();
    q.insert("range".to_string(), "chr1:10-20".to_string());
    let parsed = parse_list_genes_params(&q).expect("range parse");
    assert_eq!(parsed.range.as_deref(), Some("chr1:10-20"));

    let mut bad = base_query();
    bad.insert("range".to_string(), "chr1:20-10".to_string());
    let err = parse_list_genes_params(&bad).expect_err("invalid range");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    assert!(err.details["field_errors"][0]["parameter"].as_str() == Some("range"));

    let mut too_wide = base_query();
    too_wide.insert("range".to_string(), "chr1:1-5000001".to_string());
    let err = parse_list_genes_params(&too_wide).expect_err("range span capped");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
}

#[test]
fn request_validation_unknown_filter_rejected_with_allowed_list() {
    let mut q = base_query();
    q.insert("foo".to_string(), "bar".to_string());
    let err = parse_list_genes_params(&q).expect_err("unknown filter");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    assert!(err.message.contains("filter"));
    assert!(err.details["field_errors"][0]["value"]
        .as_str()
        .unwrap_or("")
        .contains("allowed"));
}

#[test]
fn request_validation_name_like_rejects_invalid_operator_forms() {
    let mut q = base_query();
    q.insert("name_like".to_string(), "*BRCA".to_string());
    let err = parse_list_genes_params(&q).expect_err("invalid wildcard");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);

    q.insert("name_like".to_string(), "BR*CA".to_string());
    let err = parse_list_genes_params(&q).expect_err("mid wildcard");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
}

#[test]
fn request_validation_contig_requires_and_matches_range() {
    let mut q = base_query();
    q.insert("contig".to_string(), "chr1".to_string());
    let err = parse_list_genes_params(&q).expect_err("contig requires range");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);

    q.insert("range".to_string(), "chr2:1-10".to_string());
    let err = parse_list_genes_params(&q).expect_err("contig mismatch");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
}

#[test]
fn request_validation_sort_contract_is_strict() {
    let mut q = base_query();
    q.insert("sort".to_string(), "gene_id:asc".to_string());
    parse_list_genes_params(&q).expect("default sort accepted");

    q.insert("sort".to_string(), "region:asc".to_string());
    parse_list_genes_params(&q).expect("region sort parsed");

    q.insert("sort".to_string(), "gene_id:desc".to_string());
    let err = parse_list_genes_params(&q).expect_err("unsupported direction");
    assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
}

#[test]
fn request_validation_filter_parsing_is_order_independent() {
    let mut q1 = base_query();
    q1.insert("gene_id".to_string(), "ENSG1".to_string());
    q1.insert("name_like".to_string(), "BRCA*".to_string());
    q1.insert("biotype".to_string(), "protein_coding".to_string());

    let mut q2 = base_query();
    q2.insert("biotype".to_string(), "protein_coding".to_string());
    q2.insert("name_like".to_string(), "BRCA*".to_string());
    q2.insert("gene_id".to_string(), "ENSG1".to_string());

    assert_eq!(
        parse_list_genes_params(&q1).ok(),
        parse_list_genes_params(&q2).ok()
    );
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
        .join("../../configs/openapi/v1/openapi.snapshot.json");
    let snapshot = std::fs::read(snapshot_path).expect("read snapshot");
    assert_eq!(generated, snapshot);
}

fn compatibility_delta(previous: &serde_json::Value, current: &serde_json::Value) -> Vec<String> {
    let mut issues = Vec::new();
    let prev_paths = previous["paths"]
        .as_object()
        .expect("previous paths object");
    let cur_paths = current["paths"].as_object().expect("current paths object");

    let prev_set = prev_paths.keys().cloned().collect::<BTreeSet<_>>();
    let cur_set = cur_paths.keys().cloned().collect::<BTreeSet<_>>();
    for removed in prev_set.difference(&cur_set) {
        issues.push(format!("removed path: {removed}"));
    }

    for path in prev_set.intersection(&cur_set) {
        let prev_ops = prev_paths[path]
            .as_object()
            .expect("previous path operations");
        let cur_ops = cur_paths[path]
            .as_object()
            .expect("current path operations");
        let prev_ops_set = prev_ops.keys().cloned().collect::<BTreeSet<_>>();
        let cur_ops_set = cur_ops.keys().cloned().collect::<BTreeSet<_>>();
        for removed_op in prev_ops_set.difference(&cur_ops_set) {
            issues.push(format!("removed operation: {path}::{removed_op}"));
        }
    }

    issues
}

#[test]
fn openapi_minor_version_bump_remains_compatible() {
    let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/openapi/v1/openapi.snapshot.json");
    let previous: serde_json::Value =
        serde_json::from_slice(&std::fs::read(snapshot_path).expect("read snapshot"))
            .expect("parse snapshot");
    let current = openapi_v1_spec();

    let issues = compatibility_delta(&previous, &current);
    assert!(
        issues.is_empty(),
        "current openapi must be backward compatible with snapshot: {issues:?}"
    );

    let mut bumped = current.clone();
    bumped["info"]["version"] = serde_json::Value::String("v1.1".to_string());
    assert!(
        compatibility_delta(&current, &bumped).is_empty(),
        "minor info.version bump should remain compatible when surface is unchanged"
    );
}
