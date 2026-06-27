// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::adapters::inbound::client::{run_with_retry, AtlasClient, ClientConfig};

#[test]
fn legacy_client_import_path_remains_constructible() {
    let _ = std::mem::size_of::<Option<AtlasClient>>();
    let config = ClientConfig::default();
    assert!(config.base_url.starts_with("http://"));
}

#[test]
fn legacy_client_import_path_reexports_retry_helper() {
    let value = run_with_retry(1, 0, || Ok::<_, bijux_atlas_api::ClientError>(42))
        .expect("compat retry helper");
    assert_eq!(value, 42);
}
