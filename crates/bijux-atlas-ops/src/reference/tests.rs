// SPDX-License-Identifier: Apache-2.0

use super::ops_paths::resolve_ops_root;

#[test]
fn owned_reference_surface_exposes_ops_root_resolution() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(root.path().join("ops")).expect("create ops root");

    let resolved = resolve_ops_root(root.path(), None).expect("resolve ops root");

    assert!(resolved.ends_with("ops"));
}
