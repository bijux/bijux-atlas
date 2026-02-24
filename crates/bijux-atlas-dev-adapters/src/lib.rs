#![forbid(unsafe_code)]

use std::path::Path;

pub trait ProcessAdapter {
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<i32, String>;
}

#[derive(Debug, Default)]
pub struct StdProcessAdapter;

impl ProcessAdapter for StdProcessAdapter {
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<i32, String> {
        let status = std::process::Command::new(program)
            .args(args)
            .current_dir(cwd)
            .status()
            .map_err(|err| format!("failed to execute `{program}`: {err}"))?;
        Ok(status.code().unwrap_or(1))
    }
}
