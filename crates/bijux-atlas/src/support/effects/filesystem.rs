// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)] // ATLAS-EXC-0001

use std::path::Path;

use crate::CacheError;

pub(crate) fn read(path: &Path) -> Result<Vec<u8>, CacheError> {
    std::fs::read(path).map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn write(path: &Path, bytes: &[u8]) -> Result<(), CacheError> {
    std::fs::write(path, bytes).map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn create_dir_all(path: &Path) -> Result<(), CacheError> {
    std::fs::create_dir_all(path).map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn rename(src: &Path, dst: &Path) -> Result<(), CacheError> {
    std::fs::rename(src, dst).map_err(|e| CacheError(e.to_string()))
}
