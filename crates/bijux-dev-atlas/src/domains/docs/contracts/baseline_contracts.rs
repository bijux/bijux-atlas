// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use crate::contracts::Contract;

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    let _ = repo_root;
    Ok(Vec::new())
}
