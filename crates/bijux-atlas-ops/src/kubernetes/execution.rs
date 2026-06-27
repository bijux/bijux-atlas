// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SubprocessCapture {
    pub stdout: String,
    pub event: Value,
}

pub trait KubernetesCommandRunner {
    fn run(&self, binary: &str, args: &[String], cwd: &Path) -> Result<SubprocessCapture, String>;
}
