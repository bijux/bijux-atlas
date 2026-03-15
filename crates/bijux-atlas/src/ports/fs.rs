// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use crate::contracts::errors::Result;

pub trait FsPort {
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn write_all(&self, path: &Path, data: &[u8]) -> Result<()>;
    fn exists(&self, path: &Path) -> Result<bool>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
}
