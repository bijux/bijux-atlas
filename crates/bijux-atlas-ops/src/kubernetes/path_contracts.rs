// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn atlas_ops_root(repo_root: &Path) -> PathBuf {
    repo_root.join("ops")
}

#[must_use]
pub fn atlas_chart_dir_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("k8s/charts/bijux-atlas")
}

#[must_use]
pub fn atlas_chart_dir(repo_root: &Path) -> PathBuf {
    atlas_chart_dir_from_ops_root(&atlas_ops_root(repo_root))
}

#[must_use]
pub fn atlas_values_root_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("k8s/values")
}

#[must_use]
pub fn atlas_values_root(repo_root: &Path) -> PathBuf {
    atlas_values_root_from_ops_root(&atlas_ops_root(repo_root))
}

#[must_use]
pub fn atlas_values_schema_from_ops_root(ops_root: &Path) -> PathBuf {
    atlas_chart_dir_from_ops_root(ops_root).join("values.schema.json")
}

#[must_use]
pub fn atlas_values_schema(repo_root: &Path) -> PathBuf {
    atlas_values_schema_from_ops_root(&atlas_ops_root(repo_root))
}

#[must_use]
pub fn atlas_values_file_from_ops_root(ops_root: &Path) -> PathBuf {
    atlas_chart_dir_from_ops_root(ops_root).join("values.yaml")
}

#[must_use]
pub fn atlas_toolchain_inventory_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("inventory/toolchain.json")
}

#[must_use]
pub fn atlas_toolchain_inventory(repo_root: &Path) -> PathBuf {
    atlas_toolchain_inventory_from_ops_root(&atlas_ops_root(repo_root))
}

#[must_use]
pub fn atlas_dataset_manifest_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("datasets/manifest.json")
}

#[must_use]
pub fn atlas_dataset_manifest(repo_root: &Path) -> PathBuf {
    atlas_dataset_manifest_from_ops_root(&atlas_ops_root(repo_root))
}

#[must_use]
pub fn atlas_install_matrix_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("k8s/install-matrix.json")
}

#[must_use]
pub fn atlas_install_matrix(repo_root: &Path) -> PathBuf {
    atlas_install_matrix_from_ops_root(&atlas_ops_root(repo_root))
}

#[must_use]
pub fn atlas_rollout_safety_contract_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("k8s/rollout-safety-contract.json")
}

#[must_use]
pub fn atlas_rollout_safety_contract(repo_root: &Path) -> PathBuf {
    atlas_rollout_safety_contract_from_ops_root(&atlas_ops_root(repo_root))
}

#[must_use]
pub fn atlas_hpa_policy_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("stack/hpa-policy.json")
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
    fn kubernetes_path_contracts_point_to_existing_owned_paths() {
        let root = repo_root();
        for path in [
            atlas_ops_root(&root),
            atlas_chart_dir(&root),
            atlas_values_root(&root),
            atlas_values_schema(&root),
            atlas_values_file_from_ops_root(&atlas_ops_root(&root)),
            atlas_toolchain_inventory(&root),
            atlas_dataset_manifest(&root),
            atlas_install_matrix(&root),
            atlas_rollout_safety_contract(&root),
            atlas_hpa_policy_from_ops_root(&atlas_ops_root(&root)),
        ] {
            assert!(
                path.exists(),
                "missing kubernetes path contract: {}",
                path.display()
            );
        }
    }
}
