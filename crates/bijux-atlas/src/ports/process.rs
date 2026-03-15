// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use crate::errors::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResult {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub trait ProcessPort {
    fn run(&self, program: &str, args: &[String], cwd: Option<&PathBuf>) -> Result<ProcessResult>;
}
