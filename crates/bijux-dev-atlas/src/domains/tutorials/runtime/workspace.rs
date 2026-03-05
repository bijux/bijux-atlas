// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TutorialWorkspaceManager {
    root: PathBuf,
}

impl TutorialWorkspaceManager {
    pub fn new(repo_root: &Path) -> Self {
        Self {
            root: repo_root.join("artifacts/tutorials/workspace"),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn ensure(&self) -> Result<(), String> {
        fs::create_dir_all(&self.root)
            .map_err(|err| format!("failed to create workspace {}: {err}", self.root.display()))
    }

    pub fn safe_cleanup(&self, target: &Path, dry_run: bool) -> Result<bool, String> {
        let canonical_root = fs::canonicalize(&self.root).unwrap_or_else(|_| self.root.clone());
        let canonical_target = fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
        if !canonical_target.starts_with(&canonical_root) {
            return Err(format!(
                "refusing to delete non-workspace path: {} (workspace root: {})",
                target.display(),
                self.root.display()
            ));
        }
        if dry_run {
            return Ok(false);
        }
        if target.exists() {
            fs::remove_dir_all(target)
                .map_err(|err| format!("failed to remove {}: {err}", target.display()))?;
            return Ok(true);
        }
        Ok(false)
    }
}
