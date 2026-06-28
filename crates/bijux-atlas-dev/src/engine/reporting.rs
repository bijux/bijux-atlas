// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use crate::engine::report_codec;
use crate::model::{ReportHeader, ReportRef, RunId, RunnableId};

#[derive(Debug, Clone)]
pub struct ArtifactStore {
    root: PathBuf,
}

impl ArtifactStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn runnable_root(&self, run_id: &RunId, runnable_id: &RunnableId) -> PathBuf {
        self.root.join(run_id.as_str()).join(runnable_id.as_str())
    }

    pub fn write_json_report(
        &self,
        run_id: &RunId,
        runnable_id: &RunnableId,
        report_id: &str,
        payload: &serde_json::Value,
    ) -> Result<ReportRef, String> {
        validate_run_result_report(report_id, payload)?;
        let root = self.runnable_root(run_id, runnable_id);
        fs::create_dir_all(&root)
            .map_err(|err| format!("create {} failed: {err}", root.display()))?;
        let path = root.join(format!("{report_id}.json"));
        report_codec::write_json(&path, payload)?;
        Ok(ReportRef {
            report_id: report_id.to_string(),
            path: path.display().to_string(),
        })
    }
}

pub fn validate_run_result_report(
    report_id: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let object = payload
        .as_object()
        .ok_or_else(|| format!("report `{report_id}` must be a JSON object"))?;
    for required in [
        "report_id",
        "version",
        "inputs",
        "artifacts",
        "run_id",
        "runnable_id",
        "status",
    ] {
        if !object.contains_key(required) {
            return Err(format!(
                "report `{report_id}` is missing required field `{required}`"
            ));
        }
    }
    let header = ReportHeader {
        report_id: object
            .get("report_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        version: object
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default(),
        inputs: object
            .get("inputs")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        artifacts: object
            .get("artifacts")
            .and_then(serde_json::Value::as_array)
            .map(|rows| {
                rows.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    };
    if header.report_id != report_id {
        return Err(format!(
            "report `{report_id}` must declare matching report_id, found `{}`",
            header.report_id
        ));
    }
    if header.version == 0 {
        return Err(format!("report `{report_id}` must declare version >= 1"));
    }
    Ok(())
}
