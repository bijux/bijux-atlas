// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

pub struct PublishLockGuard {
    lock_path: PathBuf,
}

impl PublishLockGuard {
    pub(crate) fn new(lock_path: PathBuf) -> Self {
        Self { lock_path }
    }
}

impl Drop for PublishLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}
