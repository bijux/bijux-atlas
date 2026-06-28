// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn atlas_stack_root_from_ops_root(ops_root: &Path) -> PathBuf {
    ops_root.join("stack")
}

#[must_use]
pub fn atlas_stack_manifest_from_repo_root(repo_root: &Path) -> PathBuf {
    repo_root.join("ops").join("stack").join("stack.toml")
}

#[must_use]
pub fn atlas_generated_version_manifest_from_repo_root(repo_root: &Path) -> PathBuf {
    repo_root
        .join("ops")
        .join("stack")
        .join("generated")
        .join("version-manifest.json")
}

#[must_use]
pub fn atlas_stack_profiles_manifest_from_ops_root(ops_root: &Path) -> PathBuf {
    atlas_stack_root_from_ops_root(ops_root).join("profiles.json")
}

#[must_use]
pub fn atlas_stack_profile_registry_from_ops_root(ops_root: &Path) -> PathBuf {
    atlas_stack_root_from_ops_root(ops_root).join("profile-registry.json")
}

#[must_use]
pub fn atlas_stack_hpa_policy_from_ops_root(ops_root: &Path) -> PathBuf {
    atlas_stack_root_from_ops_root(ops_root).join("hpa-policy.json")
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
    fn stack_path_contracts_point_to_existing_owned_paths() {
        let ops_root = repo_root().join("ops");
        for path in [
            atlas_stack_root_from_ops_root(&ops_root),
            atlas_stack_manifest_from_repo_root(&repo_root()),
            atlas_generated_version_manifest_from_repo_root(&repo_root()),
            atlas_stack_profiles_manifest_from_ops_root(&ops_root),
            atlas_stack_profile_registry_from_ops_root(&ops_root),
            atlas_stack_hpa_policy_from_ops_root(&ops_root),
        ] {
            assert!(
                path.exists(),
                "missing stack path contract: {}",
                path.display()
            );
        }
    }
}
