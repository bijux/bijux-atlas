// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn model_module_has_no_store_query_or_server_dependency() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/model.rs");
    let text = std::fs::read_to_string(&path).expect("read source");
    for forbidden in [
        "crate::store",
        "crate::query",
        "crate::ingest",
        "crate::artifact_validation",
        "reqwest",
        "rusqlite",
        "tokio",
    ] {
        assert!(
            !text.contains(forbidden),
            "forbidden dependency token `{forbidden}` in {}",
            path.display()
        );
    }
}
