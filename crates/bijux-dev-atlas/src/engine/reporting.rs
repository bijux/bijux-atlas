// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{ReportRef, RunId, RunnableId};

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
        let rendered = serde_json::to_string_pretty(payload)
            .map_err(|err| format!("encode {report_id} failed: {err}"))?;
        fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
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
    for required in ["report_id", "run_id", "runnable_id", "status"] {
        if !object.contains_key(required) {
            return Err(format!(
                "report `{report_id}` is missing required field `{required}`"
            ));
        }
    }
    Ok(())
}
