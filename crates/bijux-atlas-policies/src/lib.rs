#![forbid(unsafe_code)]

mod limits;
mod schema;
mod validate;

pub use limits::{
    DEFAULT_HEAVY_PROJECTION_LIMIT, DEFAULT_MAX_SERIALIZATION_BYTES, MAX_DEPTH_HARD, MAX_LOC_HARD,
    MAX_MODULES_PER_DIR_HARD, MAX_RS_FILES_PER_DIR_HARD, MAX_SCHEMA_BUMP_STEP,
    MIN_POLICY_SCHEMA_VERSION,
};
pub use schema::{
    CacheBudget, ConcurrencyBulkheads, DocumentedDefault, EndpointClassBudget, PolicyConfig,
    PolicyMode, PolicyModeProfile, PolicyModes, PolicySchema, PolicySchemaVersion, PublishGates,
    QueryBudgetPolicy, RateLimitPolicy, ResponseBudgetPolicy, StoreResiliencePolicy,
    TelemetryPolicy,
};
pub use validate::{
    canonical_config_json, load_policy_from_workspace, policy_config_path, policy_schema_path,
    validate_policy_change_requires_version_bump, validate_policy_config, resolve_mode_profile,
    validate_schema_version_transition, PolicyValidationError,
};

pub const CRATE_NAME: &str = "bijux-atlas-policies";
