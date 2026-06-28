// SPDX-License-Identifier: Apache-2.0
//! Typed contract runner summary models.

use serde::{Deserialize, Serialize};

use crate::model::{ReportHeader, ReportRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractCaseStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractCaseResult {
    pub contract_id: String,
    pub contract_name: String,
    pub case_name: String,
    pub status: ContractCaseStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    #[serde(default)]
    pub artifact_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractRunPreflight {
    #[serde(default)]
    pub required_tools: Vec<String>,
    #[serde(default)]
    pub missing_tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRunCounts {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub not_run: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRunSummary {
    pub header: ReportHeader,
    pub mode: String,
    pub jobs: String,
    pub fail_fast: bool,
    pub preflight: ContractRunPreflight,
    pub counts: ContractRunCounts,
    #[serde(default)]
    pub reports: Vec<ReportRef>,
    pub cases: Vec<ContractCaseResult>,
}
