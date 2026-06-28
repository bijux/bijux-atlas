// SPDX-License-Identifier: Apache-2.0
//! Shared report header fields for machine-readable reports.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportHeader {
    pub report_id: String,
    pub version: u64,
    pub inputs: serde_json::Value,
    #[serde(default)]
    pub artifacts: Vec<String>,
}

impl ReportHeader {
    pub fn new(
        report_id: impl Into<String>,
        version: u64,
        inputs: serde_json::Value,
        artifacts: Vec<String>,
    ) -> Self {
        Self {
            report_id: report_id.into(),
            version,
            inputs,
            artifacts,
        }
    }
}
