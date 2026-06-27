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
fn runtime_wrapper_modules_stay_reexport_only() {
    let checks = [
        (
            "src/api.rs",
            "pub use crate::compat::api::*;",
            "runtime api wrapper must forward to src/compat/api.rs",
        ),
        (
            "src/core.rs",
            "pub use crate::compat::core::*;",
            "runtime core wrapper must forward to src/compat/core.rs",
        ),
        (
            "src/model/dataset.rs",
            "pub use crate::compat::model::dataset::*;",
            "runtime dataset wrapper must forward to src/compat/model/dataset.rs",
        ),
        (
            "src/model/policy.rs",
            "pub use crate::compat::model::policy::*;",
            "runtime policy wrapper must forward to src/compat/model/policy.rs",
        ),
        (
            "src/query.rs",
            "pub use crate::compat::query::*;",
            "runtime query wrapper must forward to src/compat/query.rs",
        ),
        (
            "src/domain/ingest.rs",
            "pub use crate::compat::ingest::*;",
            "runtime ingest wrapper must forward to src/compat/ingest.rs",
        ),
    ];

    for (relative, required, context) in checks {
        let text = read_source(relative);
        assert!(text.contains(required), "{context}");
        for forbidden in ["pub struct ", "pub enum ", "pub trait ", "impl ", "pub fn "] {
            assert!(
                !text.contains(forbidden),
                "{relative} must stay a thin path-stable wrapper without `{forbidden}`"
            );
        }
    }
}

#[test]
fn compatibility_implementation_surface_stays_under_src_compat() {
    let root = crate_root();
    let expected = [
        (
            "src/compat",
            vec![
                "api.rs",
                "core.rs",
                "ingest.rs",
                "mod.rs",
                "model",
                "query.rs",
            ],
        ),
        (
            "src/compat/model",
            vec!["dataset.rs", "mod.rs", "policy.rs"],
        ),
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
            "{relative} must stay a bounded compatibility implementation directory"
        );
    }
}
