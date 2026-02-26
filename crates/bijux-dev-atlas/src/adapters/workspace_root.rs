// SPDX-License-Identifier: Apache-2.0

use crate::adapters::{discover_repo_root, AdapterError};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRoot(PathBuf);

impl WorkspaceRoot {
    pub fn discover_from(start: &Path) -> Result<Self, AdapterError> {
        discover_repo_root(start).map(Self)
    }

    pub fn from_explicit_path(path: PathBuf) -> Result<Self, AdapterError> {
        path.canonicalize()
            .map(Self)
            .map_err(|err| AdapterError::Io {
                op: "canonicalize",
                path,
                detail: err.to_string(),
            })
    }

    pub fn from_cli_or_cwd(arg: Option<PathBuf>) -> Result<Self, AdapterError> {
        match arg {
            Some(path) => Self::from_explicit_path(path),
            None => {
                let cwd = std::env::current_dir().map_err(|err| AdapterError::Io {
                    op: "current_dir",
                    path: PathBuf::from("."),
                    detail: err.to_string(),
                })?;
                Self::discover_from(&cwd)
            }
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn into_inner(self) -> PathBuf {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceRoot;
    use crate::adapters::AdapterError;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let suffix = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_nanos(),
            Err(_) => 0,
        };
        let root = std::env::temp_dir().join(format!("bijux-dev-atlas-workspace-root-{suffix}"));
        let _ = fs::create_dir_all(&root);
        root
    }

    #[test]
    fn workspace_root_discovers_repo_from_nested_path() {
        let repo = temp_root();
        let nested = repo.join("a/b/c");
        let _ = fs::write(repo.join("Cargo.toml"), "[workspace]\nmembers=[]\n");
        let _ = fs::create_dir_all(&nested);

        let resolved = WorkspaceRoot::discover_from(&nested);
        assert!(resolved.is_ok(), "workspace root discovery failed");
        let resolved = match resolved {
            Ok(v) => v,
            Err(_) => return,
        };
        let expected = match repo.canonicalize() {
            Ok(v) => v,
            Err(_) => panic!("expected temp repo to canonicalize"),
        };
        assert_eq!(resolved.as_path(), expected.as_path());
    }

    #[test]
    fn workspace_root_returns_explicit_error_when_marker_missing() {
        let root = temp_root();
        let nested = root.join("x/y");
        let _ = fs::create_dir_all(&nested);
        let err = WorkspaceRoot::discover_from(&nested);
        assert!(matches!(err, Err(AdapterError::PathViolation { .. })));
    }

    #[test]
    fn explicit_workspace_root_path_is_not_promoted_to_repo_root() {
        let repo = temp_root();
        let nested_crate = repo.join("crates").join("sample");
        let _ = fs::write(repo.join("Cargo.toml"), "[workspace]\nmembers=[]\n");
        let _ = fs::write(repo.join(".git"), "not-a-real-git-dir-marker-for-test");
        let _ = fs::create_dir_all(&nested_crate);
        let _ = fs::write(nested_crate.join("Cargo.toml"), "[package]\nname=\"sample\"\nversion=\"0.1.0\"\n");

        let resolved = WorkspaceRoot::from_explicit_path(nested_crate.clone());
        assert!(resolved.is_ok(), "explicit workspace root canonicalization failed");
        let resolved = match resolved {
            Ok(v) => v,
            Err(_) => return,
        };
        let expected = match nested_crate.canonicalize() {
            Ok(v) => v,
            Err(_) => panic!("expected explicit path to canonicalize"),
        };
        assert_eq!(resolved.as_path(), expected.as_path());
    }
}
