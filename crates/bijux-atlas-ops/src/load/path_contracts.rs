// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn load_run_root(repo_root: &Path, run_id: &str, suite: &str) -> PathBuf {
    repo_root
        .join("artifacts/ops")
        .join(run_id)
        .join(format!("load/{suite}"))
}

#[must_use]
pub fn load_summary_path(repo_root: &Path, run_id: &str, suite: &str) -> PathBuf {
    load_run_root(repo_root, run_id, suite).join("k6-summary.json")
}

#[must_use]
pub fn load_report_path(repo_root: &Path, run_id: &str, suite: &str) -> PathBuf {
    load_run_root(repo_root, run_id, suite).join("report.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn load_path_contracts_keep_suite_outputs_together() {
        let repo_root = Path::new("/tmp/bijux-atlas");
        assert_eq!(
            load_run_root(repo_root, "atlas-run", "mixed"),
            PathBuf::from("/tmp/bijux-atlas/artifacts/ops/atlas-run/load/mixed")
        );
        assert_eq!(
            load_summary_path(repo_root, "atlas-run", "mixed"),
            PathBuf::from("/tmp/bijux-atlas/artifacts/ops/atlas-run/load/mixed/k6-summary.json")
        );
        assert_eq!(
            load_report_path(repo_root, "atlas-run", "mixed"),
            PathBuf::from("/tmp/bijux-atlas/artifacts/ops/atlas-run/load/mixed/report.json")
        );
    }
}
