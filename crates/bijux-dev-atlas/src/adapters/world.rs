// SPDX-License-Identifier: Apache-2.0

use crate::adapters::{AdapterError, RealFs, RealProcessRunner};
use crate::ports::{Fs, FsWrite, Git, Network, ProcessRunner};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct RealGit;

impl Git for RealGit {
    fn tracked_files(&self, repo_root: &Path) -> Result<Vec<String>, AdapterError> {
        let output = std::process::Command::new("git")
            .args(["ls-files"])
            .current_dir(repo_root)
            .output()
            .map_err(|err| AdapterError::Git {
                detail: err.to_string(),
            })?;
        if !output.status.success() {
            return Err(AdapterError::Git {
                detail: format!("git ls-files exited with {}", output.status),
            });
        }
        let text = String::from_utf8(output.stdout).map_err(|err| AdapterError::Git {
            detail: err.to_string(),
        })?;
        Ok(text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect())
    }
}

#[derive(Debug, Default)]
pub struct DeniedProcessRunner;

impl ProcessRunner for DeniedProcessRunner {
    fn run(&self, program: &str, _args: &[String], _repo_root: &Path) -> Result<i32, AdapterError> {
        Err(AdapterError::EffectDenied {
            effect: "subprocess",
            detail: format!("attempted to execute `{program}`"),
        })
    }
}

#[derive(Debug, Default)]
pub struct DeniedNetwork;

impl Network for DeniedNetwork {
    fn get_text(&self, url: &str) -> Result<String, AdapterError> {
        Err(AdapterError::EffectDenied {
            effect: "network",
            detail: format!("attempted to fetch `{url}`"),
        })
    }
}

#[derive(Debug, Default)]
pub struct RealWorld {
    pub fs: RealFs,
    pub process: RealProcessRunner,
    pub git: RealGit,
    pub network: DeniedNetwork,
}

impl RealWorld {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default)]
pub struct FakeWorld {
    files: std::collections::BTreeMap<PathBuf, String>,
    commands: std::collections::BTreeMap<(String, Vec<String>), i32>,
}

impl FakeWorld {
    pub fn with_file(mut self, path: impl Into<PathBuf>, text: impl Into<String>) -> Self {
        self.files.insert(path.into(), text.into());
        self
    }

    pub fn with_command_status(mut self, program: &str, args: &[String], status: i32) -> Self {
        self.commands
            .insert((program.to_string(), args.to_vec()), status);
        self
    }
}

impl Fs for FakeWorld {
    fn read_text(&self, repo_root: &Path, path: &Path) -> Result<String, AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        self.files.get(&target).cloned().ok_or(AdapterError::Io {
            op: "read_text",
            path: target,
            detail: "file not present in FakeWorld".to_string(),
        })
    }

    fn exists(&self, repo_root: &Path, path: &Path) -> bool {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        self.files.contains_key(&target)
    }

    fn canonicalize(&self, repo_root: &Path, path: &Path) -> Result<PathBuf, AdapterError> {
        Ok(if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        })
    }
}

impl FsWrite for FakeWorld {
    fn write_text(
        &self,
        _repo_root: &Path,
        _run_id: &str,
        _path: &Path,
        _content: &str,
    ) -> Result<PathBuf, AdapterError> {
        Err(AdapterError::EffectDenied {
            effect: "fs_write",
            detail: "FakeWorld write requires mutable store plumbing".to_string(),
        })
    }
}

impl ProcessRunner for FakeWorld {
    fn run(&self, program: &str, args: &[String], _repo_root: &Path) -> Result<i32, AdapterError> {
        self.commands
            .get(&(program.to_string(), args.to_vec()))
            .copied()
            .ok_or(AdapterError::Process {
                program: program.to_string(),
                detail: "command not stubbed in FakeWorld".to_string(),
            })
    }
}
