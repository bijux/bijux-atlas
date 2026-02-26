// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub use crate::ports::{AdapterError, Capabilities, Fs, FsWrite, Git, Network, ProcessRunner};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCapture {
    pub program: String,
    pub args: Vec<String>,
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprocessPolicy {
    allowed_programs: std::collections::BTreeSet<String>,
}

impl SubprocessPolicy {
    pub fn strict_default() -> Self {
        Self {
            allowed_programs: ["git", "cargo", "rustc", "bijux"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }

    pub fn allows(&self, program: &str) -> bool {
        self.allowed_programs.contains(program)
    }
}

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

#[derive(Debug, Default)]
pub struct RealProcessRunner;

impl ProcessRunner for RealProcessRunner {
    fn run(&self, program: &str, args: &[String], repo_root: &Path) -> Result<i32, AdapterError> {
        Ok(run_subprocess_captured(
            program,
            args,
            repo_root,
            &SubprocessPolicy::strict_default(),
        )?
        .status)
    }
}

pub fn run_subprocess_captured(
    program: &str,
    args: &[String],
    repo_root: &Path,
    policy: &SubprocessPolicy,
) -> Result<CommandCapture, AdapterError> {
    if !policy.allows(program) {
        return Err(AdapterError::EffectDenied {
            effect: "subprocess",
            detail: format!("program `{program}` is not in subprocess allowlist"),
        });
    }
    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|err| AdapterError::Process {
            program: program.to_string(),
            detail: err.to_string(),
        })?;
    Ok(CommandCapture {
        program: program.to_string(),
        args: args.to_vec(),
        status: output.status.code().unwrap_or(1),
        stdout: normalize_line_endings(&String::from_utf8_lossy(&output.stdout)),
        stderr: normalize_line_endings(&String::from_utf8_lossy(&output.stderr)),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo_root() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("bijux-dev-atlas-adapter-io-{suffix}"));
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

    #[test]
    fn fake_world_reads_stubbed_file() {
        let repo_root = temp_repo_root();
        let file_path = repo_root.join("docs/INDEX.md");
        let fake = FakeWorld::default().with_file(&file_path, "index");
        let text = fake
            .read_text(&repo_root, Path::new("docs/INDEX.md"))
            .expect("read");
        assert_eq!(text, "index");
    }

    #[test]
    fn subprocess_policy_blocks_non_allowlisted_programs() {
        let repo_root = temp_repo_root();
        let policy = SubprocessPolicy::strict_default();
        let err = run_subprocess_captured("python3", &[], &repo_root, &policy).expect_err("deny");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "subprocess",
                ..
            }
        ));
    }

    #[test]
    fn repo_root_discovery_has_explicit_failure_mode() {
        let repo_root = temp_repo_root();
        let nested = repo_root.join("deep/nested");
        fs::create_dir_all(&nested).expect("mkdir nested");
        let err = discover_repo_root(&nested).expect_err("must fail");
        assert!(matches!(err, AdapterError::PathViolation { .. }));
    }

    #[test]
    fn denied_network_is_default_in_real_world_bundle() {
        let world = RealWorld::new();
        let err = world
            .network
            .get_text("https://example.com")
            .expect_err("must deny");
        assert!(matches!(
            err,
            AdapterError::EffectDenied {
                effect: "network",
                ..
            }
        ));
    }
}
