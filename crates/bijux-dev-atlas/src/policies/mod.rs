// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
//! `policies` defines policy schemas, defaults, and validation helpers.
//!
//! Boundary: policies may depend on `model` and `std`; it must not depend on `core` or `adapters`.

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

/// Development control-plane policy APIs and source-of-truth paths.
///
/// Keep runtime policies in `bijux-atlas-policies`; this namespace is only for dev-atlas
/// governance policy loading/validation against `ops/inventory/policies/dev-atlas-policy*.json`.
pub mod dev {
    pub use super::{
        canonical_policy_json, policy_config_path, policy_schema_path,
        validate_policy_change_requires_version_bump, validate_policy_registry_ids,
        validate_relaxation_expiry, DevAtlasPolicySet, DevAtlasPolicySetDocument, PolicyCategory,
        PolicyDefinition, PolicyDocumentedDefault, PolicyInputSnapshot, PolicySchemaVersion,
        PolicyValidationError, PolicyViolation, RatchetRule, Relaxation, CRATE_NAME,
    };
}
