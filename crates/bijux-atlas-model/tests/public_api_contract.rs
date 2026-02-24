// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

#[test]
fn public_api_doc_matches_export_surface() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_doc = fs::read_to_string(manifest_dir.join("docs/public-api.md"))
        .expect("read docs/public-api.md");
    let lib_rs = fs::read_to_string(manifest_dir.join("src/lib.rs")).expect("read src/lib.rs");

    for item in [
        "CRATE_NAME",
        "DatasetId",
        "DatasetSelector",
        "ModelVersion",
        "GeneId",
        "SeqId",
        "ParseError",
        "ArtifactManifest",
        "Catalog",
        "ShardId",
        "OptionalFieldPolicy",
        "StrictnessMode",
        "GeneIdentifierPolicy",
    ] {
        assert!(
            api_doc.contains(item),
            "docs/PUBLIC_API missing item: {item}"
        );
    }

    for token in [
        "pub use dataset::{",
        "pub use gene::{",
        "pub use manifest::{",
        "pub use policy::{",
        "pub const CRATE_NAME",
    ] {
        assert!(
            lib_rs.contains(token),
            "src/lib.rs missing export token: {token}"
        );
    }
}
