// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::api::client::{
    run_with_retry as alias_run_with_retry, AtlasClient as AliasAtlasClient,
    ClientConfig as AliasClientConfig,
};
use bijux_atlas::domain::ingest::IngestOptions as AliasIngestOptions;
use bijux_atlas::domain::sha256_hex as alias_sha256_hex;
use bijux_atlas::query::Region as AliasRegion;
use bijux_atlas_api::client::{
    run_with_retry as canonical_run_with_retry, AtlasClient as CanonicalAtlasClient,
    ClientConfig as CanonicalClientConfig,
};
use bijux_atlas_ingest::IngestOptions as RuntimeIngestOptions;
use bijux_atlas_query::Region as QueryRegion;
use bijux_atlas_runtime::domain::sha256_hex as runtime_sha256_hex;

#[test]
fn alias_reexports_runtime_root_functions() {
    assert_eq!(
        bijux_atlas::no_randomness_policy(),
        bijux_atlas_runtime::no_randomness_policy()
    );
}

#[test]
fn alias_reexports_runtime_module_functions() {
    assert_eq!(alias_sha256_hex(b"atlas"), runtime_sha256_hex(b"atlas"));
}

#[test]
fn alias_query_types_match_canonical_query_types() {
    let query_region = QueryRegion::parse("chr1:10-20").expect("query region");
    let alias_region: AliasRegion = query_region;
    assert_eq!(
        alias_region,
        QueryRegion::parse("chr1:10-20").expect("query region again")
    );
}

#[test]
fn alias_ingest_types_match_canonical_ingest_types() {
    let alias_type_name = std::any::type_name::<AliasIngestOptions>();
    let ingest_type_name = std::any::type_name::<RuntimeIngestOptions>();
    assert_eq!(alias_type_name, ingest_type_name);
}

#[test]
fn alias_api_client_surface_matches_canonical_api_client_surface() {
    let alias_type_name = std::any::type_name::<AliasAtlasClient>();
    let canonical_type_name = std::any::type_name::<CanonicalAtlasClient>();
    assert_eq!(alias_type_name, canonical_type_name);

    let alias_config = AliasClientConfig::default();
    let canonical_config = CanonicalClientConfig::default();
    assert_eq!(alias_config.base_url, canonical_config.base_url);

    let alias_value = alias_run_with_retry(1, 0, || Ok::<_, bijux_atlas_api::ClientError>(7))
        .expect("alias retry helper");
    let canonical_value =
        canonical_run_with_retry(1, 0, || Ok::<_, bijux_atlas_api::ClientError>(7))
            .expect("canonical retry helper");
    assert_eq!(alias_value, canonical_value);
}
