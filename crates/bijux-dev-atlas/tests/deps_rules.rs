// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::{fs, path::PathBuf};

#[test]
fn internal_module_skeleton_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for rel in [
        "lib.rs",
        "model/mod.rs",
        "policies/mod.rs",
        "ports/mod.rs",
        "adapters/mod.rs",
        "core/mod.rs",
        "commands/mod.rs",
        "cli/mod.rs",
    ] {
        assert!(
            root.join(rel).exists(),
            "expected skeleton file to exist: {}",
            root.join(rel).display()
        );
    }
}

#[test]
fn model_module_does_not_depend_on_core_or_adapters() {
    let model_mod = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/model/mod.rs");
    let content = fs::read_to_string(&model_mod)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", model_mod.display()));
    for forbidden in ["crate::core", "crate::adapters"] {
        assert!(
            !content.contains(forbidden),
            "model module must not depend on {forbidden}"
        );
    }
}

#[test]
fn policies_module_does_not_depend_on_core_or_adapters() {
    let policies_mod = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/policies/mod.rs");
    let policies_schema = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/policies/schema.rs");
    let policies_validate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/policies/validate.rs");
    for path in [policies_mod, policies_schema, policies_validate] {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for forbidden in ["crate::core", "crate::adapters"] {
            assert!(
                !content.contains(forbidden),
                "policies module file {} must not depend on {forbidden}",
                path.display()
            );
        }
    }
}

#[test]
fn core_module_does_not_import_adapters() {
    let core_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/core");
    let mut stack = vec![core_root];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)
            .unwrap_or_else(|err| panic!("failed to read dir {}: {err}", path.display()))
        {
            let entry = entry.expect("dir entry");
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let content = fs::read_to_string(&entry_path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", entry_path.display()));
            assert!(
                !content.contains("crate::adapters"),
                "core source file {} must not import crate::adapters",
                entry_path.display()
            );
        }
    }
}
