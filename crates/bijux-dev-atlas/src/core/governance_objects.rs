// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "governance_objects_parts/collect.rs"]
mod collect;
#[path = "governance_objects_parts/reports.rs"]
mod reports;
#[path = "governance_objects_parts/validation.rs"]
mod validation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceObject {
    pub id: String,
    pub domain: String,
    pub owner: String,
    pub consumers: Vec<String>,
    pub lifecycle: String,
    pub evidence: Vec<String>,
    pub links: Vec<String>,
    pub authority_source: String,
    pub reviewed_on: String,
}

#[derive(Debug, Clone)]
pub struct GovernanceValidationReport {
    pub errors: Vec<String>,
    pub orphan_rows: Vec<serde_json::Value>,
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path).map_err(|e| format!("read {} failed: {e}", path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", path.display()))
}

pub fn collect_governance_objects(repo_root: &Path) -> Result<Vec<GovernanceObject>, String> {
    collect::collect_governance_objects(repo_root)
}

pub fn governance_summary(objects: &[GovernanceObject]) -> BTreeMap<String, usize> {
    reports::governance_summary(objects)
}

pub fn validate_governance_objects(
    repo_root: &Path,
    objects: &[GovernanceObject],
) -> GovernanceValidationReport {
    validation::validate_governance_objects(repo_root, objects)
}

pub fn find_governance_object<'a>(
    objects: &'a [GovernanceObject],
    id: &str,
) -> Option<&'a GovernanceObject> {
    objects.iter().find(|obj| obj.id == id)
}

pub fn governance_object_schema() -> serde_json::Value {
    serde_json::json!({
      "schema_version": 1,
      "kind": "governance_object_schema",
      "required": ["id", "domain", "owner", "consumers", "lifecycle", "evidence", "links", "authority_source", "reviewed_on"],
      "fields": {
        "id": "{domain}:{kind}:{name}",
        "domain": "docs|configs|ops|make|docker|...",
        "owner": "stable owner id",
        "consumers": "list of real commands/paths/services",
        "lifecycle": "stable|experimental|deprecated",
        "evidence": "artifacts/governance/<domain>/...",
        "links": "authority links in repository",
        "authority_source": "single authority file path",
        "reviewed_on": "YYYY-MM-DD required for stable objects"
      }
    })
}

pub fn governance_summary_markdown(objects: &[GovernanceObject]) -> String {
    reports::governance_summary_markdown(objects)
}

pub fn governance_summary_paths(repo_root: &Path) -> (PathBuf, PathBuf) {
    reports::governance_summary_paths(repo_root)
}

pub fn governance_index_path(repo_root: &Path) -> PathBuf {
    reports::governance_index_path(repo_root)
}

pub fn governance_contract_coverage_path(repo_root: &Path) -> PathBuf {
    reports::governance_contract_coverage_path(repo_root)
}

pub fn governance_lane_coverage_path(repo_root: &Path) -> PathBuf {
    reports::governance_lane_coverage_path(repo_root)
}

pub fn governance_orphan_checks_path(repo_root: &Path) -> PathBuf {
    reports::governance_orphan_checks_path(repo_root)
}

pub fn governance_policy_surface_path(repo_root: &Path) -> PathBuf {
    reports::governance_policy_surface_path(repo_root)
}

pub fn governance_drift_path(repo_root: &Path) -> PathBuf {
    reports::governance_drift_path(repo_root)
}

pub fn governance_version(
    objects: &[GovernanceObject],
    contracts: &[serde_json::Value],
    lanes: &[serde_json::Value],
    report_mappings: &[serde_json::Value],
) -> String {
    reports::governance_version(objects, contracts, lanes, report_mappings)
}

pub fn governance_index_payload(
    repo_root: &Path,
    objects: &[GovernanceObject],
) -> serde_json::Value {
    reports::governance_index_payload(repo_root, objects)
}

pub fn governance_contract_coverage_payload(repo_root: &Path) -> serde_json::Value {
    reports::governance_contract_coverage_payload(repo_root)
}

pub fn governance_lane_coverage_payload(repo_root: &Path) -> serde_json::Value {
    reports::governance_lane_coverage_payload(repo_root)
}

pub fn governance_orphan_checks_payload(repo_root: &Path) -> serde_json::Value {
    reports::governance_orphan_checks_payload(repo_root)
}

pub fn governance_policy_surface_payload(repo_root: &Path) -> serde_json::Value {
    reports::governance_policy_surface_payload(repo_root)
}

pub fn governance_drift_payload(
    current_index: &serde_json::Value,
    previous_index: Option<&serde_json::Value>,
) -> serde_json::Value {
    reports::governance_drift_payload(current_index, previous_index)
}

pub fn governance_coverage_score(objects: &[GovernanceObject]) -> serde_json::Value {
    reports::governance_coverage_score(objects)
}

pub fn governance_coverage_path(repo_root: &Path) -> PathBuf {
    reports::governance_coverage_path(repo_root)
}

pub fn governance_orphan_report_path(repo_root: &Path) -> PathBuf {
    reports::governance_orphan_report_path(repo_root)
}

pub fn governance_orphan_report_payload(rows: &[serde_json::Value]) -> serde_json::Value {
    reports::governance_orphan_report_payload(rows)
}
