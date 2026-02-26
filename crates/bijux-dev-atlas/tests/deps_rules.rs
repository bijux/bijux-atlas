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
