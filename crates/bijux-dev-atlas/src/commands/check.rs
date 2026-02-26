// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(test), allow(dead_code))]

use std::path::Path;

use crate::core::{
    explain_output, list_output, load_registry, registry_doctor, select_checks,
    RegistryDoctorReport, Selectors,
};
use crate::model::{CheckId, CheckSpec};

pub(crate) fn list_checks(
    repo_root: &Path,
    selectors: &Selectors,
) -> Result<Vec<CheckSpec>, String> {
    let registry = load_registry(repo_root)?;
    select_checks(&registry, selectors)
}

pub(crate) fn render_check_list_text(
    repo_root: &Path,
    selectors: &Selectors,
) -> Result<String, String> {
    let checks = list_checks(repo_root, selectors)?;
    Ok(list_output(&checks))
}

pub(crate) fn render_check_explain_text(
    repo_root: &Path,
    check_id: &str,
) -> Result<String, String> {
    let registry = load_registry(repo_root)?;
    let id = CheckId::parse(check_id)?;
    explain_output(&registry, &id)
}

pub(crate) fn doctor_registry(repo_root: &Path) -> RegistryDoctorReport {
    registry_doctor(repo_root)
}
