// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bijux_atlas_model::{LATEST_ALIAS_POLICY, NO_IMPLICIT_DEFAULT_DATASET_POLICY};

pub const CRATE_NAME: &str = "bijux-atlas-api";
pub const API_POLICY_LATEST_ALIAS: &str = LATEST_ALIAS_POLICY;
pub const API_POLICY_NO_IMPLICIT_DEFAULT_DATASET: &str = NO_IMPLICIT_DEFAULT_DATASET_POLICY;

pub mod compat;
pub mod convert;
pub mod dto;
pub mod error_mapping;
pub mod errors;
mod generated;
pub mod openapi;
pub mod params;
pub mod responses;
pub mod wire;

pub use dto::DatasetKeyDto;
pub use errors::{ApiError, ApiErrorCode};
pub use openapi::openapi_v1_spec;
pub use params::{
    parse_list_genes_params, parse_list_genes_params_with_limit, parse_range_filter,
    parse_region_filter, IncludeField, ListGenesParams, MAX_CURSOR_BYTES,
};
pub use responses::{ApiContentType, ApiResponseEnvelope, ContentNegotiation};
pub use wire::{list_genes_v1, QueryAdapter};

#[must_use]
pub fn dataset_route_key(dataset: &DatasetKeyDto) -> String {
    dataset.route_key()
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
        q.insert("name_like".to_string(), "BR*".to_string());

        let parsed = parse_list_genes_params(&q).expect("params parse");
        assert_eq!(parsed.limit, 42);
        assert_eq!(parsed.name_like.as_deref(), Some("BR*"));
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
    fn parse_params_invalid_include() {
        let mut q = required_dataset_dims();
        q.insert("include".to_string(), "coords,nope".to_string());
        let err = parse_list_genes_params(&q).expect_err("invalid include");
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
    fn parse_range_strict_and_stable() {
        let parsed = parse_range_filter(Some("chr1:10-20".to_string())).expect("range parse");
        assert_eq!(parsed.expect("range").seqid, "chr1");
        let err = parse_range_filter(Some("chr1:20-10".to_string())).expect_err("invalid");
        assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    }

    #[test]
    fn api_error_details_schema_stable() {
        let e = ApiError::invalid_param("limit", "nope");
        assert!(e.details.get("field_errors").is_some());
        assert!(e.request_id.starts_with("req-"));
    }

    #[test]
    fn openapi_routes_and_determinism() {
        let spec = openapi_v1_spec();
        for route in [
            "/healthz",
            "/readyz",
            "/metrics",
            "/v1/datasets",
            "/v1/datasets/{release}/{species}/{assembly}",
            "/v1/releases/{release}/species/{species}/assemblies/{assembly}",
            "/v1/genes",
            "/v1/genes/count",
            "/v1/diff/genes",
            "/v1/diff/region",
            "/v1/sequence/region",
            "/v1/genes/{gene_id}/sequence",
            "/v1/genes/{gene_id}/transcripts",
            "/v1/transcripts/{tx_id}",
            "/debug/datasets",
            "/debug/dataset-health",
        ] {
            assert!(spec["paths"].get(route).is_some(), "missing route: {route}");
        }

        let a = bijux_atlas_core::canonical::stable_json_bytes(&spec).expect("stable bytes a");
        let b = bijux_atlas_core::canonical::stable_json_bytes(&openapi_v1_spec())
            .expect("stable bytes b");
        assert_eq!(a, b);
    }

    #[test]
    fn error_contract_matches_frozen_registry() {
        let freeze = include_str!("../docs/ssot/ERROR_CODES.json");
        let val: serde_json::Value = serde_json::from_str(freeze).expect("freeze json");
        let codes = val["codes"]
            .as_array()
            .expect("error_codes array")
            .iter()
            .map(|v| v.as_str().expect("code").to_string())
            .collect::<Vec<_>>();
        let runtime = crate::generated::error_codes::API_ERROR_CODES
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>();
        assert_eq!(runtime, codes);
    }

    #[test]
    fn api_error_json_field_order_is_stable() {
        let err = ApiError::invalid_param("limit", "bad");
        let encoded = serde_json::to_string(&err).expect("encode");
        assert_eq!(
            encoded,
            "{\"code\":\"InvalidQueryParameter\",\"message\":\"invalid query parameter: limit\",\"details\":{\"field_errors\":[{\"parameter\":\"limit\",\"reason\":\"invalid\",\"value\":\"bad\"}]},\"request_id\":\"req-unknown\"}"
        );
    }

    #[test]
    fn error_codes_match_generated_contract() {
        let generated = crate::generated::error_codes::API_ERROR_CODES;
        let from_enum = [
            ApiErrorCode::ArtifactCorrupted,
            ApiErrorCode::ArtifactQuarantined,
            ApiErrorCode::DatasetNotFound,
            ApiErrorCode::GeneNotFound,
            ApiErrorCode::IngestDuplicateTranscriptId,
            ApiErrorCode::IngestInvalidCdsPhase,
            ApiErrorCode::IngestInvalidStrand,
            ApiErrorCode::IngestMissingParent,
            ApiErrorCode::IngestMissingRequiredField,
            ApiErrorCode::IngestMissingTranscriptId,
            ApiErrorCode::IngestMultiParentChild,
            ApiErrorCode::IngestMultiParentTranscript,
            ApiErrorCode::IngestSeqidCollision,
            ApiErrorCode::IngestUnknownFeature,
            ApiErrorCode::Internal,
            ApiErrorCode::InvalidCursor,
            ApiErrorCode::InvalidQueryParameter,
            ApiErrorCode::MissingDatasetDimension,
            ApiErrorCode::NotReady,
            ApiErrorCode::PayloadTooLarge,
            ApiErrorCode::QueryRejectedByPolicy,
            ApiErrorCode::QueryTooExpensive,
            ApiErrorCode::RangeTooLarge,
            ApiErrorCode::RateLimited,
            ApiErrorCode::ResponseTooLarge,
            ApiErrorCode::Timeout,
            ApiErrorCode::UpstreamStoreUnavailable,
            ApiErrorCode::ValidationFailed,
        ]
        .map(ApiErrorCode::as_str);
        assert_eq!(generated, from_enum);
    }

    #[test]
    fn every_generated_error_code_is_parseable_and_stable() {
        for code in crate::generated::error_codes::API_ERROR_CODES {
            let parsed = ApiErrorCode::parse(code).expect("generated error code parse");
            assert_eq!(parsed.as_str(), *code);
        }
    }

    #[test]
    fn openapi_paths_match_endpoint_contract() {
        let spec = openapi_v1_spec();
        let spec_paths = spec
            .get("paths")
            .and_then(serde_json::Value::as_object)
            .expect("openapi paths object");

        let contract: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/reference/contracts/schemas/ENDPOINTS.json"
        ))
        .expect("parse endpoints contract");
        let expected = contract
            .get("endpoints")
            .and_then(serde_json::Value::as_array)
            .expect("endpoints array")
            .iter()
            .map(|e| {
                e.get("path")
                    .and_then(serde_json::Value::as_str)
                    .expect("path")
                    .to_string()
            })
            .collect::<std::collections::BTreeSet<_>>();
        let observed = spec_paths
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn openapi_paths_match_api_surface_registry() {
        let spec = openapi_v1_spec();
        let spec_paths = spec
            .get("paths")
            .and_then(serde_json::Value::as_object)
            .expect("openapi paths object")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();

        let contract: serde_json::Value =
            serde_json::from_str(include_str!("../docs/ssot/API_SURFACE.json"))
                .expect("parse api surface");
        let expected = contract
            .get("endpoints")
            .and_then(serde_json::Value::as_array)
            .expect("endpoints array")
            .iter()
            .map(|v| v.as_str().expect("path").to_string())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(spec_paths, expected);
    }
}
