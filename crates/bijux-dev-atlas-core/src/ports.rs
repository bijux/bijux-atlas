// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    EffectDenied {
        effect: &'static str,
        detail: String,
    },
    PathViolation {
        path: PathBuf,
        detail: String,
    },
    Io {
        op: &'static str,
        path: PathBuf,
        detail: String,
    },
    Process {
        program: String,
        detail: String,
    },
    Git {
        detail: String,
    },
    Network {
        detail: String,
    },
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EffectDenied { effect, detail } => {
                write!(f, "effect denied: {effect} ({detail})")
            }
            Self::PathViolation { path, detail } => {
                write!(f, "path violation: {} ({detail})", path.display())
            }
            Self::Io { op, path, detail } => {
                write!(f, "io error: {op} {} ({detail})", path.display())
            }
            Self::Process { program, detail } => write!(f, "process error: {program} ({detail})"),
            Self::Git { detail } => write!(f, "git error: {detail}"),
            Self::Network { detail } => write!(f, "network error: {detail}"),
        }
    }
}

impl std::error::Error for AdapterError {}

pub trait Fs {
    fn read_text(&self, repo_root: &Path, path: &Path) -> Result<String, AdapterError>;
    fn exists(&self, repo_root: &Path, path: &Path) -> bool;
    fn canonicalize(&self, repo_root: &Path, path: &Path) -> Result<PathBuf, AdapterError>;
}

pub trait FsWrite {
    fn write_text(
        &self,
        repo_root: &Path,
        run_id: &str,
        path: &Path,
        content: &str,
    ) -> Result<PathBuf, AdapterError>;
}

pub trait ProcessRunner {
    fn run(&self, program: &str, args: &[String], repo_root: &Path) -> Result<i32, AdapterError>;
}

pub trait Git {
    fn tracked_files(&self, repo_root: &Path) -> Result<Vec<String>, AdapterError>;
}

pub trait Network {
    fn get_text(&self, url: &str) -> Result<String, AdapterError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub fs_write: bool,
    pub subprocess: bool,
    pub git: bool,
    pub network: bool,
}

impl Capabilities {
    pub fn deny_all() -> Self {
        Self {
            fs_write: false,
            subprocess: false,
            git: false,
            network: false,
        }
    }

    pub fn from_cli_flags(
        allow_fs_write: bool,
        allow_subprocess: bool,
        allow_git: bool,
        allow_network: bool,
    ) -> Self {
        Self {
            fs_write: allow_fs_write,
            subprocess: allow_subprocess,
            git: allow_git,
            network: allow_network,
        }
    }
}
