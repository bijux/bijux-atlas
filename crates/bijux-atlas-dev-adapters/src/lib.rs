#![forbid(unsafe_code)]

use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

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

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterEvent {
    pub adapter: &'static str,
    pub operation: &'static str,
    pub detail: String,
}

pub trait EventLogger {
    fn log(&self, event: AdapterEvent);
}

#[derive(Debug, Default)]
pub struct NoopLogger;

impl EventLogger for NoopLogger {
    fn log(&self, _event: AdapterEvent) {}
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

#[derive(Debug, Default)]
pub struct RealFs;

impl Fs for RealFs {
    fn read_text(&self, repo_root: &Path, path: &Path) -> Result<String, AdapterError> {
        let target = canonicalize_from_repo_root(repo_root, path)?;
        fs::read_to_string(&target).map_err(|err| AdapterError::Io {
            op: "read_to_string",
            path: target,
            detail: err.to_string(),
        })
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
        fs::write(&target, content).map_err(|err| AdapterError::Io {
            op: "write",
            path: target.clone(),
            detail: err.to_string(),
        })?;
        Ok(target)
    }
}

#[derive(Debug, Default)]
pub struct RealProcessRunner;

impl ProcessRunner for RealProcessRunner {
    fn run(&self, program: &str, args: &[String], repo_root: &Path) -> Result<i32, AdapterError> {
        let status = std::process::Command::new(program)
            .args(args)
            .current_dir(repo_root)
            .status()
            .map_err(|err| AdapterError::Process {
                program: program.to_string(),
                detail: err.to_string(),
            })?;
        Ok(status.code().unwrap_or(1))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo_root() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("bijux-atlas-dev-adapters-{suffix}"));
        fs::create_dir_all(&root).expect("mkdir");
        root
    }

    #[test]
    fn write_guard_allows_only_artifacts_run_root() {
        let repo_root = temp_repo_root();
        let fs_adapter = RealFs;
        let allowed = PathBuf::from("artifacts/atlas-dev/run_one/report.json");
        let denied = PathBuf::from("ops/out.json");

        let ok = fs_adapter.write_text(&repo_root, "run_one", &allowed, "{}");
        assert!(ok.is_ok());

        let fail = fs_adapter.write_text(&repo_root, "run_one", &denied, "{}");
        assert!(matches!(fail, Err(AdapterError::PathViolation { .. })));
    }

    #[test]
    fn denied_process_runner_blocks_execution() {
        let runner = DeniedProcessRunner;
        let err = runner
            .run("echo", &[], Path::new("."))
            .expect_err("must fail");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "subprocess",
                ..
            }
        ));
    }

    #[test]
    fn denied_network_blocks_fetch() {
        let network = DeniedNetwork;
        let err = network
            .get_text("https://example.com")
            .expect_err("must fail");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "network",
                ..
            }
        ));
    }

    #[test]
    fn capabilities_from_cli_flags_maps_expected_effects() {
        let caps = Capabilities::from_cli_flags(true, false, true, false);
        assert!(caps.fs_write);
        assert!(!caps.subprocess);
        assert!(caps.git);
        assert!(!caps.network);
    }
}
