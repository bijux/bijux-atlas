// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas_core::{
    render_json, run_checks, Capabilities, Fs, ProcessRunner, RunOptions, RunRequest, Selectors,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

struct TestFs;
impl Fs for TestFs {
    fn read_text(
        &self,
        repo_root: &Path,
        path: &Path,
    ) -> Result<String, bijux_dev_atlas_core::ports::AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        fs::read_to_string(target).map_err(|err| bijux_dev_atlas_core::ports::AdapterError::Io {
            op: "read_to_string",
            path: repo_root.join(path),
            detail: err.to_string(),
        })
    }
    fn exists(&self, repo_root: &Path, path: &Path) -> bool {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target.exists()
    }
    fn canonicalize(
        &self,
        repo_root: &Path,
        path: &Path,
    ) -> Result<PathBuf, bijux_dev_atlas_core::ports::AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target
            .canonicalize()
            .map_err(|err| bijux_dev_atlas_core::ports::AdapterError::Io {
                op: "canonicalize",
                path: target,
                detail: err.to_string(),
            })
    }
}

struct DeniedProcessRunner;
impl ProcessRunner for DeniedProcessRunner {
    fn run(
        &self,
        program: &str,
        _args: &[String],
        _repo_root: &Path,
    ) -> Result<i32, bijux_dev_atlas_core::ports::AdapterError> {
        Err(bijux_dev_atlas_core::ports::AdapterError::EffectDenied {
            effect: "subprocess",
            detail: format!("attempted to execute `{program}`"),
        })
    }
}

fn normalize_dynamic_fields(value: &mut Value) {
    if let Some(results) = value.get_mut("results").and_then(Value::as_array_mut) {
        for row in results {
            if let Some(duration_ms) = row.get_mut("duration_ms") {
                *duration_ms = Value::from(0u64);
            }
        }
    }
    for key in ["durations_ms", "timings_ms"] {
        if let Some(map) = value.get_mut(key).and_then(Value::as_object_mut) {
            for ms in map.values_mut() {
                *ms = Value::from(0u64);
            }
        }
    }
}

#[test]
fn report_json_matches_golden_shape_and_content() {
    let repo_root = root();
    let request = RunRequest {
        repo_root: repo_root.clone(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
        command: Some("check run".to_string()),
    };

    let report = run_checks(
        &DeniedProcessRunner,
        &TestFs,
        &request,
        &Selectors::default(),
        &RunOptions::default(),
    )
    .expect("run");

    let json = render_json(&report).expect("json");
    let replaced = json.replace(&repo_root.display().to_string(), "<repo_root>");
    let mut parsed: Value = serde_json::from_str(&replaced).expect("parsed");
    normalize_dynamic_fields(&mut parsed);
    let normalized = serde_json::to_string_pretty(&parsed).expect("normalized");

    let golden_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/report_default.json");
    if std::env::var("UPDATE_GOLDENS").ok().as_deref() == Some("1") {
        fs::write(&golden_path, &normalized).expect("write golden");
    }
    let expected = fs::read_to_string(golden_path).expect("golden");
    assert_eq!(normalized, expected);

    assert!(parsed.get("run_id").is_some());
    assert!(parsed.get("repo_root").is_some());
    assert!(parsed.get("results").is_some_and(Value::is_array));
    assert!(parsed.get("summary").is_some_and(Value::is_object));
}
