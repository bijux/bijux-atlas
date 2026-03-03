// SPDX-License-Identifier: Apache-2.0
//! Ops contracts canonical module surface.

pub mod baseline_contracts;
pub mod effect_contracts;
pub mod static_contracts;
pub mod support;

use std::path::Path;

use crate::contracts::Contract;

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    super::contracts(repo_root)
}
