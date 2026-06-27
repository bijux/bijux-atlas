// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn atlas_inventory_root_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("inventory")
}

#[must_use]
pub fn atlas_toolchain_inventory_from_ops_root(ops_root: &Path) -> PathBuf {
    atlas_inventory_root_from_ops_root(ops_root).join("toolchain.json")
}

#[must_use]
pub fn atlas_tools_manifest_from_repo_root(repo_root: &Path) -> PathBuf {
    repo_root.join("ops").join("inventory").join("tools.toml")
}

#[must_use]
pub fn atlas_pins_manifest_from_repo_root(repo_root: &Path) -> PathBuf {
    repo_root.join("ops").join("inventory").join("pins.yaml")
}

#[must_use]
pub fn atlas_toolchain_inventory(repo_root: &Path) -> PathBuf {
    repo_root
        .join("ops")
        .join("inventory")
        .join("toolchain.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn inventory_path_contracts_point_to_existing_owned_paths() {
        let root = repo_root();
        let ops_root = root.join("ops");
        for path in [
            atlas_inventory_root_from_ops_root(&ops_root),
            atlas_tools_manifest_from_repo_root(&root),
            atlas_pins_manifest_from_repo_root(&root),
            atlas_toolchain_inventory_from_ops_root(&ops_root),
            atlas_toolchain_inventory(&root),
        ] {
            assert!(
                path.exists(),
                "missing inventory path contract: {}",
                path.display()
            );
        }
    }
}
