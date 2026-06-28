// SPDX-License-Identifier: Apache-2.0

use super::validate::{load_policy_from_workspace, PolicyValidationError};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PolicySchemaVersion {
    #[serde(rename = "1")]
    V1,
}

impl PolicySchemaVersion {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => "1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevAtlasPolicyMode {
    Dev,
    Ci,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckPolicyCompatibility {
    pub from: DevAtlasPolicyMode,
    pub to: DevAtlasPolicyMode,
    pub allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoPolicy {
    pub max_loc_warn: usize,
    pub max_loc_hard: usize,
    pub max_depth_hard: usize,
    pub max_rs_files_per_dir_hard: usize,
    pub max_modules_per_dir_hard: usize,
    pub loc_allowlist: Vec<String>,
    pub rs_files_per_dir_allowlist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpsPolicy {
    pub registry_relpath: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDocumentedDefault {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCategory {
    Repo,
    Docs,
    Ops,
    Configs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RatchetRule {
    pub id: String,
    pub max_allowed: usize,
    pub allowlist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relaxation {
    pub policy_id: String,
    pub reason: String,
    pub expires_on: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevAtlasPolicySetDocument {
    pub schema_version: PolicySchemaVersion,
    pub mode: DevAtlasPolicyMode,
    pub repo: RepoPolicy,
    pub ops: OpsPolicy,
    pub compatibility: Vec<CheckPolicyCompatibility>,
    pub documented_defaults: Vec<PolicyDocumentedDefault>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ratchets: Vec<RatchetRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relaxations: Vec<Relaxation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevAtlasPolicySet {
    pub schema_version: PolicySchemaVersion,
    pub mode: DevAtlasPolicyMode,
    pub repo_policy: RepoPolicy,
    pub ops_policy: OpsPolicy,
    pub compatibility: Vec<CheckPolicyCompatibility>,
    pub documented_defaults: Vec<PolicyDocumentedDefault>,
    pub ratchets: Vec<RatchetRule>,
    pub relaxations: Vec<Relaxation>,
}

impl DevAtlasPolicySet {
    pub fn load(repo_root: &Path) -> Result<Self, PolicyValidationError> {
        let doc = load_policy_from_workspace(repo_root)?;
        Ok(Self {
            schema_version: doc.schema_version,
            mode: doc.mode,
            repo_policy: doc.repo,
            ops_policy: doc.ops,
            compatibility: doc.compatibility,
            documented_defaults: doc.documented_defaults,
            ratchets: doc.ratchets,
            relaxations: doc.relaxations,
        })
    }

    pub fn to_document(&self) -> DevAtlasPolicySetDocument {
        DevAtlasPolicySetDocument {
            schema_version: self.schema_version,
            mode: self.mode,
            repo: self.repo_policy.clone(),
            ops: self.ops_policy.clone(),
            compatibility: self.compatibility.clone(),
            documented_defaults: self.documented_defaults.clone(),
            ratchets: self.ratchets.clone(),
            relaxations: self.relaxations.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub id: &'static str,
    pub category: PolicyCategory,
    pub description: &'static str,
}

pub const POLICY_REGISTRY: &[PolicyDefinition] = &[
    PolicyDefinition {
        id: "repo.max_loc_hard",
        category: PolicyCategory::Repo,
        description: "Maximum allowed lines of code per Rust source file",
    },
    PolicyDefinition {
        id: "repo.max_depth_hard",
        category: PolicyCategory::Repo,
        description: "Maximum allowed directory nesting depth",
    },
    PolicyDefinition {
        id: "ops.registry_relpath",
        category: PolicyCategory::Ops,
        description: "Canonical path to check registry SSOT",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyInputSnapshot {
    pub rust_file_line_counts: Vec<(String, usize)>,
    pub registry_relpath_exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub policy_id: String,
    pub category: PolicyCategory,
    pub message: String,
    pub evidence: String,
}

pub fn evaluate_policy_set_pure(
    policy: &DevAtlasPolicySet,
    snapshot: &PolicyInputSnapshot,
) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();
    for (path, lines) in &snapshot.rust_file_line_counts {
        if *lines > policy.repo_policy.max_loc_hard
            && !policy.repo_policy.loc_allowlist.contains(path)
            && !is_relaxed(&policy.relaxations, "repo.max_loc_hard")
        {
            violations.push(PolicyViolation {
                policy_id: "repo.max_loc_hard".to_string(),
                category: PolicyCategory::Repo,
                message: format!(
                    "file exceeds max_loc_hard ({lines} > {})",
                    policy.repo_policy.max_loc_hard
                ),
                evidence: path.clone(),
            });
        }
    }
    if !snapshot.registry_relpath_exists && !is_relaxed(&policy.relaxations, "ops.registry_relpath")
    {
        violations.push(PolicyViolation {
            policy_id: "ops.registry_relpath".to_string(),
            category: PolicyCategory::Ops,
            message: "configured ops registry path is missing".to_string(),
            evidence: policy.ops_policy.registry_relpath.clone(),
        });
    }
    violations
}

fn is_relaxed(relaxations: &[Relaxation], policy_id: &str) -> bool {
    relaxations.iter().any(|r| r.policy_id == policy_id)
}
