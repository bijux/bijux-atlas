// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn model_module_has_no_store_query_or_server_dependency() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/model");
    for entry in std::fs::read_dir(src_root).expect("read src/model") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
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
}
