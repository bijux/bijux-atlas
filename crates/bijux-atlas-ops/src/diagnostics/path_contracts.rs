// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn diagnose_root(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/ops/diagnose")
}

#[must_use]
pub fn diagnose_run_root(repo_root: &Path, run_id: &str) -> PathBuf {
    diagnose_root(repo_root).join(run_id)
}

#[must_use]
pub fn diagnose_bundle_path(repo_root: &Path, run_id: &str) -> PathBuf {
    diagnose_run_root(repo_root, run_id).join("bundle.json")
}

#[must_use]
pub fn diagnose_redacted_bundle_path(bundle_path: &Path) -> PathBuf {
    bundle_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("bundle.redacted.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnose_paths_stay_under_the_owned_artifacts_root() {
        let repo_root = Path::new("/tmp/bijux-atlas");
        assert_eq!(
            diagnose_root(repo_root),
            PathBuf::from("/tmp/bijux-atlas/artifacts/ops/diagnose")
        );
        assert_eq!(
            diagnose_run_root(repo_root, "atlas-run"),
            PathBuf::from("/tmp/bijux-atlas/artifacts/ops/diagnose/atlas-run")
        );
        assert_eq!(
            diagnose_bundle_path(repo_root, "atlas-run"),
            PathBuf::from("/tmp/bijux-atlas/artifacts/ops/diagnose/atlas-run/bundle.json")
        );
    }

    #[test]
    fn redacted_bundle_path_stays_beside_the_source_bundle() {
        let source = Path::new("/tmp/bijux-atlas/artifacts/ops/diagnose/atlas-run/bundle.json");
        assert_eq!(
            diagnose_redacted_bundle_path(source),
            PathBuf::from("/tmp/bijux-atlas/artifacts/ops/diagnose/atlas-run/bundle.redacted.json")
        );
    }
}
