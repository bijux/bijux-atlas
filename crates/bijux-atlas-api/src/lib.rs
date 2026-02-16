#![forbid(unsafe_code)]

use bijux_atlas_model::{DatasetId, LATEST_ALIAS_POLICY, NO_IMPLICIT_DEFAULT_DATASET_POLICY};

#[cfg(feature = "allow-raw-genome-io")]
compile_error!(
    "Feature `allow-raw-genome-io` is forbidden in API layer: raw GFF3/FASTA reads are disallowed"
);

pub const CRATE_NAME: &str = "bijux-atlas-api";
pub const API_POLICY_LATEST_ALIAS: &str = LATEST_ALIAS_POLICY;
pub const API_POLICY_NO_IMPLICIT_DEFAULT_DATASET: &str = NO_IMPLICIT_DEFAULT_DATASET_POLICY;

pub mod errors;
pub mod openapi;
pub mod params;
pub mod responses;

pub use errors::{ApiError, ApiErrorCode};
pub use openapi::openapi_v1_spec;
pub use params::{
    parse_list_genes_params, parse_list_genes_params_with_limit, parse_region_filter,
    ListGenesParams, MAX_CURSOR_BYTES,
};
pub use responses::{ApiContentType, ApiResponseEnvelope, ContentNegotiation};

#[must_use]
pub fn dataset_route_key(dataset: &DatasetId) -> String {
    format!(
        "release={}/species={}/assembly={}",
        dataset.release, dataset.species, dataset.assembly
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn required_dataset_dims() -> BTreeMap<String, String> {
        let mut q = BTreeMap::new();
        q.insert("release".to_string(), "110".to_string());
        q.insert("species".to_string(), "homo_sapiens".to_string());
        q.insert("assembly".to_string(), "GRCh38".to_string());
        q
    }

    #[test]
    fn parse_params_success_exhaustive() {
        let mut q = required_dataset_dims();
        q.insert("limit".to_string(), "42".to_string());
        q.insert("name_prefix".to_string(), "BR".to_string());

        let parsed = parse_list_genes_params(&q).expect("params parse");
        assert_eq!(parsed.limit, 42);
        assert_eq!(parsed.name_prefix.as_deref(), Some("BR"));
        assert!(!parsed.pretty);
    }

    #[test]
    fn parse_params_missing_dimensions() {
        let q = BTreeMap::new();
        let err = parse_list_genes_params(&q).expect_err("expected error");
        assert_eq!(err.code, ApiErrorCode::MissingDatasetDimension);
    }

    #[test]
    fn parse_params_invalid_limit() {
        let mut q = required_dataset_dims();
        q.insert("limit".to_string(), "nope".to_string());

        let err = parse_list_genes_params(&q).expect_err("expected invalid limit");
        assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    }

    #[test]
    fn parse_params_cursor_size_validation() {
        let mut q = required_dataset_dims();
        q.insert("cursor".to_string(), "x".repeat(MAX_CURSOR_BYTES + 1));
        let err = parse_list_genes_params(&q).expect_err("cursor too large");
        assert_eq!(err.code, ApiErrorCode::InvalidCursor);
    }

    #[test]
    fn parse_params_invalid_fields() {
        let mut q = required_dataset_dims();
        q.insert("fields".to_string(), "name,nope".to_string());
        let err = parse_list_genes_params(&q).expect_err("invalid fields");
        assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    }

    #[test]
    fn parse_region_strict_and_stable() {
        let parsed = parse_region_filter(Some("chr1:10-20".to_string())).expect("region parse");
        assert_eq!(parsed.expect("region").seqid, "chr1");
        let err = parse_region_filter(Some("chr1:20-10".to_string())).expect_err("invalid");
        assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    }

    #[test]
    fn api_error_details_schema_stable() {
        let e = ApiError::invalid_param("limit", "nope");
        assert!(e.details.get("parameter").is_some());
        assert!(e.details.get("value").is_some());
    }

    #[test]
    fn openapi_routes_and_determinism() {
        let spec = openapi_v1_spec();
        for route in [
            "/healthz",
            "/readyz",
            "/metrics",
            "/v1/datasets",
            "/v1/genes",
            "/v1/genes/count",
            "/debug/datasets",
        ] {
            assert!(spec["paths"].get(route).is_some(), "missing route: {route}");
        }

        let a = bijux_atlas_core::canonical::stable_json_bytes(&spec).expect("stable bytes a");
        let b = bijux_atlas_core::canonical::stable_json_bytes(&openapi_v1_spec())
            .expect("stable bytes b");
        assert_eq!(a, b);
    }
}
