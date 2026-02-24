use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::errors::Result;

pub trait FsPort {
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn write_all(&self, path: &Path, data: &[u8]) -> Result<()>;
    fn exists(&self, path: &Path) -> Result<bool>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
}

pub trait ClockPort {
    fn now(&self) -> Result<SystemTime>;
    fn sleep(&self, duration: Duration) -> Result<()>;
}

pub trait NetPort {
    fn get_bytes(&self, url: &str) -> Result<Vec<u8>>;
    fn post_json(&self, url: &str, body: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResult {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub trait ProcessPort {
    fn run(&self, program: &str, args: &[String], cwd: Option<&PathBuf>) -> Result<ProcessResult>;
}
