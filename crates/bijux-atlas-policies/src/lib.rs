#![forbid(unsafe_code)]

mod adapters;
mod evaluation;
mod limits;
mod policy_set;
mod schema;
mod validate;

pub use adapters::{load_policy_set_from_workspace, policy_config_path, policy_schema_path};
pub use evaluation::{
    evaluate_policy_set, evaluate_repository_metrics, PolicySeverity, PolicyViolation,
    RepositoryMetrics,
};
pub use limits::{
    DEFAULT_HEAVY_PROJECTION_LIMIT, DEFAULT_MAX_SERIALIZATION_BYTES, MAX_SCHEMA_BUMP_STEP,
    MIN_POLICY_SCHEMA_VERSION,
};
pub use schema::{
    CacheBudget, ConcurrencyBulkheads, DocumentedDefault, EndpointClassBudget, PolicyConfig,
    PolicyMode, PolicyModeProfile, PolicyModes, PolicySchema, PolicySchemaVersion, PolicySet,
    PublishGates,
    QueryBudgetPolicy, RateLimitPolicy, ResponseBudgetPolicy, StoreResiliencePolicy,
    TelemetryPolicy,
};
pub use policy_set::{parse_policy_set_json, validate_policy_set};
pub use validate::{
    canonical_config_json, load_policy_from_workspace, resolve_mode_profile,
    validate_policy_change_requires_version_bump, validate_policy_config,
    validate_schema_version_transition, PolicyValidationError,
};

pub const CRATE_NAME: &str = "bijux-atlas-policies";
