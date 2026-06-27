// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(relative: &str) -> String {
    let path = crate_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

#[test]
fn runtime_model_query_and_ingest_facades_stay_reexport_only() {
    let checks = [
        (
            "src/model/dataset.rs",
            "pub use bijux_atlas_model::dataset::{",
            "runtime dataset facade must forward to bijux-atlas-model",
        ),
        (
            "src/model/policy.rs",
            "pub use bijux_atlas_model::policy::*;",
            "runtime policy facade must forward to bijux-atlas-model",
        ),
        (
            "src/query/mod.rs",
            "pub use bijux_atlas_query::*;",
            "runtime query facade must forward to bijux-atlas-query",
        ),
        (
            "src/domain/ingest/mod.rs",
            "pub use bijux_atlas_ingest::*;",
            "runtime ingest facade must forward to bijux-atlas-ingest",
        ),
    ];

    for (relative, required, context) in checks {
        let text = read_source(relative);
        assert!(text.contains(required), "{context}");
        for forbidden in ["pub struct ", "pub enum ", "pub trait ", "impl ", "pub fn "] {
            assert!(
                !text.contains(forbidden),
                "{relative} must stay a thin compatibility facade without `{forbidden}`"
            );
        }
    }
}

#[test]
fn compatibility_facade_directories_do_not_regrow_hidden_impl_files() {
    let root = crate_root();
    let expected = [
        ("src/domain/ingest", vec!["mod.rs"]),
        ("src/query", vec!["mod.rs"]),
        ("src/model", vec!["dataset.rs", "mod.rs", "policy.rs"]),
    ];

    for (relative, allowed) in expected {
        let dir = root.join(relative);
        let mut names = std::fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
            .map(|entry| {
                entry
                    .unwrap_or_else(|err| panic!("failed to read {} entry: {err}", dir.display()))
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|_| panic!("non-utf8 entry under {}", dir.display()))
            })
            .collect::<Vec<_>>();
        names.sort();
        let mut expected_names = allowed;
        expected_names.sort();
        assert_eq!(
            names, expected_names,
            "{relative} must stay a bounded compatibility facade directory"
        );
    }
}
