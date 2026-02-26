// SPDX-License-Identifier: Apache-2.0

use crate::adapters::AdapterError;
use crate::ports::ProcessRunner;
use std::path::Path;
use std::process::Command;

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
        stdout: super::normalize_line_endings(&String::from_utf8_lossy(&output.stdout)),
        stderr: super::normalize_line_endings(&String::from_utf8_lossy(&output.stderr)),
    })
}
