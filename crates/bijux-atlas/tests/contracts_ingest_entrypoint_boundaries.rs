// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
fn cli_entrypoints_depend_on_app_ingest_boundary_not_domain_ingest() {
    let cli_root = crate_root().join("src/adapters/inbound/cli");
    for file in rust_files_under(&cli_root) {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        assert!(
            !text.contains("crate::domain::ingest::"),
            "cli entrypoint file must not directly import domain ingest: {}",
            file.display()
        );
    }
}

#[test]
fn ingest_facade_owns_entrypoint_bridge() {
    let path = crate_root().join("src/app/ingest/mod.rs");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    for required in [
        "pub fn ingest_dataset",
        "pub fn replay_normalized_counts",
        "pub fn diff_normalized_ids",
    ] {
        assert!(
            text.contains(required),
            "ingest facade missing required bridge `{required}`"
        );
    }
}
