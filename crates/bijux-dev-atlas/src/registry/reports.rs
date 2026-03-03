// SPDX-License-Identifier: Apache-2.0
//! Typed report registry loading.

use std::path::Path;

use serde::{Deserialize, Serialize};

pub const REPORTS_REGISTRY_PATH: &str = "configs/reports/reports.registry.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportRegistryEntry {
    pub report_id: String,
    pub version: u64,
    pub schema_path: String,
    pub example_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportRegistry {
    pub schema_version: u64,
    pub reports: Vec<ReportRegistryEntry>,
}

impl ReportRegistry {
    pub fn load(repo_root: &Path) -> Result<Self, String> {
        let path = repo_root.join(REPORTS_REGISTRY_PATH);
        let text = std::fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?;
        let registry: Self = serde_json::from_str(&text)
            .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
        if registry.schema_version != 1 {
            return Err(format!("{} must declare schema_version=1", path.display()));
        }
        for report in &registry.reports {
            if report.report_id.trim().is_empty() {
                return Err(format!("{} contains a blank report_id", path.display()));
            }
            if report.version == 0 {
                return Err(format!(
                    "{} report `{}` must declare version >= 1",
                    path.display(),
                    report.report_id
                ));
            }
        }
        Ok(registry)
    }
}
