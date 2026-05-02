// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
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
fn workspace_declares_core_runtime_and_dev_crates_explicitly() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("Cargo.toml")).expect("workspace Cargo.toml");
    for member in [
        "crates/bijux-atlas-core",
        "crates/bijux-atlas",
        "crates/bijux-dev-atlas",
    ] {
        assert!(
            cargo.contains(member),
            "workspace members missing required architecture crate `{member}`"
        );
    }
}

#[test]
fn core_crate_stays_runtime_independent_by_dependency_contract() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("crates/bijux-atlas-core/Cargo.toml"))
        .expect("core cargo");

    for forbidden in [
        "bijux-atlas =",
        "bijux-dev-atlas =",
        "axum =",
        "tokio =",
        "rusqlite =",
        "reqwest =",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "core crate must not depend on runtime/dev surface `{forbidden}`"
        );
    }
}

#[test]
fn domain_and_policy_layers_do_not_depend_on_adapter_or_runtime_modules() {
    let root = workspace_root().join("crates/bijux-atlas/src/domain");
    let scoped_roots = [
        root.join("dataset"),
        root.join("query"),
        root.join("policy"),
    ];
    for scope in scoped_roots {
        for file in rust_files_under(&scope) {
            let text = std::fs::read_to_string(&file)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
            for forbidden in [
                "crate::adapters::",
                "crate::runtime::",
                "crate::app::server",
            ] {
                assert!(
                    !text.contains(forbidden),
                    "domain layer file {} contains forbidden dependency `{forbidden}`",
                    file.display()
                );
            }
        }
    }
}
