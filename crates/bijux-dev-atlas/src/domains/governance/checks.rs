// SPDX-License-Identifier: Apache-2.0
//! Governance checks canonical module surface.

use std::path::Path;

use crate::registry::{CheckCatalog, CheckCatalogEntry};

pub fn checks(repo_root: &Path) -> Result<Vec<CheckCatalogEntry>, String> {
    Ok(CheckCatalog::load(repo_root)?
        .entries()
        .iter()
        .filter(|entry| entry.domain == "governance")
        .cloned()
        .collect())
}
