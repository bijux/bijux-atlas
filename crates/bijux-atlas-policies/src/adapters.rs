use std::fs;
use std::path::{Path, PathBuf};

use crate::policy_set::parse_policy_set_json;
use crate::schema::PolicySet;
use crate::validate::PolicyValidationError;

const POLICY_CONFIG_PATH: &str = "configs/policy/policy.json";
const POLICY_SCHEMA_PATH: &str = "configs/policy/policy.schema.json";

#[must_use]
pub fn policy_config_path(root: &Path) -> PathBuf {
    root.join(POLICY_CONFIG_PATH)
}

#[must_use]
pub fn policy_schema_path(root: &Path) -> PathBuf {
    root.join(POLICY_SCHEMA_PATH)
}

pub fn load_policy_set_from_workspace(root: &Path) -> Result<PolicySet, PolicyValidationError> {
    let config_raw = fs::read_to_string(policy_config_path(root))
        .map_err(|e| PolicyValidationError(format!("read policy config failed: {e}")))?;
    let schema_raw = fs::read_to_string(policy_schema_path(root))
        .map_err(|e| PolicyValidationError(format!("read policy schema failed: {e}")))?;

    parse_policy_set_json(&config_raw, &schema_raw)
}
