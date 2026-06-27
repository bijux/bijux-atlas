// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn read_workspace_source(relative: &str) -> String {
    let path = workspace_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn rust_files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out
}

#[test]
fn alias_leaf_forwarders_bind_directly_to_owning_crates() {
    let checks = [
        (
            "crates/bijux-atlas/src/api.rs",
            "pub use bijux_atlas_api::*;",
            "atlas alias api forwarding surface must bind directly to bijux-atlas-api",
        ),
        (
            "crates/bijux-atlas/src/query.rs",
            "pub use bijux_atlas_query::*;",
            "atlas alias query forwarding surface must bind directly to bijux-atlas-query",
        ),
        (
            "crates/bijux-atlas/src/domain/ingest.rs",
            "pub use bijux_atlas_ingest::*;",
            "atlas alias ingest forwarding surface must bind directly to bijux-atlas-ingest",
        ),
    ];

    for (relative, required, context) in checks {
        let text = read_workspace_source(relative);
        assert!(text.contains(required), "{context}");
        for forbidden in ["pub struct ", "pub enum ", "pub trait ", "impl ", "pub fn "] {
            assert!(
                !text.contains(forbidden),
                "{relative} must stay a thin compatibility wrapper without `{forbidden}`"
            );
        }
    }
}

#[test]
fn runtime_compatibility_implementation_surface_stays_bounded() {
    let root = crate_root();
    let expected = [("src/compat", vec!["core.rs", "mod.rs"])];

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

#[test]
fn runtime_leaf_forwarders_do_not_reappear() {
    for relative in ["src/api.rs", "src/query.rs", "src/domain/ingest.rs"] {
        assert!(
            !crate_root().join(relative).exists(),
            "runtime must not own a compatibility wrapper at {relative}"
        );
    }
}

#[test]
fn runtime_internals_use_owning_crates_not_path_stable_wrappers() {
    let root = crate_root().join("src");
    let forbidden = ["crate::api::", "crate::domain::ingest::", "crate::query::"];

    for path in rust_files_under(&root) {
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for token in forbidden {
            assert!(
                !text.contains(token),
                "runtime internal file must use owning crate paths instead of wrapper `{token}`: {}",
                rel
            );
        }
    }
}
