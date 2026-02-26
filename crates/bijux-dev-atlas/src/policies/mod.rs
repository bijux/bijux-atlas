// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod schema;
mod validate;

pub use schema::{
    evaluate_policy_set_pure, CheckPolicyCompatibility, DevAtlasPolicyMode, DevAtlasPolicySet,
    DevAtlasPolicySetDocument, OpsPolicy, PolicyCategory, PolicyDefinition,
    PolicyDocumentedDefault, PolicyInputSnapshot, PolicySchemaVersion, PolicyViolation,
    RatchetRule, Relaxation, RepoPolicy, POLICY_REGISTRY,
};
pub use validate::{
    canonical_policy_json, policy_config_path, policy_schema_path,
    validate_policy_change_requires_version_bump, validate_policy_registry_ids,
    validate_relaxation_expiry, PolicyValidationError,
};

pub const CRATE_NAME: &str = "bijux-dev-atlas";
