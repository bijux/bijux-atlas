// SPDX-License-Identifier: Apache-2.0

use crate::load::manifest::LoadSuiteToml;
use crate::load::path_contracts::{load_run_root, load_summary_path};
use crate::load::plan_payload::load_plan_payload;
use crate::load::report_contract::{evaluate_load_report, LoadReportError};
use crate::load::report_payload::{load_report_payload, write_load_report};
use crate::load::run_payload::load_run_payload;
use serde_json::Value;
use std::path::Path;

pub trait LoadCommandRunner {
    fn run(&self, binary: &str, args: &[String], cwd: &Path) -> Result<(String, Value), String>;
}

pub fn load_plan_command_payload(
    suite: &str,
    suite_cfg: &LoadSuiteToml,
    manifest_errors: Vec<String>,
) -> Value {
    load_plan_payload(
        suite,
        &suite_cfg.script,
        &suite_cfg.dataset,
        &suite_cfg.thresholds,
        &suite_cfg.env,
        manifest_errors,
    )
}

pub fn load_report_command_payload(
    repo_root: &Path,
    suite: &str,
    suite_cfg: &LoadSuiteToml,
    run_id: &str,
) -> Result<(Value, i32), String> {
    let report =
        evaluate_load_report(repo_root, suite, suite_cfg, run_id).map_err(map_report_error)?;
    let report_path = write_load_report(repo_root, run_id, suite, &report)?;
    let payload = load_report_payload(&report_path.display().to_string(), &report);
    let exit_code = if payload["summary"]["errors"] == serde_json::json!(0) {
        0
    } else {
        1
    };
    Ok((payload, exit_code))
}

pub fn load_run_command_payload(
    runner: &impl LoadCommandRunner,
    repo_root: &Path,
    suite: &str,
    suite_cfg: &LoadSuiteToml,
    run_id: &str,
    report: Value,
    report_code: i32,
) -> Result<Value, String> {
    let dataset_path = repo_root.join(&suite_cfg.dataset);
    if !dataset_path.exists() {
        return Err(format!(
            "OPS_MANIFEST_ERROR: dataset path missing `{}` and downloads are disabled by default",
            suite_cfg.dataset
        ));
    }
    let out_dir = load_run_root(repo_root, run_id, suite);
    std::fs::create_dir_all(&out_dir).map_err(|err| err.to_string())?;
    let summary_path = load_summary_path(repo_root, run_id, suite);
    let script_path = repo_root.join(&suite_cfg.script);
    let mut argv = vec![
        "run".to_string(),
        script_path.display().to_string(),
        "--summary-export".to_string(),
        summary_path.display().to_string(),
    ];
    for (key, value) in &suite_cfg.env {
        argv.push("-e".to_string());
        argv.push(format!("{key}={value}"));
    }
    let (stdout, event) = runner.run("k6", &argv, repo_root)?;
    Ok(load_run_payload(
        suite,
        run_id,
        &stdout,
        event,
        &summary_path.display().to_string(),
        report,
        report_code,
    ))
}

fn map_report_error(error: LoadReportError) -> String {
    match error {
        LoadReportError::Read { .. } => format!("OPS_MANIFEST_ERROR: {}", error.detail()),
        LoadReportError::Parse { .. } => format!("OPS_SCHEMA_ERROR: {}", error.detail()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct MockRunner;

    impl LoadCommandRunner for MockRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<(String, Value), String> {
            Ok(("k6 output".to_string(), serde_json::json!({"binary":"k6"})))
        }
    }

    fn suite() -> LoadSuiteToml {
        LoadSuiteToml {
            script: "ops/load/k6/suites/mixed-80-20.js".to_string(),
            dataset: "ops/load/queries/pinned-v1.json".to_string(),
            thresholds: "ops/load/thresholds/mixed.thresholds.json".to_string(),
            env: BTreeMap::from([("K6_OUT".to_string(), "json=/tmp/out.json".to_string())]),
        }
    }

    #[test]
    fn load_plan_command_payload_keeps_suite_contract() {
        let payload = load_plan_command_payload("mixed", &suite(), vec![]);

        assert_eq!(payload["rows"][0]["suite"], "mixed");
        assert_eq!(payload["summary"]["errors"], 0);
    }

    #[test]
    fn load_run_command_payload_uses_owned_summary_path() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/load/k6/suites")).expect("mkdir suites");
        std::fs::create_dir_all(root.path().join("ops/load/queries")).expect("mkdir queries");
        std::fs::write(root.path().join("ops/load/k6/suites/mixed-80-20.js"), "").expect("script");
        std::fs::write(root.path().join("ops/load/queries/pinned-v1.json"), "{}").expect("dataset");

        let payload = load_run_command_payload(
            &MockRunner,
            root.path(),
            "mixed",
            &suite(),
            "atlas-run",
            serde_json::json!({"summary":{"errors":0}}),
            0,
        )
        .expect("load run payload");

        assert_eq!(payload["rows"][0]["subprocess_event"]["binary"], "k6");
        assert!(payload["rows"][0]["summary_path"]
            .as_str()
            .is_some_and(|value| value.ends_with("k6-summary.json")));
    }
}
