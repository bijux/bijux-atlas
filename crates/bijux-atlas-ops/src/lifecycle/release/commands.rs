// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::SubprocessCapture;
use crate::lifecycle::evidence::support::sha256_file;
use crate::lifecycle::simulation::paths::simulation_current_chart_path;
use std::path::{Path, PathBuf};

pub trait ReleaseCommandRunner {
    fn run(&self, binary: &str, args: &[String], cwd: &Path) -> Result<SubprocessCapture, String>;
}

pub fn helm_release_manifest(
    runner: &impl ReleaseCommandRunner,
    repo_root: &Path,
    namespace: &str,
) -> Result<String, String> {
    let argv = vec![
        "get".to_string(),
        "manifest".to_string(),
        "bijux-atlas".to_string(),
        "--namespace".to_string(),
        namespace.to_string(),
    ];
    runner
        .run("helm", &argv, repo_root)
        .map(|capture| capture.stdout)
}

pub fn prior_release_revision(
    runner: &impl ReleaseCommandRunner,
    repo_root: &Path,
    namespace: &str,
) -> Result<String, String> {
    let argv = vec![
        "history".to_string(),
        "bijux-atlas".to_string(),
        "--namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let stdout = runner.run("helm", &argv, repo_root)?.stdout;
    parse_prior_release_revision(&stdout)
}

pub fn package_chart_for_evidence(
    runner: &impl ReleaseCommandRunner,
    repo_root: &Path,
) -> Result<PathBuf, String> {
    let evidence_root = repo_root.join("ops/release/evidence");
    let package_dir = evidence_root.join("packages");
    std::fs::create_dir_all(&package_dir)
        .map_err(|err| format!("failed to create {}: {err}", package_dir.display()))?;
    let chart_path = simulation_current_chart_path(repo_root);
    let argv = vec![
        "package".to_string(),
        chart_path.display().to_string(),
        "--destination".to_string(),
        package_dir.display().to_string(),
    ];
    runner.run("helm", &argv, repo_root)?;
    latest_packaged_chart(&package_dir)
}

pub fn latest_packaged_chart(package_dir: &Path) -> Result<PathBuf, String> {
    let mut packages = std::fs::read_dir(package_dir)
        .map_err(|err| format!("failed to read {}: {err}", package_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tgz"))
        .collect::<Vec<_>>();
    packages.sort();
    packages
        .pop()
        .ok_or_else(|| format!("no chart package produced in {}", package_dir.display()))
}

pub fn release_package_inventory(
    repo_root: &Path,
    path: &Path,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "path": path.strip_prefix(repo_root).unwrap_or(path).display().to_string(),
        "sha256": sha256_file(path)?
    }))
}

pub fn parse_prior_release_revision(history_json: &str) -> Result<String, String> {
    let rows: serde_json::Value = serde_json::from_str(history_json)
        .map_err(|err| format!("failed to parse helm history: {err}"))?;
    let history = rows
        .as_array()
        .ok_or_else(|| "helm history payload must be an array".to_string())?;
    if history.len() < 2 {
        return Err("rollback requires at least two release revisions".to_string());
    }
    history
        .get(history.len() - 2)
        .and_then(|row| row.get("revision"))
        .and_then(serde_json::Value::as_i64)
        .map(|revision| revision.to_string())
        .ok_or_else(|| "helm history did not contain a usable previous revision".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockRunner {
        calls: RefCell<Vec<(String, Vec<String>, PathBuf)>>,
        results: RefCell<VecDeque<Result<SubprocessCapture, String>>>,
    }

    impl ReleaseCommandRunner for MockRunner {
        fn run(
            &self,
            binary: &str,
            args: &[String],
            cwd: &Path,
        ) -> Result<SubprocessCapture, String> {
            self.calls
                .borrow_mut()
                .push((binary.to_string(), args.to_vec(), cwd.to_path_buf()));
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    #[test]
    fn helm_release_manifest_uses_owned_release_command() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([Ok(SubprocessCapture {
                stdout: "kind: ConfigMap\n".to_string(),
                event: json!({"binary":"helm"}),
            })])),
        };

        let manifest = helm_release_manifest(&runner, root.path(), "atlas-kind").expect("manifest");

        assert_eq!(manifest, "kind: ConfigMap\n");
        assert_eq!(runner.calls.borrow()[0].0, "helm");
        assert!(runner.calls.borrow()[0].1.contains(&"manifest".to_string()));
    }

    #[test]
    fn parse_prior_release_revision_uses_previous_history_entry() {
        let revision =
            parse_prior_release_revision(r#"[{"revision":4},{"revision":5},{"revision":6}]"#)
                .expect("revision");

        assert_eq!(revision, "5");
    }

    #[test]
    fn package_chart_for_evidence_uses_owned_package_directory_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/k8s/charts/bijux-atlas"))
            .expect("mkdir chart");
        let package_dir = root.path().join("ops/release/evidence/packages");
        let packaged_chart = package_dir.join("bijux-atlas-0.2.1.tgz");
        let runner = MockRunner {
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([Ok(SubprocessCapture {
                stdout: String::new(),
                event: json!({"binary":"helm"}),
            })])),
        };
        std::fs::create_dir_all(&package_dir).expect("mkdir package dir");
        std::fs::write(&packaged_chart, "chart").expect("write chart");

        let path = package_chart_for_evidence(&runner, root.path()).expect("package path");

        assert_eq!(path, packaged_chart);
        assert!(runner.calls.borrow()[0].1.contains(&"package".to_string()));
    }

    #[test]
    fn release_package_inventory_reports_hash_and_path() {
        let root = tempfile::tempdir().expect("tempdir");
        let chart_path = root.path().join("bijux-atlas.tgz");
        std::fs::write(&chart_path, "chart").expect("write chart");

        let payload = release_package_inventory(root.path(), &chart_path).expect("inventory");

        assert_eq!(payload["path"], "bijux-atlas.tgz");
        assert!(payload["sha256"].as_str().is_some());
    }
}
