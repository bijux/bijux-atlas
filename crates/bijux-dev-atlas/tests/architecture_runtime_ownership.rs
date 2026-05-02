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
fn runtime_ownership_boundary_document_exists() {
    let path = crate_root().join("docs_runtime_ownership.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    for required in [
        "Forbidden ownership",
        "Allowed ownership",
        "ingest normalization",
        "query planning",
        "server route behavior",
    ] {
        assert!(
            text.contains(required),
            "runtime ownership document missing `{required}`"
        );
    }
}

#[test]
fn dev_atlas_source_does_not_implement_runtime_ingest_or_query_semantics() {
    let root = crate_root().join("src");
    let forbidden = [
        "crate::domain::ingest",
        "bijux_atlas::domain::ingest",
        "query_genes(",
        "parse_gene_query_request(",
        "plan_gene_query(",
        "build_router(",
        "axum::Router",
    ];

    for file in rust_files_under(&root) {
        let content = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        for token in forbidden {
            assert!(
                !content.contains(token),
                "dev-atlas must not own runtime behavior token `{token}` in {}",
                file.display()
            );
        }
    }
}
