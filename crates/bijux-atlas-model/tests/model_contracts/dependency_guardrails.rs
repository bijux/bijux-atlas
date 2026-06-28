// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn model_crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn removed_model_root_does_not_reappear() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("crates/bijux-atlas-runtime/src/model.rs");
    assert!(
        !path.exists(),
        "legacy model root must stay removed: {}",
        path.display()
    );
}

#[test]
fn canonical_model_types_stay_free_of_runtime_dependencies() {
    let manifest_dir = model_crate_dir();
    for path in [
        manifest_dir.join("src/dataset/keys.rs"),
        manifest_dir.join("src/dataset/manifest.rs"),
        manifest_dir.join("src/dataset/version.rs"),
        manifest_dir.join("src/diff/mod.rs"),
        manifest_dir.join("src/gene/mod.rs"),
        manifest_dir.join("src/policy.rs"),
        manifest_dir.join("src/query.rs"),
    ] {
        let text = std::fs::read_to_string(&path).expect("read source");
        for forbidden in ["reqwest", "rusqlite", "tokio", "axum", "hyper"] {
            assert!(
                !text.contains(forbidden),
                "forbidden dependency token `{forbidden}` in {}",
                path.display()
            );
        }
    }
}
