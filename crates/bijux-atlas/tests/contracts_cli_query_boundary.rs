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
fn cli_sources_depend_on_app_query_boundary_not_domain_query_path() {
    let cli_root = crate_root().join("src/adapters/inbound/cli");
    for file in rust_files_under(&cli_root) {
        let rel = file
            .strip_prefix(crate_root().join("src/adapters/inbound/cli"))
            .expect("cli-relative path");
        let rel_text = rel.to_string_lossy();
        if rel_text.starts_with("operations/")
            || rel_text == "output.rs"
            || rel_text == "ingest_inputs.rs"
        {
            continue;
        }
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        assert!(
            !text.contains("crate::domain::query::"),
            "cli source must depend on app query boundary: {}",
            file.display()
        );
    }
}

#[test]
fn cli_entrypoints_do_not_import_server_runtime_or_policy_loaders() {
    let cli_root = crate_root().join("src/adapters/inbound/cli");
    for file in rust_files_under(&cli_root) {
        let rel = file
            .strip_prefix(crate_root().join("src/adapters/inbound/cli"))
            .expect("cli-relative path");
        let rel_text = rel.to_string_lossy();
        if rel_text.starts_with("operations/")
            || rel_text == "output.rs"
            || rel_text == "ingest_inputs.rs"
        {
            continue;
        }
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        for forbidden in ["crate::app::server", "load_policy_from_workspace"] {
            assert!(
                !text.contains(forbidden),
                "cli entrypoint must not own server/policy runtime behavior (`{forbidden}`): {}",
                file.display()
            );
        }
    }
}
