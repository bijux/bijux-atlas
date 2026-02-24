// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn model_crate_has_no_store_query_or_server_dependency() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml =
        std::fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    for forbidden in [
        "bijux-atlas-store",
        "bijux-atlas-query",
        "bijux-atlas-server",
    ] {
        assert!(
            !cargo_toml.contains(forbidden),
            "forbidden dependency in model crate: {forbidden}"
        );
    }
}
