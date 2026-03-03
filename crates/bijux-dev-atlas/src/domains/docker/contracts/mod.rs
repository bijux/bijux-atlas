// SPDX-License-Identifier: Apache-2.0
//! Docker contracts canonical module surface.

pub mod baseline_contracts;
pub mod effect_contracts;
pub mod static_contracts;

use std::path::Path;

use crate::contracts::Contract;

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    super::contracts(repo_root)
}
