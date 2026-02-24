#![forbid(unsafe_code)]

mod schema;
mod validate;

pub use schema::{
    CheckPolicyCompatibility, DevAtlasPolicyMode, DevAtlasPolicySet, DevAtlasPolicySetDocument,
    OpsPolicy, PolicyDocumentedDefault, PolicySchemaVersion, RepoPolicy,
};
pub use validate::{
    canonical_policy_json, policy_config_path, policy_schema_path, PolicyValidationError,
};

pub const CRATE_NAME: &str = "bijux-dev-atlas-policies";
