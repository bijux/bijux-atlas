use crate::validate::{load_policy_from_workspace, PolicyValidationError};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevAtlasPolicySetDocument {
    pub schema_version: PolicySchemaVersion,
    pub mode: DevAtlasPolicyMode,
    pub repo: RepoPolicy,
    pub ops: OpsPolicy,
    pub compatibility: Vec<CheckPolicyCompatibility>,
    pub documented_defaults: Vec<PolicyDocumentedDefault>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevAtlasPolicySet {
    pub schema_version: PolicySchemaVersion,
    pub mode: DevAtlasPolicyMode,
    pub repo_policy: RepoPolicy,
    pub ops_policy: OpsPolicy,
    pub compatibility: Vec<CheckPolicyCompatibility>,
    pub documented_defaults: Vec<PolicyDocumentedDefault>,
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
        }
    }
}
