// SPDX-License-Identifier: Apache-2.0
//! Tutorials checks canonical module surface.

pub mod baseline_checks;
pub mod effect_checks;
pub mod static_checks;

use std::path::Path;

use crate::registry::{CheckCatalog, CheckCatalogEntry};

pub fn checks(repo_root: &Path) -> Result<Vec<CheckCatalogEntry>, String> {
    Ok(CheckCatalog::load(repo_root)?
        .entries()
        .iter()
        .filter(|entry| entry.domain == "tutorials")
        .cloned()
        .collect())
}
