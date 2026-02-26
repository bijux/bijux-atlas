// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

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

