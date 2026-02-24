use bijux_atlas_api::compat::{
    compatibility_channel, legacy_dataset_path_redirect, ApiCompatibilityChannel,
};
use bijux_atlas_api::error_mapping::{map_error, API_ERROR_SCHEMA_REF};
use bijux_atlas_api::{ApiError, ApiErrorCode, DatasetKeyDto};
use serde_json::json;

#[test]
fn legacy_dataset_path_redirect_contract_is_stable() {
    let dataset = DatasetKeyDto::new(
        "110".to_string(),
        "homo_sapiens".to_string(),
        "GRCh38".to_string(),
    )
    .expect("dataset dto");

    let redirect = legacy_dataset_path_redirect(&dataset);
    assert_eq!(
        redirect.from_path,
        "/v1/releases/110/species/homo_sapiens/assemblies/GRCh38"
    );
    assert_eq!(redirect.to_path, "/v1/datasets/110/homo_sapiens/GRCh38");
    assert_eq!(redirect.status_code, 308);
}

#[test]
fn compatibility_channel_marks_legacy_route_family() {
    assert_eq!(
        compatibility_channel("/v1/releases/110/species/homo_sapiens/assemblies/GRCh38"),
        ApiCompatibilityChannel::LegacyV0Redirect
    );
    assert_eq!(
        compatibility_channel("/v1/datasets/110/homo_sapiens/GRCh38"),
        ApiCompatibilityChannel::StableV1
    );
}

#[test]
fn api_error_mapping_is_centralized_and_stable() {
    let err = ApiError::new(
        ApiErrorCode::QueryRejectedByPolicy,
        "rejected",
        json!({}),
        "req-1",
    );
    let mapped = map_error(&err);
    assert_eq!(mapped.status_code, 422);
    assert_eq!(mapped.schema_ref, API_ERROR_SCHEMA_REF);
}
