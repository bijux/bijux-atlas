// SPDX-License-Identifier: Apache-2.0

use crate::adapters::AdapterError;
use crate::ports::{Fs, FsWrite, Walk};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn sorted_non_empty_lines(text: &str) -> Vec<String> {
    let mut lines = normalize_line_endings(text)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines.sort();
    lines
}

pub fn discover_repo_root(start: &Path) -> Result<PathBuf, AdapterError> {
    let mut current = start.canonicalize().map_err(|err| AdapterError::Io {
        op: "canonicalize",
        path: start.to_path_buf(),
        detail: err.to_string(),
    })?;
    loop {
        if current.join(".git").exists() || current.join("Cargo.toml").exists() {
            return Ok(current);
        }
        let Some(parent) = current.parent() else {
            return Err(AdapterError::PathViolation {
                path: start.to_path_buf(),
                detail: "unable to discover repository root from start path".to_string(),
            });
        };
        current = parent.to_path_buf();
    }
}

pub fn canonicalize_from_repo_root(repo_root: &Path, path: &Path) -> Result<PathBuf, AdapterError> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    joined.canonicalize().map_err(|err| AdapterError::Io {
        op: "canonicalize",
        path: joined,
        detail: err.to_string(),
    })
}

pub fn ensure_write_path_under_artifacts(
    repo_root: &Path,
    run_id: &str,
    target: &Path,
) -> Result<PathBuf, AdapterError> {
    let write_root = repo_root.join("artifacts").join("atlas-dev").join(run_id);
    fs::create_dir_all(&write_root).map_err(|err| AdapterError::Io {
        op: "create_dir_all",
        path: write_root.clone(),
        detail: err.to_string(),
    })?;

    let absolute_target = if target.is_absolute() {
        target.to_path_buf()
    } else {
        repo_root.join(target)
    };

    if let Some(parent) = absolute_target.parent() {
        fs::create_dir_all(parent).map_err(|err| AdapterError::Io {
            op: "create_dir_all",
            path: parent.to_path_buf(),
            detail: err.to_string(),
        })?;
    }

    let normalized_root = normalize_path(&write_root);
    let normalized_target = normalize_path(&absolute_target);

    if !normalized_target.starts_with(&normalized_root) {
        return Err(AdapterError::PathViolation {
            path: absolute_target,
            detail: format!("writes allowed only under {}", normalized_root.display()),
        });
    }
    Ok(absolute_target)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[derive(Debug, Default)]
pub struct RealFs;

impl Fs for RealFs {
    fn read_text(&self, repo_root: &Path, path: &Path) -> Result<String, AdapterError> {
        let target = canonicalize_from_repo_root(repo_root, path)?;
        let text = fs::read_to_string(&target).map_err(|err| AdapterError::Io {
            op: "read_to_string",
            path: target,
            detail: err.to_string(),
        })?;
        Ok(normalize_line_endings(&text))
    }

    fn exists(&self, repo_root: &Path, path: &Path) -> bool {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target.exists()
    }

    fn canonicalize(&self, repo_root: &Path, path: &Path) -> Result<PathBuf, AdapterError> {
        canonicalize_from_repo_root(repo_root, path)
    }
}

impl FsWrite for RealFs {
    fn write_text(
        &self,
        repo_root: &Path,
        run_id: &str,
        path: &Path,
        content: &str,
    ) -> Result<PathBuf, AdapterError> {
        let target = ensure_write_path_under_artifacts(repo_root, run_id, path)?;
        let normalized = normalize_line_endings(content);
        fs::write(&target, normalized).map_err(|err| AdapterError::Io {
            op: "write",
            path: target.clone(),
            detail: err.to_string(),
        })?;
        Ok(target)
    }
}

impl Walk for RealFs {
    fn walk_files(&self, repo_root: &Path, root: &Path) -> Result<Vec<PathBuf>, AdapterError> {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), AdapterError> {
            let entries = fs::read_dir(dir).map_err(|err| AdapterError::Io {
                op: "read_dir",
                path: dir.to_path_buf(),
                detail: err.to_string(),
            })?;
            for entry in entries {
                let entry = entry.map_err(|err| AdapterError::Io {
                    op: "read_dir_entry",
                    path: dir.to_path_buf(),
                    detail: err.to_string(),
                })?;
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out)?;
                } else {
                    out.push(path);
                }
            }
            Ok(())
        }

        let target = if root.is_absolute() {
            root.to_path_buf()
        } else {
            repo_root.join(root)
        };
        let mut out = Vec::new();
        if target.exists() {
            walk(&target, &mut out)?;
            out.sort();
        }
        Ok(out)
    }
}
