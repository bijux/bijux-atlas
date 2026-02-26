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

#[cfg(test)]
mod tests {
    use super::{doctor_registry, list_checks, render_check_explain_text, render_check_list_text};
    use crate::core::Selectors;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = match crate_dir.parent() {
            Some(path) => path,
            None => panic!("workspace directory missing"),
        };
        let repo_dir = match workspace_dir.parent() {
            Some(path) => path,
            None => panic!("repo directory missing"),
        };
        repo_dir.to_path_buf()
    }

    #[test]
    fn check_helpers_produce_deterministic_text_outputs() {
        let root = repo_root();
        let selectors = Selectors::default();
        let list_rows = list_checks(&root, &selectors);
        assert!(list_rows.is_ok(), "list_checks failed");
        let list_rows = list_rows.unwrap_or_default();
        assert!(!list_rows.is_empty(), "registry should list checks");

        let rendered_list = render_check_list_text(&root, &selectors);
        assert!(rendered_list.is_ok(), "render_check_list_text failed");
        let rendered_list = rendered_list.unwrap_or_default();
        assert!(
            rendered_list.lines().next().is_some(),
            "list rendering must include at least one row"
        );

        let first_id = list_rows[0].id.as_str().to_string();
        let explained = render_check_explain_text(&root, &first_id);
        assert!(explained.is_ok(), "render_check_explain_text failed");
        let explained = explained.unwrap_or_default();
        assert!(explained.contains("id: "), "explain output missing id line");
        assert!(explained.contains("effects_required:"), "explain output missing effects");
    }

    #[test]
    fn registry_doctor_helper_executes() {
        let root = repo_root();
        let report = doctor_registry(&root);
        assert!(
            report.errors.iter().all(|err| !err.trim().is_empty()),
            "registry doctor errors must be non-empty strings when present"
        );
    }
}
