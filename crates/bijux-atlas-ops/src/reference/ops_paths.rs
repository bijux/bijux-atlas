// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpsRootError {
    Canonicalize { path: PathBuf, message: String },
}

impl OpsRootError {
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Canonicalize { path, message } => {
                format!("cannot resolve ops root {}: {message}", path.display())
            }
        }
    }
}

pub fn resolve_ops_root(
    repo_root: &Path,
    ops_root: Option<PathBuf>,
) -> Result<PathBuf, OpsRootError> {
    let path = ops_root.unwrap_or_else(|| repo_root.join("ops"));
    path.canonicalize()
        .map_err(|err| OpsRootError::Canonicalize {
            path,
            message: err.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_ops_root_uses_canonical_ops_directory_by_default() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_root = root.path().join("ops");
        std::fs::create_dir_all(&ops_root).expect("create ops root");

        let resolved = resolve_ops_root(root.path(), None).expect("resolve ops root");

        assert_eq!(
            resolved,
            ops_root.canonicalize().expect("canonical ops root")
        );
    }

    #[test]
    fn resolve_ops_root_reports_the_requested_path() {
        let root = tempfile::tempdir().expect("tempdir");
        let requested = root.path().join("custom-ops");

        let error =
            resolve_ops_root(root.path(), Some(requested.clone())).expect_err("missing root");

        assert_eq!(
            error.detail(),
            format!(
                "cannot resolve ops root {}: No such file or directory (os error 2)",
                requested.display()
            )
        );
    }
}
